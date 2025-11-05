use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::Deserialize;
use serde_json;

const MIN_COMMENTS: u32 = 2;

#[derive(Default)]
struct CommenterTable {
    pub data: Vec<Vec<u32>>,
}

impl ToString for CommenterTable {
    fn to_string(&self) -> String {
        self.data
            .iter()
            .map(|row| {
                let mut rowstr = row
                    .iter()
                    .map(|e| e.to_string())
                    .fold(String::new(), |acc, e| acc + &e + ",");
                rowstr.pop();
                rowstr
            })
            .fold(String::new(), |acc, e| acc + &e + "\n")
    }
}

#[derive(Deserialize)]
struct VideoEntry {
    channel_id: String,
    display_id: String,
    // view_count: f32,
    // categories: String,
}

fn main() {
    let start = std::time::Instant::now();
    let file = File::open("../dataset/df_channels_en.tsv.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let (channelmap, channels): (HashMap<String, usize>, Vec<String>) = reader
        .lines()
        .skip(1)
        .enumerate()
        .map(|(i, read_line)| {
            let line = read_line.unwrap();
            let split = line.splitn(4, "\t");
            let channel_id = split.skip(2).next().unwrap();
            ((channel_id.to_string(), i), channel_id.to_string())
        })
        .collect();

    println!("read {} channels in {:?}", channels.len(), start.elapsed());

    let start = std::time::Instant::now();
    let file = File::open("../dataset/yt_metadata_en.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let channelinds: HashMap<String, usize> = reader
        .lines()
        .filter_map(|read_line| {
            let line = read_line.unwrap();
            let Ok(entry) = serde_json::from_str::<VideoEntry>(&line) else {
                return None;
            };

            let Some(&chan_ind) = channelmap.get(&entry.channel_id) else {
                return None;
            };

            Some((entry.display_id, chan_ind))
        })
        .collect();

    println!("read {} videos in {:?}", channelinds.len(), start.elapsed());

    let mut start = std::time::Instant::now();
    let file = File::open("../dataset/youtube_comments.tsv.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let file = File::create("chans.csv.gz").unwrap();
    let mut writer =
        BufWriter::new(GzEncoder::new(file, Compression::default()));
    writeln!(&mut writer, "{}", channels.join(",")).unwrap();

    let mut curr_author = 1u64;
    let mut curr_comments = vec![0u32; channels.len()];
    let mut table = CommenterTable {
        data: Vec::with_capacity(1_000_000),
    };
    reader
        .lines()
        .skip(1)
        .enumerate()
        .for_each(|(i, read_line)| {
            if i != 0 && i % 10_000_000 == 0 {
                println!("processed {i} lines in {:?}", start.elapsed());
                start = std::time::Instant::now();
            }

            let line = read_line.unwrap();
            let mut split = line.splitn(3, "\t");
            let line_author = split.next().unwrap().parse().unwrap();
            let video_id = split.next().unwrap();

            if curr_author != line_author {
                curr_author = line_author;

                if curr_comments.iter().sum::<u32>() >= MIN_COMMENTS {
                    table.data.push(curr_comments.clone());
                }
                curr_comments.fill(0u32);

                if table.data.len() >= 1_000_000 {
                    writer.write_all(table.to_string().as_bytes()).unwrap();
                    table.data.clear();
                }
            }

            let Some(&chan_ind) = channelinds.get(video_id) else {
                return;
            };
            curr_comments[chan_ind as usize] += 1;
        });

    writer.write_all(table.to_string().as_bytes()).unwrap();
}
