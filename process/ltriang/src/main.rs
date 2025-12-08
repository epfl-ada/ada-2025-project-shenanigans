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

    println!("{}", domchannels.len());

    let file =
        File::open("sponsoreddomains_unshortened_v4_full_data.csv").unwrap();
    let reader = BufReader::new(file);
    let tmp: HashSet<String> = reader
        .lines()
        .skip(1)
        .filter_map(|read_line| {
            let line = read_line.unwrap();
            let (url, yn) = line.split_once(",,").unwrap();
            let url = url.to_lowercase();
            let yn = yn.to_lowercase();
            match yn.as_str() {
                "yes" => Some(url),
                "no" => None,
                _ => unreachable!(),
            }
        })
        .collect();
    domchannels
        .values_mut()
        .for_each(|e| e.retain(|d| tmp.contains(d)));

    // let file =
    //     File::open("sponsoreddomains_unshortened_v4_full_data.csv").unwrap();
    // let reader = BufReader::new(file);
    // let tmp: HashSet<String> = reader
    //     .lines()
    //     .skip(1)
    //     .filter_map(|read_line| {
    //         let line = read_line.unwrap();
    //         let (url, yn) = line.split_once(",,").unwrap();
    //         let url = url.to_lowercase();
    //         let yn = yn.to_lowercase();
    //         match yn.as_str() {
    //             "yes" => Some(url),
    //             "no" => None,
    //             _ => unreachable!(),
    //         }
    //     })
    //     .collect();
    // domchannels.retain(|k, _| tmp.contains(k));

    let file = File::open("../dataset/df_channels_en.tsv.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let names: HashMap<String, String> = reader
        .lines()
        .map(|read_line| {
            let line = read_line.unwrap();
            let mut splits = line.split('\t');

            let chan = splits.nth(2).unwrap().to_string();
            let name = splits.next().unwrap().to_string();

            (chan, name)
        })
        .collect();

    let domchannels: Vec<(String, HashSet<String>)> =
        domchannels.drain().collect();

    let mut nodes = HashMap::<String, NodeIndex>::new();
    let mut graph = UnGraph::<(String, String), usize>::default();
    for (i, (d1, cs1)) in
        domchannels.iter().enumerate().take(domchannels.len() - 1)
    {
        for (d2, cs2) in domchannels.iter().skip(i + 1) {
            let weight = cs1.intersection(cs2).count();
            if weight < 3 {
                continue;
            }
            // if weight == 0 {
            //     continue;
            // }

            let name1 = names.get(d1).unwrap();
            let name2 = names.get(d2).unwrap();

            let n1 = *nodes
                .entry(d1.clone())
                .or_insert_with(|| graph.add_node((d1.clone(), name1.clone())));

            let n2 = *nodes
                .entry(d2.clone())
                .or_insert_with(|| graph.add_node((d2.clone(), name2.clone())));

            graph.add_edge(n1, n2, weight);
        }
    }

    println!("{}", graph.node_count());
    let connected = tarjan_scc(&graph);
    let max_con = connected.iter().max_by_key(|e| e.len()).unwrap();
    graph.retain_nodes(|_, e| max_con.contains(&e));
    println!("{}", graph.node_count());

    // let file = File::create("domaingraph.json.gz").unwrap();
    let file = File::create("channelgraph.json.gz").unwrap();
    let writer = BufWriter::new(GzEncoder::new(file, Compression::default()));
    serde_json::to_writer(writer, &graph).unwrap();
}
