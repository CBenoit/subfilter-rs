use std::path::PathBuf;
use structopt::StructOpt;
use subfilter::{parse, Config};
use time::Duration;

#[derive(Debug, StructOpt)]
#[structopt(about)]
struct Opt {
    // Whether timecode should be shown for the first line
    #[structopt(long)]
    hide_time: bool,

    /// Separate blocks if next timecode is later by an offset of this value in milliseconds.
    #[structopt(short = "i", long = "sep-interval", default_value = "5000")]
    separation_interval_ms: i64,

    /// Number of lines to show before and after each match.
    /// This overrides both the -B/--before-context and -A/--after-context flags.
    #[structopt(short = "C", long = "context")]
    around_context: Option<u32>,

    /// Number of lines to show after each match.
    #[structopt(short = "A", long, default_value = "0")]
    after_context: u32,

    /// Number of lines to show after each match.
    #[structopt(short = "B", long, default_value = "0")]
    before_context: u32,

    /// Pattern to replace before pattern matching (see https://docs.rs/regex/1.3.7/regex/)
    #[structopt(long = "pre-replace-pattern")]
    pre_replace_pattern: Option<String>,

    /// Replacement string before pattern matching (see https://docs.rs/regex/1.3.7/regex/)
    #[structopt(long = "pre-replace-with")]
    pre_replace_with: Option<String>,

    /// Pattern to replace after pattern matching (see https://docs.rs/regex/1.3.7/regex/)
    #[structopt(long = "post-replace-pattern")]
    post_replace_pattern: Option<String>,

    /// Replacement string after pattern matching (see https://docs.rs/regex/1.3.7/regex/)
    #[structopt(long = "post-replace-with")]
    post_replace_with: Option<String>,

    /// Input file
    #[structopt(parse(from_os_str))]
    file_path: PathBuf,

    /// Pattern to find (see https://docs.rs/regex/1.3.7/regex/)
    pattern: Option<String>,
}

fn print_duration(d: &Duration) {
    let h = d.whole_hours();
    let min = d.whole_minutes() % 60;
    let s = d.whole_seconds() % 60;
    let ms = d.whole_milliseconds() % 1000;
    println!("### {}:{}:{}.{} ###", h, min, s, ms);
}

fn main() {
    let opt = Opt::from_args();

    let pre_replace_pattern = opt.pre_replace_pattern;
    let pre_replace_with = opt.pre_replace_with;
    let post_replace_pattern = opt.post_replace_pattern;
    let post_replace_with = opt.post_replace_with;

    let config = Config {
        before_context: opt.around_context.unwrap_or(opt.before_context),
        after_context: opt.around_context.unwrap_or(opt.after_context),
        pattern: opt.pattern,
        pre_replace: pre_replace_pattern
            .map(|pat| (pat, pre_replace_with.expect("pre-replace-with is missing"))),
        post_replace: post_replace_pattern.map(|pat| {
            (
                pat,
                post_replace_with.expect("post-replace-with is missing"),
            )
        }),
    };

    let content = std::fs::read_to_string(&opt.file_path).unwrap();

    let entries = parse(opt.file_path.extension(), &content, config);

    let mut previous_timecode: Option<Duration> = None;
    let sep_interval = Duration::milliseconds(opt.separation_interval_ms);

    for entry in entries {
        match previous_timecode.take() {
            Some(prev) if entry.start_ms - prev >= sep_interval => {
                if !opt.hide_time {
                    print_duration(&prev);
                    println!();
                    print_duration(&entry.start_ms);
                } else {
                    println!();
                }
            }
            None if !opt.hide_time => {
                print_duration(&entry.start_ms);
            }
            _ => {}
        }

        previous_timecode = Some(entry.end_ms);

        println!("{}", entry.line);
    }
}
