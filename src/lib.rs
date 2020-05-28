use regex::Regex;
use std::ffi::OsStr;
use time::Duration;

#[derive(Debug)]
pub struct Config {
    pub before_context: u32,
    pub after_context: u32,
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

    let pre_process: Box<dyn Fn(&mut Entry)> = if let Some((pat, rep)) = pre_replace {
        Box::new(move |e| e.line = pat.replace_all(&e.line, rep.as_str()).into_owned())
    } else {
        Box::new(|_| {})
    };

    let post_process: Box<dyn Fn(&mut Entry)> = if let Some((pat, rep)) = post_replace {
        Box::new(move |e| e.line = pat.replace_all(&e.line, rep.as_str()).into_owned())
    } else {
        Box::new(|_| {})
    };

    let format = subparse::get_subtitle_format(extension, file_content.as_bytes())
        .expect("couldn't detect subtitle format");
    let file = subparse::parse_str(format, file_content, 30f64).expect("couldn't parse subtitles"); // FIXME: fps arg was set arbitrarily

    let mut nb_to_keep_after: u32 = 0;
    let mut previous_entries = Vec::with_capacity(conf.before_context as usize);

    let mut entries = Vec::new();
    for entry in file
        .get_subtitle_entries()
        .expect("couldn't get subtitles entries")
    {
        if let Some(line) = entry.line {
            let mut current_entry = Entry {
                start_ms: Duration::milliseconds(entry.timespan.start.msecs()),
                end_ms: Duration::milliseconds(entry.timespan.end.msecs()),
                line,
                is_match: false,
            };

            pre_process(&mut current_entry);

            let keep = if let Some(pattern) = &pattern {
                current_entry.is_match = pattern.is_match(&current_entry.line);
                current_entry.is_match
            } else {
                true
            };

            if keep {
                for mut e in previous_entries.drain(..) {
                    post_process(&mut e);
                    entries.push(e);
                }

                post_process(&mut current_entry);
                entries.push(current_entry);

                nb_to_keep_after = conf.after_context;

                continue;
            }

            if nb_to_keep_after > 0 {
                nb_to_keep_after -= 1;

                previous_entries.clear();

                post_process(&mut current_entry);
                entries.push(current_entry);

                continue;
            }

            if conf.before_context >= 1 {
                if previous_entries.len() >= conf.before_context as usize {
                    previous_entries.remove(0);
                }
                previous_entries.push(current_entry);

                continue;
            }
        }
    }

    entries
}
