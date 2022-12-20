use std::io;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

use nix::unistd::Uid;
use regex::bytes::Regex;
use shell_escape::unix::escape;
use structopt::StructOpt;
use subprocess::{Exec, Redirection};

const ANSI_CLEAR_SCREEN: &[u8] = &[0x1b, b'[', b'2', b'J'];

fn wait_millisec(wait: u32) {
    if wait > 0 {
        let (s, ms) = (wait / 1000, wait % 1000);
        let duration = Duration::new(s as u64, ms * 1_000_000);
        thread::sleep(duration);
    }
}

// ref: https://stackoverflow.com/questions/35901547/how-can-i-find-a-subsequence-in-a-u8-slice
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

/// A TUI app to replay text including ANSI escape sequencs.
/// Read text from the standard input and write to the standard output, with inserting a wait when a clear-screen ANSI escape sequence or a carridge-return char appears.
#[derive(StructOpt, Debug)]
#[structopt(name = "ansi-flip-book")]
enum Opt {
    Play(Play),
    Log(Log),
}

#[derive(StructOpt, Debug)]
struct Play {
    /// wait of clear screen, in millisecond
    #[structopt(short = "c", long, default_value = "200")]
    wait_clear_screen: u32,

    /// wait of carrige return (CR), in millisecond
    #[structopt(short = "r", long, default_value = "50")]
    wait_carrige_return: u32,

    /// wait of new line (LF), in millisecond
    #[structopt(short = "n", long, default_value = "5")]
    wait_new_line: u32,

    /// wait of user typeing, in millisecond
    #[structopt(short = "u", long, default_value = "80")]
    wait_user_typing: u32,

    /// regex pattern for shell prompt
    #[structopt(short = "p", long, default_value = "^(.+@.+:.+[$] |[$] )")]
    shell_prompt: String,
}

#[derive(StructOpt, Debug)]
struct Log {
    /// argv
    cmd: Vec<String>,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    match opt {
        Opt::Play(opt_play) => main_play(opt_play),
        Opt::Log(opt_record) => main_record(opt_record),
    }
}

fn main_play(opt: Play) -> io::Result<()> {
    let shell_prompt_pattern = Regex::new(&opt.shell_prompt).unwrap();

    let mut stdout = io::stdout();
    let mut stdin_iter = io::stdin().bytes();
    loop {
        // read until either \n or \r
        let mut line: Vec<u8> = vec![];
        let mut last_b: Option<u8> = None;
        while let Some(r) = stdin_iter.next() {
            let b = r?;
            line.push(b);
            if b == b'\n' && last_b != Some(b'\\') || b == b'\r' {
                break;
            }
            last_b = Some(b);
        }

        // reached EOF?
        if line.len() == 0 {
            break;
        }

        if let Some(caps) = shell_prompt_pattern.captures(&line) {
            let prompt_end_pos = caps.get(1).unwrap().end();
            stdout.write_all(&line[..prompt_end_pos])?;
            stdout.flush()?;
            for b in line[prompt_end_pos..].iter() {
                stdout.write_all(&[*b])?;
                stdout.flush()?;
                wait_millisec(opt.wait_user_typing);
            }
        } else {
            while let Some(pos) = find_subsequence(&line, ANSI_CLEAR_SCREEN) {
                stdout.write_all(&line[..pos])?;
                stdout.flush()?;
                wait_millisec(opt.wait_clear_screen);
                line.drain(..pos + ANSI_CLEAR_SCREEN.len());
            }

            if line.last() == Some(&b'\r') {
                stdout.write_all(&line[..line.len() - 1])?;
                stdout.flush()?;
                wait_millisec(opt.wait_carrige_return);
                stdout.write_all(&line[line.len() - 1..])?;
            } else {
                stdout.write_all(&line)?;
                stdout.flush()?;
                wait_millisec(opt.wait_new_line);
            }
        }
    }
    stdout.write_all(b"\n")?;
    stdout.flush()?;

    Ok(())
}

// ref: ANSI Escape Sequences https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797

fn main_record(opt: Log) -> io::Result<()> {
    let mut cmd = vec!["unbuffer", "-p"];
    for s in opt.cmd.iter() {
        cmd.push(&s);
    }

    let exec = Exec::cmd(cmd[0]).args(&cmd[1..]).stderr(Redirection::Merge);
    let r = exec.stream_stdout().unwrap();

    let mut stdout = io::stdout();

    stdout.write(
        if Uid::effective().is_root() { b"#" } else { b"$" }
    )?;
    for a in opt.cmd {
        stdout.write(b" ")?;
        let s: String = escape(a.into()).to_string();
        stdout.write(s.as_bytes())?;
    }
    stdout.write(b"\n")?;

    for b in r.bytes() {
        stdout.write(&[b.unwrap()])?;
    }
    stdout.flush()?;

    Ok(())
}
