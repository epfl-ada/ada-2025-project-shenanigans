use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter},
};

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::Deserialize;
use serde_json;

use utils::CATEGORIES;

#[derive(Deserialize)]
struct VideoEntry {
    display_id: String,
    categories: String,
}

fn main() {
    let file = File::open("../dataset/yt_metadata_en.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let catmap: HashMap<String, usize> = reader
        .lines()
        .filter_map(|read_line| {
            let line = read_line.unwrap();
            let VideoEntry {
                display_id,
                categories,
            } = serde_json::from_str(&line).unwrap();

            let Some(cat) =
                CATEGORIES.into_iter().position(|e| e == categories)
            else {
                return None;
            };

            Some((display_id.to_string(), cat))
        })
        .collect();

    let file = File::create("catmap.json.gz").unwrap();
    let writer = BufWriter::new(GzEncoder::new(file, Compression::default()));
    serde_json::to_writer(writer, &catmap).unwrap();
}
