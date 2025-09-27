use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::sync::{Arc, Mutex};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use petgraph::dot::Dot;
use petgraph::graph::{NodeIndex, UnGraph};
use rayon;
use rayon::prelude::*;
use serde::Deserialize;
use serde_json;

const MIN_VIEWS: f32 = 1e5;

#[derive(Deserialize)]
struct Entry {
    channel_id: String,
    display_id: String,
    view_count: f32,
}

fn parse_meta(
    read_line: Result<String, std::io::Error>,
) -> Option<(String, String)> {
    let line = read_line.unwrap();
    let Ok(entry) = serde_json::from_str::<Entry>(&line) else {
        return None;
    };
    if entry.view_count > MIN_VIEWS {
        Some((entry.display_id, entry.channel_id))
    } else {
        None
    }
}

fn update_graph(
    write_ids: Vec<String>,
    graph: Arc<Mutex<UnGraph<String, u32>>>,
    node_ids: Arc<Mutex<HashMap<String, NodeIndex>>>,
) {
    write_ids
        .iter()
        .take(write_ids.len() - 1)
        .enumerate()
        .for_each(|(i, id1)| {
            let a = *node_ids
                .lock()
                .unwrap()
                .entry(id1.to_string())
                .or_insert(graph.lock().unwrap().add_node(id1.to_string()));
            write_ids.iter().skip(i + 1).for_each(|id2| {
                let b =
                    *node_ids.lock().unwrap().entry(id2.to_string()).or_insert(
                        graph.lock().unwrap().add_node(id2.to_string()),
                    );

                let mut graph = graph.lock().unwrap();
                if let Some(eid) = graph.find_edge(a, b) {
                    *graph.edge_weight_mut(eid).unwrap() += 1;
                } else {
                    graph.add_edge(a, b, 0);
                }
            });
        });
}

fn main() {
    let mut start = std::time::Instant::now();
    let file = File::open("../dataset/yt_metadata_en.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let video_channel: HashMap<String, String> =
        reader.lines().par_bridge().filter_map(parse_meta).collect();
    println!("loaded video-channel map in {:?}", start.elapsed());

    let file = File::open("../dataset/youtube_comments.tsv.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let node_ids = Arc::new(Mutex::new(HashMap::<String, NodeIndex>::new()));
    let graph = Arc::new(Mutex::new(UnGraph::<String, u32>::default()));

    start = std::time::Instant::now();
    rayon::scope(|scope| {
        let mut curr_author = 1u64;
        let mut video_ids = HashSet::<String>::new();
        reader
            .lines()
            .skip(1)
            .enumerate()
            .for_each(|(i, read_line)| {
                if i != 0 && i % 1_000_000 == 0 {
                    println!("processed {i} lines in {:?}", start.elapsed());
                    start = std::time::Instant::now();

                    let file = File::create("graph.dot.gz").unwrap();
                    let mut writer = BufWriter::new(GzEncoder::new(
                        file,
                        Compression::default(),
                    ));
                    let graph = graph.lock().unwrap();
                    let dot = Dot::new(&*graph);
                    let _ = writer.write_all(dot.to_string().as_bytes());
                }

                let line = read_line.unwrap();
                let mut split = line.splitn(3, "\t");
                let line_author = split.next().unwrap().parse().unwrap();
                let video_id = split.next().unwrap();

                if curr_author != line_author {
                    curr_author = line_author;

                    let node_ids = Arc::clone(&node_ids);
                    let graph = Arc::clone(&graph);
                    let video_ids = video_ids.drain().collect();

                    scope.spawn(move |_| {
                        update_graph(video_ids, graph, node_ids);
                    });
                }

                if let Some(channel_id) = video_channel.get(video_id) {
                    video_ids.insert(channel_id.to_owned());
                }
            });
    });

    let file = File::create("graph.dot.gz").unwrap();
    let mut writer =
        BufWriter::new(GzEncoder::new(file, Compression::default()));
    let graph = graph.lock().unwrap();
    let dot = Dot::new(&*graph);
    let _ = writer.write_all(dot.to_string().as_bytes());
}
