use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::Deserialize;
use serde_json;

const CATEGORIES: [&str; 16] = [
    "",
    "Autos & Vehicles",
    "Comedy",
    "Education",
    "Entertainment",
    "Film & Animation",
    "Gaming",
    "Howto & Style",
    // "Movies",
    "Music",
    "News & Politics",
    "Nonprofits & Activism",
    "People & Blogs",
    "Pets & Animals",
    "Science & Technology",
    // "Shows",
    "Sports",
    "Travel & Events",
];

#[derive(Default)]
struct CommenterTable {
    pub data: Vec<[u16; CATEGORIES.len()]>,
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
    // channel_id: String,
    display_id: String,
    // view_count: f32,
    categories: String,
}

fn main() {
    let start = std::time::Instant::now();
    let file = File::open("../dataset/yt_metadata_en.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let categories: HashMap<String, usize> = reader
        .lines()
        .filter_map(|read_line| {
            let line = read_line.unwrap();
            let Ok(entry) = serde_json::from_str::<VideoEntry>(&line) else {
                return None;
            };

            let Some(cat_ind) =
                CATEGORIES.iter().position(|e| e == &entry.categories)
            else {
                return None;
            };

            Some((entry.display_id, cat_ind))
        })
        .collect();

    println!("constructed category map in {:?}", start.elapsed());

    let mut start = std::time::Instant::now();
    let file = File::open("../dataset/youtube_comments.tsv.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let file = File::create("cats.csv.gz").unwrap();
    let mut writer =
        BufWriter::new(GzEncoder::new(file, Compression::default()));
    writeln!(&mut writer, "{}", CATEGORIES.join(",")).unwrap();

    let mut curr_author = 1u64;
    let mut curr_comments = [0u16; CATEGORIES.len()];
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

                table.data.push(curr_comments);
                curr_comments = [0u16; CATEGORIES.len()];

                if table.data.len() >= 50_000_000 {
                    writer.write_all(table.to_string().as_bytes()).unwrap();
                    table.data.clear();
                }
            }

            let Some(&cat_ind) = categories.get(video_id) else {
                return;
            };
            curr_comments[cat_ind as usize] += 1;
        });

    writer.write_all(table.to_string().as_bytes()).unwrap();
}
