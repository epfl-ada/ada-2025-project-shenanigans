use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::Deserialize;
use serde_json;

const MIN_VIEWS: f32 = 5e5;

#[derive(Deserialize)]
struct VideoEntry {
    channel_id: String,
    display_id: String,
    view_count: f32,
}

fn update_graph(write_ids: Vec<usize>, graph_ltriang: &mut Vec<u32>) {
    write_ids
        .iter()
        .take(write_ids.len() - 1)
        .enumerate()
        .for_each(|(n, &i)| {
            write_ids.iter().skip(n + 1).for_each(|&j| {
                if i >= j {
                    graph_ltriang[i * (i + 1) / 2 + j] += 1;
                } else {
                    graph_ltriang[j * (j + 1) / 2 + i] += 1;
                }
            });
        });
}

fn main() {
    let mut start = std::time::Instant::now();

    let file = File::open("../dataset/yt_metadata_en.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let mut channels = HashMap::<String, usize>::new();
    let video_channel: HashMap<String, usize> = reader
        .lines()
        .filter_map(|read_line| {
            let line = read_line.unwrap();
            let Ok(entry) = serde_json::from_str::<VideoEntry>(&line) else {
                return None;
            };
            if entry.view_count < MIN_VIEWS {
                return None;
            }

            let nb_ids = channels.len();
            let id = *channels.entry(entry.channel_id).or_insert(nb_ids);
            Some((entry.display_id, id))
        })
        .collect();

    let dim = channels.len();

    let file = File::create("channelkey.json.gz").unwrap();
    let writer = BufWriter::new(GzEncoder::new(file, Compression::default()));
    let _ = serde_json::to_writer(writer, &channels);

    let mut graph_ltriang = vec![0u32; dim * (dim + 1) / 2];

    println!("loaded video-channel map in {:?}", start.elapsed());

    let file = File::open("../dataset/youtube_comments.tsv.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    start = std::time::Instant::now();
    let mut curr_author = 1u64;
    let mut video_ids = HashSet::<usize>::new();
    reader
        .lines()
        .skip(1)
        .enumerate()
        .for_each(|(i, read_line)| {
            if i != 0 && i % 5_000_000 == 0 {
                println!("processed {i} lines in {:?}", start.elapsed());
                start = std::time::Instant::now();

                if i % 1_000_000_000 == 0 {
                    let file = File::create("graph_ltriang.json.gz").unwrap();
                    let writer = BufWriter::new(GzEncoder::new(
                        file,
                        Compression::default(),
                    ));
                    let _ = serde_json::to_writer(writer, &graph_ltriang);
                }
            }

            let line = read_line.unwrap();
            let mut split = line.splitn(3, "\t");
            let line_author = split.next().unwrap().parse().unwrap();
            let video_id = split.next().unwrap();

            if curr_author != line_author {
                curr_author = line_author;
                update_graph(video_ids.drain().collect(), &mut graph_ltriang);
            }

            if let Some(channel_id) = video_channel.get(video_id) {
                video_ids.insert(*channel_id);
            }
        });

    let file = File::create("graph_ltriang.json.gz").unwrap();
    let writer = BufWriter::new(GzEncoder::new(file, Compression::default()));
    let _ = serde_json::to_writer(writer, &graph_ltriang);
}
