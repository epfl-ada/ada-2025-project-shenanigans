use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Deserialize)]
struct VideoEntry {
    display_id: String,
    description: String,
}

#[derive(Serialize)]
struct VideoUrls {
    display_id: String,
    urls: Vec<String>,
}

fn main() {
    let url_pattern = Regex::new(
        r#"(?x)
        (?:https?:\/\/|www\.|https?:\/\/www\.) # https and www
        [\-[:word:]@:%\+~\#=\.]{1,256}         # domains and subdomains
        \.
        [[:alnum:]()]{1,6}                     # top-level domain
        \b
        [\-[:word:]@:%\+~\#=\.()?&\/]*         # page
        "#,
    )
    .unwrap();

    let file = File::create("urls.jsonl.gz").unwrap();
    let mut writer =
        BufWriter::new(GzEncoder::new(file, Compression::default()));

    let file = File::open("../dataset/yt_metadata_en.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let mut start = std::time::Instant::now();
    let mut videourls = Vec::<VideoUrls>::with_capacity(10_000_000);

    reader.lines().enumerate().for_each(|(i, read_line)| {
        if i != 0 && i % 10_000_000 == 0 {
            println!("processed {i} lines in {:?}", start.elapsed());
            start = std::time::Instant::now();
        }

        let line = read_line.unwrap();
        let Ok(VideoEntry {
            display_id,
            description,
        }) = serde_json::from_str::<VideoEntry>(&line)
        else {
            return;
        };

        let urls: Vec<String> = url_pattern
            .find_iter(&description)
            .map(|e| e.as_str().to_owned())
            .collect();

        if urls.is_empty() {
            return;
        }

        videourls.push(VideoUrls { display_id, urls });

        if videourls.len() >= 10_000_000 {
            let buf = videourls
                .drain(..)
                .map(|e| serde_json::to_string(&e).unwrap())
                .fold(String::new(), |acc, e| acc + &e + "\n");
            writer.write_all(buf.as_bytes()).unwrap();
        }
    });

    let buf = videourls
        .drain(..)
        .map(|e| serde_json::to_string(&e).unwrap())
        .fold(String::new(), |acc, e| acc + &e + "\n");
    writer.write_all(buf.as_bytes()).unwrap();
}
