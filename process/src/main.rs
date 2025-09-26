use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use flate2::Compression;
use flate2::{read::GzDecoder, write::GzEncoder};
use serde::Deserialize;
use serde_json;

#[derive(Deserialize)]
struct Entry {
    channel_id: String,
    display_id: String,
}

fn main() {
    let file = File::open("../dataset/yt_metadata_en.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let video_channel: HashMap<String, String> = reader
        .lines()
        .map(|read_line| {
            let line = read_line.unwrap();
            let entry = serde_json::from_str::<Entry>(&line).unwrap();
            (entry.display_id, entry.channel_id)
        })
        .collect();

    let file = File::open("../dataset/youtube_comments.tsv.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let file = File::create("graph.tsv.gz").unwrap();
    let mut writer =
        BufWriter::new(GzEncoder::new(file, Compression::default()));

    let mut curr_author = 1u64;
    let mut video_ids = HashSet::new();

    reader
        .lines()
        .skip(1)
        .take(1_000_000_000)
        .for_each(|read_line| {
            let line = read_line.unwrap();
            let mut split = line.splitn(3, "\t");
            let line_author =
                u64::from_str_radix(split.next().unwrap(), 10).unwrap();
            let video_id = split.next().unwrap();

            if curr_author == line_author {
                video_ids
                    .insert(video_channel.get(video_id).unwrap().to_owned());
                return;
            }

            curr_author = line_author;
            let write_ids: Vec<&String> = video_ids.iter().collect();
            let mut out_buf = String::new();
            write_ids
                .iter()
                .take(write_ids.len() - 1)
                .enumerate()
                .for_each(|(i, id1)| {
                    write_ids.iter().skip(i + 1).for_each(|id2| {
                        out_buf += id1;
                        out_buf += " ";
                        out_buf += id2;
                        out_buf += "\n";
                    })
                });
            let _ = writer.write_all(out_buf.as_bytes());
        });
}
