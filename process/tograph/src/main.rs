use std::collections::hash_map::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use petgraph::graph::UnGraph;
use scarlet;
use scarlet::color::RGBColor;
use scarlet::colormap::ColorMap;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Clone, Copy, Debug)]
enum UrlCat {
    Content,
    Entertainement,
    Gaming,
    Legal,
    Monetisation,
    News,
    Social,
    None,
}

const LEIDEN_COLOURS: [[u8; 3]; 14] = [
    [4, 165, 229],
    [223, 142, 29],
    [136, 57, 239],
    [220, 138, 120],
    [234, 118, 203],
    [210, 15, 57],
    [230, 69, 83],
    [254, 100, 11],
    [64, 160, 43],
    [221, 120, 120],
    [32, 159, 181],
    [23, 146, 153],
    [30, 102, 245],
    [114, 135, 253],
];

#[derive(Debug, Serialize, Deserialize)]
struct SerProps {
    id: usize,
    s: f64,
    c: [u8; 3],
    l: [u8; 3],
    a: [u8; 3],
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
    assert!(leiden.len() <= LEIDEN_COLOURS.len());

    let cats = HashMap::from([
        ("content/music", UrlCat::Content),
        ("content/portfolios", UrlCat::Content),
        ("content/video", UrlCat::Content),
        ("entertainement", UrlCat::Entertainement),
        ("gaming", UrlCat::Gaming),
        ("legal/reference", UrlCat::Legal),
        ("monetization/affiliates", UrlCat::Monetisation),
        ("monetization/courses", UrlCat::Monetisation),
        ("monetization/direct", UrlCat::Monetisation),
        ("monetization/merch", UrlCat::Monetisation),
        ("monetization/sponsors", UrlCat::Monetisation),
        ("monetization/stores", UrlCat::Monetisation),
        ("news", UrlCat::News),
        ("non-existing site", UrlCat::None),
        ("other", UrlCat::None),
        ("social_media", UrlCat::Social),
        ("unsure", UrlCat::None),
        ("url_shorteners", UrlCat::None),
        ("utils", UrlCat::None),
    ]);
    let file =
        File::open("../urlclasses/sponsoreddomains_unshortened_checkpoint.csv")
            .unwrap();
    let reader = BufReader::new(file);
    let cat_map: HashMap<String, UrlCat> = reader
        .lines()
        .skip(1)
        .map(|read_line| {
            let line = read_line.unwrap();
            let mut splits = line.split(',');

            let dom = splits.next().unwrap();

            let cat_str = splits.nth(1).unwrap();
            let cat = cats.get(cat_str).unwrap();

            (dom.to_string(), *cat)
        })
        .collect();

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
            let s = f64::powf(n as f64 / max_nei, 0.25);

            let conc: RGBColor = cmap.transform_single(s);

            let lind =
                leiden.iter().position(|l| l.contains(&e.index())).unwrap();
            let leic = LEIDEN_COLOURS[lind];

            let dom = graph.node_weight(e).unwrap();
            let cat = match cat_map.get(dom) {
                Some(e) => e,
                None => {
                    println!("{dom}");
                    &UrlCat::None
                }
            };
            let catc = match cat {
                UrlCat::Content => [220, 138, 120],
                UrlCat::Entertainement => [210, 15, 57],
                UrlCat::Gaming => [234, 118, 203],
                UrlCat::Legal => [114, 135, 253],
                UrlCat::Monetisation => [64, 160, 43],
                UrlCat::News => [30, 102, 245],
                UrlCat::Social => [254, 100, 11],
                UrlCat::None => [76, 79, 105],
            };

            let id = e.index();
            SerProps {
                id: id,
                s: s,
                c: [conc.int_r(), conc.int_g(), conc.int_b()],
                l: leic,
                a: catc,
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
