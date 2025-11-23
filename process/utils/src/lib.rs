use serde::Serialize;
use std::collections::HashSet;
use std::io::BufRead;

pub fn make_sponsor_set<T: BufRead>(
    reader: T,
) -> Result<HashSet<String>, std::io::Error> {
    reader
        .lines()
        .skip(1)
        .filter_map(|read_line| {
            let Ok(line) = read_line else {
                return Some(read_line);
            };

            if line.starts_with(r#"","#) || line.starts_with(r#""""#) {
                return None;
            }
            if line.starts_with('"') {
                return Some(Ok(line.trim_start_matches('"').to_string()));
            }

            if let Some((video_id, _)) = line.split_once(",") {
                Some(Ok(video_id.to_string()))
            } else {
                None
            }
        })
        .collect()
}

pub fn clean_url(url: &str) -> String {
    let cleaned = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches(|e: char| e.is_ascii_punctuation());

    "https://".to_string() + cleaned
}

pub fn to_jsonl<T: Serialize>(data: &[T]) -> Result<String, serde_json::Error> {
    data.iter().try_fold(String::new(), |acc, e| {
        serde_json::to_string(e).and_then(|j| Ok(acc + &j + "\n"))
    })
}

pub fn line_progress(
    i: usize,
    start: &mut std::time::Instant,
    interval: usize,
) {
    if i == 0 || i % interval != 0 {
        return;
    }
    println!(
        "processed {i} lines ({:?} since last update)",
        start.elapsed()
    );
    *start = std::time::Instant::now();
}
