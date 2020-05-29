use colored::{Color, Colorize};
use std::path::PathBuf;
use structopt::StructOpt;
use subfilter::{parse, Config, ContextConfig};
use time::Duration;

#[derive(Debug, StructOpt)]
#[structopt(about)]
struct Opt {
    /// Disable color output for matching part
    #[structopt(long)]
    no_color: bool,

    /// Whether timecode should be shown for the first line
    #[structopt(long)]
    hide_time: bool,

    /// Verbose output
    #[structopt(short = "v", long)]
    verbose: bool,

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

    /// Duration threshold in milliseconds to decide whether we show a line around a match.
    /// This overrides --time-after, --time-before, -C/--context -B/--before-context
    /// and -A/--after-context flags.
    #[structopt(long = "time-around")]
    time_around_context: Option<i64>,

    /// Duration threshold in milliseconds to decide whether we show a line after a match.
    /// This overrides -C/--context -B/--before-context and -A/--after-context flags.
    #[structopt(long = "time-after")]
    time_after_context: Option<i64>,

    /// Duration threshold in milliseconds to decide whether we show a line before a match.
    /// This overrides -C/--context -B/--before-context and -A/--after-context flags.
    #[structopt(long = "time-before")]
    time_before_context: Option<i64>,

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

fn format_duration(d: &Duration) -> String {
    let h = d.whole_hours();
    let min = d.whole_minutes() % 60;
    let s = d.whole_seconds() % 60;
    let ms = d.whole_milliseconds() % 1000;
    format!("{}:{}:{}.{}", h, min, s, ms)
}

fn main() {
    let opt = Opt::from_args();

    if opt.verbose {
        println!("{}: {:?}\n", "args".color(Color::Yellow), opt);
    }

    let context_config = if let Some(around) = opt.time_around_context {
        let around = Duration::milliseconds(around);
        ContextConfig::Durations {
            before_duration: around,
            after_duration: around,
        }
    } else if opt.time_before_context.is_some() || opt.time_after_context.is_some() {
        ContextConfig::Durations {
            before_duration: Duration::milliseconds(opt.time_before_context.unwrap_or(0)),
            after_duration: Duration::milliseconds(opt.time_after_context.unwrap_or(0)),
        }
    } else {
        ContextConfig::Lines {
            before_context: opt.around_context.unwrap_or(opt.before_context),
            after_context: opt.around_context.unwrap_or(opt.after_context),
        }
    };

    let pre_replace_pattern = opt.pre_replace_pattern;
    let pre_replace_with = opt.pre_replace_with;
    let post_replace_pattern = opt.post_replace_pattern;
    let post_replace_with = opt.post_replace_with;

    let config = Config {
        context: context_config,
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

    if opt.verbose {
        println!("{}: {:?}\n", "config".color(Color::Yellow), config);
    }

    let content = std::fs::read_to_string(&opt.file_path).unwrap();

    let entries = parse(opt.file_path.extension(), &content, config);

    let sep_interval = Duration::milliseconds(opt.separation_interval_ms);
    let time_decoration = "###".color(Color::Yellow);

    let mut previous_timecode: Option<Duration> = None;
    for entry in entries {
        match previous_timecode.take() {
            Some(prev) if entry.start_ms - prev >= sep_interval => {
                if !opt.hide_time {
                    if opt.no_color {
                        println!(
                            "### {} ###\n\n### {} ###",
                            format_duration(&prev),
                            format_duration(&entry.start_ms),
                        );
                    } else {
                        println!(
                            "{} {} {}\n\n{} {} {}",
                            time_decoration,
                            format_duration(&prev).color(Color::BrightBlue),
                            time_decoration,
                            time_decoration,
                            format_duration(&entry.start_ms).color(Color::BrightBlue),
                            time_decoration
                        );
                    }
                } else {
                    println!();
                }
            }
            None if !opt.hide_time => {
                if opt.no_color {
                    println!("### {} ###", format_duration(&entry.start_ms),);
                } else {
                    println!(
                        "{} {} {}",
                        time_decoration,
                        format_duration(&entry.start_ms).color(Color::BrightBlue),
                        time_decoration
                    );
                }
            }
            _ => {}
        }

        previous_timecode = Some(entry.end_ms);

        if entry.is_match && !opt.no_color {
            println!("{}", entry.line.color(Color::BrightRed));
        } else {
            println!("{}", entry.line);
        }
    }

    match previous_timecode.take() {
        Some(prev) if !opt.hide_time => {
            if opt.no_color {
                println!("### {} ###", format_duration(&prev));
            } else {
                println!(
                    "{} {} {}",
                    time_decoration,
                    format_duration(&prev).color(Color::BrightBlue),
                    time_decoration
                );
            }
        }
        _ => {}
    }
}
