use std::fs::File;
use std::io::{BufReader, BufWriter};

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use petgraph::graph::UnGraph;
use scarlet;
use scarlet::color::RGBColor;
use scarlet::colormap::ColorMap;
use serde::{Deserialize, Serialize};
use serde_json;

const COLOURS: [[u8; 3]; 12] = [
    [166, 206, 227],
    [31, 120, 180],
    [178, 223, 138],
    [51, 160, 44],
    [251, 154, 153],
    [227, 26, 28],
    [253, 191, 111],
    [255, 127, 0],
    [202, 178, 214],
    [106, 61, 154],
    [255, 255, 153],
    [177, 89, 40],
];

#[derive(Debug, Serialize, Deserialize)]
struct SerProps {
    id: usize,
    c: [u8; 3],
    l: [u8; 3],
    x: f64,
    y: f64,
}

fn main() {
    let file = File::open("domaingraph.json.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let graph: UnGraph<String, usize> =
        serde_json::from_reader(reader).unwrap();

    println!(
        "loaded {} nodes and {} edges!",
        graph.node_count(),
        graph.edge_count()
    );

    let file = File::open("layout.json").unwrap();
    let reader = BufReader::new(file);
    let layout: Vec<[f64; 2]> = serde_json::from_reader(reader).unwrap();

    let file = File::open("leiden.json").unwrap();
    let reader = BufReader::new(file);
    let leiden: Vec<Vec<usize>> = serde_json::from_reader(reader).unwrap();
    assert!(leiden.len() <= COLOURS.len());

    let n_nei: Vec<usize> = graph
        .node_indices()
        .map(|e| graph.neighbors_undirected(e).count())
        .collect();
    let max_nei = *n_nei.iter().max().unwrap() as f64;

    let cmap = scarlet::colormap::ListedColorMap::plasma();
    let props: Vec<SerProps> = graph
        .node_indices()
        .zip(n_nei)
        .map(|(e, n)| {
            let conc: RGBColor =
                cmap.transform_single(f64::powf(n as f64 / max_nei, 0.25));
            let lind =
                leiden.iter().position(|l| l.contains(&e.index())).unwrap();
            let leic = COLOURS[lind];
            let id = e.index();
            SerProps {
                id: id,
                c: [conc.int_r(), conc.int_g(), conc.int_b()],
                l: leic,
                x: layout[id][0],
                y: layout[id][1],
            }
        })
        .collect();

    let file = File::create("sitegraphprops.json.gz").unwrap();
    let writer = BufWriter::new(GzEncoder::new(file, Compression::default()));
    let _ = serde_json::to_writer(writer, &props);

    let file = File::create("sitegraph.json.gz").unwrap();
    let writer = BufWriter::new(GzEncoder::new(file, Compression::default()));
    let _ = serde_json::to_writer(writer, &graph);
}
