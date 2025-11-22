use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
struct VideoUrls {
    display_id: String,
    urls: Vec<String>,
}

fn clean_url(url: &str) -> String {
    let cleaned = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches(|e: char| e.is_ascii_punctuation());

    "https://".to_string() + cleaned
}

fn main() {
    let process_line = |read_line: Result<String, std::io::Error>| {
        let line = read_line.unwrap();
        let (short, unshort) = line.split_once(",").unwrap();
        if unshort.starts_with("http://") || unshort.starts_with("https://") {
            Some((clean_url(short), unshort.to_string()))
        } else {
            println!("{short} --> {unshort}");
            None
        }
    };

    let patterns = [
        "bit.ly",
        "geni.us",
        "goo.gl",
        "is.gd",
        "j.mp",
        "ow.ly",
        "tiny.cc",
        "tinyurl.com",
    ];

    let maps: Vec<HashMap<String, String>> = patterns
        .iter()
        .map(|p| {
            let fname = format!("unshorted/{}.csv.gz", p.replace('.', "_"));
            let file = File::open(fname).unwrap();
            let reader = BufReader::new(GzDecoder::new(file));
            reader.lines().filter_map(process_line).collect()
        })
        .collect();

    let file = File::open("sponsoredurls.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let mut start = std::time::Instant::now();
    let mut videourls: Vec<VideoUrls> = reader
        .lines()
        .enumerate()
        .map(|(i, read_line)| {
            if i != 0 && i % 10_000 == 0 {
                println!("processed {i} lines in {:?}", start.elapsed());
                start = std::time::Instant::now();
            }

            let line = read_line.unwrap();
            let mut vidurls: VideoUrls = serde_json::from_str(&line).unwrap();
            vidurls.urls.iter_mut().for_each(|url| {
                let Some(i) = patterns.iter().position(|p| url.contains(p))
                else {
                    return;
                };
                let clean = clean_url(&url);
                if let Some(unshort) = maps[i].get(&clean) {
                    *url = unshort.to_owned();
                }
            });

            vidurls
        })
        .collect();

    let file = File::create("sponsoredurls_unshortened.jsonl.gz").unwrap();
    let mut writer =
        BufWriter::new(GzEncoder::new(file, Compression::default()));
    let buf = videourls
        .drain(..)
        .map(|e| serde_json::to_string(&e).unwrap())
        .fold(String::new(), |acc, e| acc + &e + "\n");
    writer.write_all(buf.as_bytes()).unwrap();
}
