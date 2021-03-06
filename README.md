[![Crates.io](https://img.shields.io/crates/v/subfilter.svg)](https://crates.io/crates/subfilter)
[![docs.rs](https://docs.rs/subfilter/badge.svg)](https://docs.rs/subfilter)
![Crates.io](https://img.shields.io/crates/l/subfilter)

# subfilter

CLI tool to process and filter out stuff from subtitle files.

I use it to get pre-processed clean output from subtitle files.
This tool is similar to `grep` or `rip-grep` and alike, but has knowledge
of subtitle file formats. In the context of this tool, a "line" is equivalent
to a full subtitle entry (might contains several new lines).
You can also request to print context lines by duration difference from the
matched line.

## Usage

```
    subfilter [FLAGS] [OPTIONS] <file-path> [pattern]

FLAGS:
    -h, --help         Prints help information
        --hide-time    Whether timecode should be shown for the first line
        --no-color     Disable color output for matching part
    -V, --version      Prints version information
    -v, --verbose      Verbose output

OPTIONS:
    -A, --after-context <after-context>                  Number of lines to show after each match [default: 0]
    -C, --context <around-context>
            Number of lines to show before and after each match. This overrides both the -B/--before-context and
            -A/--after-context flags
    -B, --before-context <before-context>                Number of lines to show after each match [default: 0]
        --post-replace-pattern <post-replace-pattern>
            Pattern to replace after pattern matching (see https://docs.rs/regex/1.3.7/regex/)

        --post-replace-with <post-replace-with>
            Replacement string after pattern matching (see https://docs.rs/regex/1.3.7/regex/)

        --pre-replace-pattern <pre-replace-pattern>
            Pattern to replace before pattern matching (see https://docs.rs/regex/1.3.7/regex/)

        --pre-replace-with <pre-replace-with>
            Replacement string before pattern matching (see https://docs.rs/regex/1.3.7/regex/)

    -i, --sep-interval <separation-interval-ms>
            Separate blocks if next timecode is later by an offset of this value in milliseconds [default: 5000]

        --time-after <time-after-context>
            Duration threshold in milliseconds to decide whether we show a line after a match. This overrides
            -C/--context -B/--before-context and -A/--after-context flags
        --time-around <time-around-context>
            Duration threshold in milliseconds to decide whether we show a line around a match. This overrides --time-
            after, --time-before, -C/--context -B/--before-context and -A/--after-context flags
        --time-before <time-before-context>
            Duration threshold in milliseconds to decide whether we show a line before a match. This overrides
            -C/--context -B/--before-context and -A/--after-context flags

ARGS:
    <file-path>    Input file
    <pattern>      Pattern to find (see https://docs.rs/regex/1.3.7/regex/)
```

## Demo

Basic usage:

![Basic usage](https://i.imgur.com/bPwrNC9.png)

With some regex preprocessing step:

![With preprocessing](https://i.imgur.com/c50IsqX.png)

## Examples

Print all lines containing "hello"
```
subfilter subs.srt hello
```

Print all lines containing "hello" or "hi"
```
subfilter subs.ass "(hello|hi")"
```

Print all lines containing "hello" with the previous line and the next one as context. 
```
subfilter -A 10 -B 1 subs.ass hello
```

Print all lines containing "hello world" but apply a match and replace regex before to strip html tags.
That way, `<span>hello</span> world` is also matched by the filtering pattern.
```
subfilter --pre-replace-pattern="<\s*[\.a-zA-Z]+[^>]*>(.*?)<\s*/\s*[\.a-zA-Z]+>" --pre-replace-with="\$1" subs.ass "hello world"
```

Print all lines containing "hello world" but replace "hello" by "hi".
```
subfilter --post-replace-pattern="hello" --post-replace-with="hi" subs.srt "hello world"
```

# Install

- [Install Rust toolchain](https://www.rust-lang.org/tools/install)
- Download and compile by running following command:
    ```
    $ cargo install subfilter
    ```
