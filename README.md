# ansi-flip-book

A TUI app to replay text including ANSI escape sequencs.

Read text from the standard input and write to the standard output, with inserting a wait when a clear-screen ANSI escape sequence or a carridge-return char appears.

## Installation

To install,

```sh
cargo install --git https://github.com/tos-kamiya/ansi-flip-book
```

To uninstall,

```sh
cargo uninstall ansi-flip-book
```

## Usage

```sh
ansi-flip-book < any-log-file
```
