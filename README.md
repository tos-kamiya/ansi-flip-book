# ansi-flip-book

A TUI app to replay text including ANSI escape sequencs.

Read text from the standard input and write to the standard output, with inserting a wait when a clear-screen ANSI escape sequence or a carridge-return char appears.

## Installation

prequresties:  

The tool `ansi-flip-book` requires `unbuffer` command in recording command-line session.
In case of Ubuntu OS, the command is a part of `expect` package.

```
sudo apt install expect
```

To install `ansi-flip-book`,

```sh
cargo install --git https://github.com/tos-kamiya/ansi-flip-book
```

To uninstall `ansi-flip-book`,

```sh
cargo uninstall ansi-flip-book
```

## Usage

To replay a text including ANSI escape sequences:

```sh
ansi-flip-book play < some-text-file
```

To record a command-line session as a text file:

```sh
ansi-flip-book log -- some-commmand-line > some-text-file
```

## Samples

The file `samples/pip-install-opencv-python.log` is a sample of recording of a command-line session.
The following command line will replay the run of the command, including coloring by ANSI escape sequences in the output.

```sh
ansi-flip-book play < samples/pip-install-opencv-python.log
```

## Todo

- [ ] Better replay of typing (looks as if a human typing)
- [ ] Fix performance glitch in `log` mode.
- [ ] Simple slow replay mode

## Similar Apps

* `scriptreplay` [https://man.gnu.org.ua/manpage/?1+scriptreplay](https://man.gnu.org.ua/manpage/?1+scriptreplay)