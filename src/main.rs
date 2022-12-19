use std::io;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

use structopt::StructOpt;

const BUFFER_SIZE: usize = 4 * 1024;
const MAX_UNFLUSHED_LINE_SIZE: usize = 64 * 1024;
const ANSI_CLEAR_SCREEN: &[u8] = &[0x1b, b'[', b'2', b'J'];

fn wait_millisec(wait: u32) {
    if wait > 0 {
        let (s, ms) = (wait / 1000, wait % 1000);
        let duration = Duration::new(s as u64, ms * 1000_000);
        thread::sleep(duration);
    }
}

/// A TUI app to replay text including ANSI escape sequencs.
/// Read text from the standard input and write to the standard output, with inserting a wait when a clear-screen ANSI escape sequence or a carridge-return char appears.
#[derive(StructOpt, Debug)]
#[structopt(name = "ansi-flip-book")]
struct Opt {
    /// wait of clear screen, in millisecond
    #[structopt(short = "c", long, default_value = "200")]
    wait_clear_screen: u32,

    /// wait of carrige return (CR), in millisecond
    #[structopt(short = "r", long, default_value = "50")]
    wait_carrige_return: u32,

    /// wait of new line (LF), in millisecond
    #[structopt(short = "n", long, default_value = "5")]
    wait_new_line: u32,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    let longest_ansi_len = ANSI_CLEAR_SCREEN.len();

    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut line: Vec<u8> = vec![];

    while let Ok(n) = stdin.read(&mut buf) {
        if n == 0 { // EOF
            stdout.write_all(&line)?;
            break;
        }

        // Adjust the search start position to prevent it falling into the middle of an ANSI escape sequence
        let last_line_len = line.len();
        line.extend_from_slice(&buf[0..n]);
        let start = if last_line_len < longest_ansi_len { 0 } else { last_line_len - longest_ansi_len };

        let mut i = start;
        while i < line.len() {
            let prev_line_len = line.len();
            let prev_i = i;
            let b = line[i];
            match b {
                b'\n' => {
                    stdout.write_all(&line[..i + 1])?;
                    stdout.flush()?;
                    wait_millisec(opt.wait_new_line);
                    line.drain(..i + 1);
                    i = 0;
                }
                b'\r' => {
                    stdout.write_all(&line[..i])?;
                    stdout.flush()?;
                    wait_millisec(opt.wait_carrige_return);
                    stdout.write_all(&line[i..i + 1])?;
                    line.drain(..i + 1);
                    i = 0;
                }
                0x1b => {
                    if line[i..i + ANSI_CLEAR_SCREEN.len()] == *ANSI_CLEAR_SCREEN {
                        stdout.write_all(&line[..i])?;
                        stdout.flush()?;
                        wait_millisec(opt.wait_clear_screen);
                        stdout.write_all(&line[i..i + ANSI_CLEAR_SCREEN.len()])?;
                        line.drain(..i + ANSI_CLEAR_SCREEN.len());
                        i = 0;
                    } else {
                        i += 1;
                    }
                }
                _ => {
                    i += 1;
                }
            }
            assert!(line.len() < prev_line_len || i > prev_i);
        }

        if line.len() > MAX_UNFLUSHED_LINE_SIZE {
            let end = line.len() - longest_ansi_len;
            stdout.write_all(&line[..end])?;
            line.drain(..end);
        }
    }
    stdout.write_all(b"\n")?;
    stdout.flush()?;

    Ok(())
}

// ref: ANSI Escape Sequences https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797