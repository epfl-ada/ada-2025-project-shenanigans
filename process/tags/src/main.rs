use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::Deserialize;
use serde_json;

#[derive(Deserialize)]
struct VideoEntry {
    // channel_id: String,
    // display_id: String,
    // view_count: f32,
    categories: String,
    tags: String,
}

fn main() {
    let file = File::open("../dataset/yt_metadata_en.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let mut tag_freqs = HashMap::<String, u64>::new();
    reader.lines().for_each(|read_line| {
        let line = read_line.unwrap();
        let Ok(VideoEntry { categories, tags }) =
            serde_json::from_str::<VideoEntry>(&line)
        else {
            return;
        };
        if categories != "Education" {
            return;
        }

        tags.split(",").for_each(|tag| {
            *tag_freqs.entry(tag.to_lowercase()).or_insert(0) += 1;
        });
    });

    let mut sorted: Vec<(String, u64)> = tag_freqs.drain().collect();
    sorted.sort_by_key(|e| e.1);
    sorted.reverse();

    let file = File::create("tag_freqs.csv.gz").unwrap();
    let mut writer =
        BufWriter::new(GzEncoder::new(file, Compression::default()));
    writeln!(writer, "tag,freq").unwrap();
    let buf = sorted
        .drain(..)
        .map(|(k, v)| format!(r#""{}",{}"#, k, v))
        .fold(String::new(), |acc, e| acc + &e + "\n");
    writer.write_all(buf.as_bytes()).unwrap();
}
