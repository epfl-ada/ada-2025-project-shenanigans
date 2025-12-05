use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use petgraph::algo::tarjan_scc;
use petgraph::graph::NodeIndex;
use petgraph::graph::UnGraph;
use serde_json;

fn main() {
    // let file = File::open("sponsoreddomainchannels.json.gz").unwrap();
    let file = File::open("sponsoredchanneldomains.json.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let mut domchannels: HashMap<String, HashSet<String>> =
        serde_json::from_reader(reader).unwrap();

    let file = File::open("../urlclasses/socialmedia.csv").unwrap();
    let reader = BufReader::new(file);
    reader.lines().for_each(|read_line| {
        let line = read_line.unwrap();
        let _ = domchannels.remove(&line);
    });

    let domchannels: Vec<(String, HashSet<String>)> =
        domchannels.drain().collect();

    let mut nodes = HashMap::<String, NodeIndex>::new();
    let mut graph = UnGraph::<String, usize>::default();
    for (i, (d1, cs1)) in
        domchannels.iter().enumerate().take(domchannels.len() - 1)
    {
        for (d2, cs2) in domchannels.iter().skip(i + 1) {
            let weight = cs1.intersection(cs2).count();
            // if weight < 5 {
            if weight < 10 {
                continue;
            }

            let n1 = *nodes
                .entry(d1.clone())
                .or_insert_with(|| graph.add_node(d1.clone()));

            let n2 = *nodes
                .entry(d2.clone())
                .or_insert_with(|| graph.add_node(d2.clone()));

            graph.add_edge(n1, n2, weight);
        }
    }

    println!("{}", graph.node_count());
    let connected = tarjan_scc(&graph);
    let max_con = connected.iter().max_by_key(|e| e.len()).unwrap();
    graph.retain_nodes(|_, e| max_con.contains(&e));
    println!("{}", graph.node_count());

    let file = File::create("channelgraph.json.gz").unwrap();
    let writer = BufWriter::new(GzEncoder::new(file, Compression::default()));
    serde_json::to_writer(writer, &graph).unwrap();
}
