use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use flate2::read::GzDecoder;
use petgraph::matrix_graph::UnMatrix;
use serde_json;

fn main() {
    // let file = File::open("channelkey.json.gz").unwrap();
    // let reader = BufReader::new(GzDecoder::new(file));
    // let mut channels: HashMap<String, usize> =
    //     serde_json::from_reader(reader).unwrap();
    // let channels: HashMap<usize, String> =
    //     channels.drain().map(|(k, v)| (v, k)).collect();

    let file = File::open("graph_ltriang.json.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let graph: Vec<u32> = serde_json::from_reader(reader).unwrap();

    let mut i = 0u16;
    let mut j = 0u16;
    let matgraph =
        UnMatrix::<(), ()>::from_edges(graph.into_iter().filter_map(|e| {
            if e < 50 {
                return None;
            }
            let res = (i, j);
            if i == j {
                i += 1;
                j = 0;
            } else {
                j += 1;
            }
            Some(res)
        }));

    println!(
        "loaded {} nodes and {} edges",
        matgraph.node_count(),
        matgraph.edge_count()
    );

    // let mut out = SerGraph::new();
    // channels.iter().for_each(|(k, v)| {
    //     out.nodes.append();
    // });
}
