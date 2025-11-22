use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::Deserialize;
use serde_json;

use utils::{line_progress, make_sponsor_set};

#[derive(Deserialize)]
struct VideoUrls {
    display_id: String,
}

fn main() {
    let start = std::time::Instant::now();
    let file = File::open("../sponsorblock/sponsorTimes.csv").unwrap();
    let reader = BufReader::new(file);
    let sponsored = make_sponsor_set(reader).unwrap();

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
        line_progress(i, &mut start, 10_000_000);

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
