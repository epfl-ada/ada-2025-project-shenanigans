use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::Deserialize;
use serde_json;

const MIN_COMMENTS: u64 = 2;

#[derive(Default)]
struct CommenterTable {
    pub data: Vec<[u64; 3]>,
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
    categories: String,
}

fn main() {
    let start = std::time::Instant::now();
    let file = File::open("../dataset/num_comments.tsv.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let commented_vids: HashSet<String> = reader
        .lines()
        .skip(1)
        .filter_map(|read_line| {
            let line = read_line.unwrap();
            let mut split = line.splitn(2, "\t");
            let video_id = split.next().unwrap();
            let num_comments: f32 = split.next().unwrap().parse().unwrap();

            if num_comments > 1.0 { Some(video_id.to_owned()) } else { None }
        })
        .collect();

    println!(
        "collected commented videos in {:?} for {} videos",
        start.elapsed(),
        commented_vids.len()
    );

    let start = std::time::Instant::now();
    let file = File::open("../dataset/yt_metadata_en.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let mut edu_channels = HashMap::<String, usize>::new();
    let video_map: HashMap<String, usize> = reader
        .lines()
        .filter_map(|read_line| {
            let line = read_line.unwrap();
            let Ok(entry) = serde_json::from_str::<VideoEntry>(&line) else {
                return None;
            };

            if entry.categories != "Education"
                || !commented_vids.contains(&entry.display_id)
            {
                return None;
            }

            let last_id = edu_channels.len();
            let id = *edu_channels.entry(entry.channel_id).or_insert(last_id);

            Some((entry.display_id, id))
        })
        .collect();

    println!(
        "constructed video map in {:?} for {} channels",
        start.elapsed(),
        edu_channels.len()
    );

    let file = File::create("edu_channelkey.json.gz").unwrap();
    let writer = BufWriter::new(GzEncoder::new(file, Compression::default()));
    let _ = serde_json::to_writer(writer, &edu_channels);

    let file = File::create("education.csv.gz").unwrap();
    let mut writer =
        BufWriter::new(GzEncoder::new(file, Compression::default()));
    writeln!(&mut writer, "author,channel,count").unwrap();

    let mut start = std::time::Instant::now();
    let file = File::open("../dataset/youtube_comments.tsv.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let mut curr_author = 1u64;
    let mut curr_comments = HashMap::<(u64, u64), u64>::new();
    let mut table = CommenterTable {
        data: Vec::with_capacity(50_000_000),
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

                let total = curr_comments.iter().map(|(_, e)| e).sum::<u64>();
                if total >= MIN_COMMENTS {
                    table.data.extend(curr_comments.drain().map(
                        |((author, channel), count)| [author, channel, count],
                    ));
                }
                curr_comments.clear();

                if table.data.len() >= 50_000_000 {
                    writer.write_all(table.to_string().as_bytes()).unwrap();
                    table.data.clear();
                }
            }

            let Some(&chan_ind) = video_map.get(video_id) else {
                return;
            };

            *curr_comments
                .entry((line_author, chan_ind as u64))
                .or_insert(0) += 1;
        });

    writer.write_all(table.to_string().as_bytes()).unwrap();
}
