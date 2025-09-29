use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read};

use fdg::{
    Force, ForceGraph,
    fruchterman_reingold::FruchtermanReingoldParallel,
    petgraph::stable_graph::{NodeIndex, StableGraph},
};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
struct SerNode {
    pub id: usize,
    pub uid: String,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SerGraph {
    pub nodes: Vec<SerNode>,
    pub edges: Vec<[usize; 2]>,
}

fn main() {
    let file = File::open("graph_ltriang.json.gz").unwrap();
    let mut reader = BufReader::new(GzDecoder::new(file));

    let mut i = 0usize;
    let mut j = 0usize;
    let mut inserted = HashMap::<usize, NodeIndex>::new();
    let mut matgraph = StableGraph::<usize, ()>::new();

    let mut dummy = [0; 1];
    let _ = reader.read_exact(&mut dummy);
    let mut buf = vec![];
    while let Ok(n) = reader.read_until(b',', &mut buf)
        && n != 0
    {
        buf.pop();
        let s = std::str::from_utf8(&buf).unwrap();
        let e: u32 = s.parse().unwrap();

        if e >= 50 {
            let a = *inserted.entry(i).or_insert_with(|| matgraph.add_node(i));
            let b = *inserted.entry(j).or_insert_with(|| matgraph.add_node(j));
            matgraph.add_edge(a, b, ());
        }

        if i == j {
            i += 1;
            j = 0;
        } else {
            j += 1;
        }

        buf.clear();
    }
    let inserted: HashMap<NodeIndex, usize> =
        inserted.drain().map(|(k, v)| (v, k)).collect();

    println!(
        "loaded {} nodes and {} edges!",
        matgraph.node_count(),
        matgraph.edge_count()
    );

    let mut forcegraph: ForceGraph<f32, 2, usize, ()> =
        fdg::init_force_graph_uniform(matgraph, 10.0);
    FruchtermanReingoldParallel::default().apply_many(&mut forcegraph, 2);
    fdg::simple::Center::default().apply(&mut forcegraph);

    println!("finished layout!");

    let file = File::open("channelkey.json.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let mut channels: HashMap<String, usize> =
        serde_json::from_reader(reader).unwrap();
    let channels: HashMap<usize, String> =
        channels.drain().map(|(k, v)| (v, k)).collect();

    println!("loaded channel map!");

    let nodes = forcegraph
        .node_weights()
        .map(|(id, c)| SerNode {
            id: *id,
            uid: channels.get(id).unwrap().to_string(),
            x: c.x,
            y: c.y,
        })
        .collect();

    let edges = forcegraph
        .edge_indices()
        .map(|e| {
            let (a, b) = forcegraph.edge_endpoints(e).unwrap();
            [*inserted.get(&a).unwrap(), *inserted.get(&b).unwrap()]
        })
        .collect();

    let out = SerGraph { nodes, edges };

    let file = File::create("bigraph.json.gz").unwrap();
    let writer = BufWriter::new(GzEncoder::new(file, Compression::default()));
    let _ = serde_json::to_writer(writer, &out);
}
