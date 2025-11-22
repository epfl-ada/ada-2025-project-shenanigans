use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::Deserialize;
use serde_json;

#[derive(Deserialize)]
struct VideoUrls {
    display_id: String,
}

fn main() {
    let start = std::time::Instant::now();
    let file = File::open("../sponsorblock/sponsorTimes.csv").unwrap();
    let reader = BufReader::new(file);

    let sponsored: HashSet<String> = reader
        .lines()
        .skip(1)
        .filter_map(|read_line| {
            let line = read_line.unwrap();
            if line.starts_with(r#"","#) || line.starts_with(r#""""#) {
                return None;
            }
            if line.starts_with('"') {
                return Some(line.trim_start_matches('"').to_string());
            }
            if let Some((video_id, _)) = line.split_once(",") {
                Some(video_id.to_string())
            } else {
                None
            }
        })
        .collect();

    println!(
        "constructed sponsor set in {:?} for {} videos",
        start.elapsed(),
        sponsored.len()
    );

    let file = File::open("urls.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let mut n_tot = 0u32;
    let mut n_spon = 0u32;

    let mut kept_lines = String::new();
    let mut start = std::time::Instant::now();
    reader.lines().enumerate().for_each(|(i, read_line)| {
        if i != 0 && i % 10_000_000 == 0 {
            println!("processed {i} lines in {:?}", start.elapsed());
            start = std::time::Instant::now();
        }

        let line = read_line.unwrap();
        let VideoUrls { display_id } = serde_json::from_str(&line).unwrap();

        n_tot += 1;

        if !sponsored.contains(&display_id) {
            return;
        }

        kept_lines += &line;
        kept_lines += "\n";
        n_spon += 1;
    });

    println!("found {n_spon} sponsored videos out of {n_tot}");

    let file = File::create("sponsoredurls.jsonl.gz").unwrap();
    let mut writer =
        BufWriter::new(GzEncoder::new(file, Compression::default()));
    writer.write_all(kept_lines.as_bytes()).unwrap();
}
