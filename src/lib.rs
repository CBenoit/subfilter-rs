use regex::Regex;
use std::ffi::OsStr;
use time::Duration;

#[derive(Debug)]
pub enum ContextConfig {
    Lines {
        before_context: u32,
        after_context: u32,
    },
    Durations {
        before_duration: Duration,
        after_duration: Duration,
    },
}

#[derive(Debug)]
pub struct Config {
    pub context: ContextConfig,
    pub pattern: Option<String>,
    pub pre_replace: Option<(String, String)>,
    pub post_replace: Option<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub start_ms: Duration,
    pub end_ms: Duration,
    pub line: String,
    pub is_match: bool,
}

struct Entries<PostProcess> {
    inner: Vec<Entry>,
    post_process: Option<PostProcess>,
}

impl<PostProcess> Entries<PostProcess>
where
    PostProcess: Fn(&mut Entry),
{
    fn new(post_process: Option<PostProcess>) -> Self {
        Self {
            inner: Vec::new(),
            post_process,
        }
    }

    fn push(&mut self, mut e: Entry) {
        if let Some(post_process) = &self.post_process {
            post_process(&mut e);
        }

        self.inner.push(e);
    }
}

pub fn parse(extension: Option<&OsStr>, file_content: &str, conf: Config) -> Vec<Entry> {
    let pattern = conf
        .pattern
        .map(|pat| Regex::new(&pat).expect("bad pattern"));
    let pre_replace = conf
        .pre_replace
        .map(|(pat, rep)| (Regex::new(&pat).expect("bad pre replace pattern"), rep));
    let post_replace = conf
        .post_replace
        .map(|(pat, rep)| (Regex::new(&pat).expect("bad post replace pattern"), rep));

    let pre_process = pre_replace.map(|(pat, rep)| {
        move |e: &mut Entry| e.line = pat.replace_all(&e.line, rep.as_str()).into_owned()
    });

    let post_process = post_replace.map(|(pat, rep)| {
        move |e: &mut Entry| e.line = pat.replace_all(&e.line, rep.as_str()).into_owned()
    });

    let format = subparse::get_subtitle_format(extension, file_content.as_bytes())
        .expect("couldn't detect subtitle format");
    let file = subparse::parse_str(format, file_content, 30f64).expect("couldn't parse subtitles"); // FIXME: fps arg was set arbitrarily

    let mut last_match_index = i64::MIN;
    let mut last_match_end_time = None;
    let mut previous_entries = Vec::new();
    let mut entries = Entries::new(post_process);

    for (i, entry) in file
        .get_subtitle_entries()
        .expect("couldn't get subtitles entries")
        .into_iter()
        .enumerate()
    {
        if let Some(line) = entry.line {
            let mut current_entry = Entry {
                start_ms: Duration::milliseconds(entry.timespan.start.msecs()),
                end_ms: Duration::milliseconds(entry.timespan.end.msecs()),
                line,
                is_match: false,
            };

            if let Some(pre_process) = &pre_process {
                pre_process(&mut current_entry);

                // if all the text is gone, skip it
                if current_entry.line.is_empty() {
                    continue;
                }
            }

            let keep = if let Some(pattern) = &pattern {
                if pattern.is_match(&current_entry.line) {
                    last_match_index = i as i64;
                    last_match_end_time = Some(current_entry.end_ms);
                    current_entry.is_match = true;
                    true
                } else {
                    false
                }
            } else {
                true
            };

            if keep {
                match conf.context {
                    ContextConfig::Lines { .. } => {
                        for e in previous_entries.drain(..) {
                            entries.push(e);
                        }
                    }
                    ContextConfig::Durations {
                        before_duration, ..
                    } => {
                        for e in previous_entries.drain(..) {
                            if current_entry.start_ms - e.end_ms < before_duration {
                                entries.push(e);
                            }
                        }
                    }
                }

                entries.push(current_entry);
            } else {
                match conf.context {
                    ContextConfig::Lines {
                        before_context,
                        after_context,
                    } => {
                        if i as i64 <= last_match_index + i64::from(after_context) {
                            previous_entries.clear();
                            entries.push(current_entry);
                        } else if before_context >= 1 {
                            if previous_entries.len() >= before_context as usize {
                                previous_entries.remove(0);
                            }
                            previous_entries.push(current_entry);
                        }
                    }
                    ContextConfig::Durations { after_duration, .. } => match last_match_end_time {
                        Some(last_match_end_time)
                            if current_entry.start_ms - last_match_end_time <= after_duration =>
                        {
                            previous_entries.clear();
                            entries.push(current_entry);
                        }
                        _ => previous_entries.push(current_entry),
                    },
                }
            }
        }
    }

    entries.inner
}
