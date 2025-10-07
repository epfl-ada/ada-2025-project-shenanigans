use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader, BufWriter, Read};
use std::process::exit;

use fdg::{
    Force, ForceGraph,
    fruchterman_reingold::FruchtermanReingoldParallel,
    petgraph::dot::Dot,
    petgraph::stable_graph::{NodeIndex, StableGraph},
};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use scarlet;
use scarlet::color::RGBColor;
use scarlet::colormap::ColorMap;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
struct SerNode {
    pub id: u32,
    pub uid: String,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SerGraph {
    pub nodes: Vec<SerNode>,
    pub edges: Vec<(u32, u32)>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SerCol {
    pub id: u32,
    pub c: [u8; 3],
}

fn main() {
    let file = File::open("graph_ltriang.json.gz").unwrap();
    let mut reader = BufReader::new(GzDecoder::new(file));

    let mut i = 0u32;
    let mut j = 0u32;
    let mut inserted = HashMap::<u32, NodeIndex>::new();
    let mut matgraph = StableGraph::<u32, ()>::new();

    let mut dummy = [0; 1];
    let _ = reader.read_exact(&mut dummy);
    let mut buf = vec![];
    while let Ok(n) = reader.read_until(b',', &mut buf)
        && n != 0
    {
        buf.pop();
        let s = std::str::from_utf8(&buf).unwrap();
        let e: u32 = s.parse().unwrap();

        if e >= 2_500 {
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

    let file = File::create("bigraph.dot.gz").unwrap();
    let mut writer =
        BufWriter::new(GzEncoder::new(file, Compression::default()));
    let _ = write!(&mut writer, "{:?}", Dot::new(&matgraph));

    exit(0);

    println!(
        "loaded {} nodes and {} edges!",
        matgraph.node_count(),
        matgraph.edge_count()
    );

    let mut forcegraph: ForceGraph<f32, 2, u32, ()> =
        fdg::init_force_graph_uniform(matgraph, 10.0);
    FruchtermanReingoldParallel::default().apply_many(&mut forcegraph, 2);
    fdg::simple::Center::default().apply(&mut forcegraph);

    println!("finished layout!");

    let file = File::open("channelkey.json.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let mut channels: HashMap<String, u32> =
        serde_json::from_reader(reader).unwrap();
    let channels: HashMap<u32, String> =
        channels.drain().map(|(k, v)| (v, k)).collect();

    println!("loaded channel map!");

    let mut nodes: Vec<SerNode> = forcegraph
        .node_weights()
        .map(|(id, c)| SerNode {
            id: inserted.get(id).unwrap().index() as u32,
            uid: channels.get(id).unwrap().to_string(),
            x: c.x,
            y: c.y,
        })
        .collect();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));

    let n_nei: Vec<usize> = nodes
        .iter()
        .map(|e| forcegraph.neighbors_undirected(e.id.into()).count())
        .collect();
    let max_nei = *n_nei.iter().max().unwrap() as f64;

    let cmap = scarlet::colormap::ListedColorMap::plasma();
    let colours: Vec<SerCol> = nodes
        .iter()
        .zip(n_nei)
        .map(|(e, n)| {
            let colour: RGBColor =
                cmap.transform_single(f64::powf(n as f64 / max_nei, 0.25));
            SerCol {
                id: e.id,
                c: [colour.int_r(), colour.int_g(), colour.int_b()],
            }
        })
        .collect();

    let file = File::create("colours.json.gz").unwrap();
    let writer = BufWriter::new(GzEncoder::new(file, Compression::default()));
    let _ = serde_json::to_writer(writer, &colours);

    let edges: Vec<(u32, u32)> = forcegraph
        .edge_indices()
        .map(|e| {
            let (a, b) = forcegraph.edge_endpoints(e).unwrap();
            (a.index() as u32, b.index() as u32)
        })
        .collect();

    let out = SerGraph { nodes, edges };

    let file = File::create("bigraph.json.gz").unwrap();
    let writer = BufWriter::new(GzEncoder::new(file, Compression::default()));
    let _ = serde_json::to_writer(writer, &out);
}
