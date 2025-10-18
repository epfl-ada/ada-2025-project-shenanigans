use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use regex::Regex;
use serde::Deserialize;
use serde_json;

const MIN_COMMENTS: u64 = 2;

#[derive(Deserialize)]
struct VideoEntry {
    display_id: String,
    categories: String,
    tags: String,
}

#[derive(Default)]
struct CommenterTable {
    pub data: Vec<[u64; 3]>,
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

fn main() {
    let langs = [
        Regex::new(r"\bada\b").unwrap(),
        Regex::new(r"\bapexlang\b").unwrap(),
        Regex::new(r"\bapl\b").unwrap(),
        Regex::new(r"\bassembly\b").unwrap(),
        Regex::new(r"\b(?:bash|shell|powershell|zsh)\b").unwrap(),
        Regex::new(r"\bc\b").unwrap(),
        Regex::new(r"\bc\#(?:\s|/|$)|\bc[\s\-]?sharp\b").unwrap(),
        Regex::new(r"\bc\+\+(?:\s|/|$)|\bc(?:pp|\splus\splus|plusplus)\b")
            .unwrap(),
        Regex::new(r"\bclojure\b").unwrap(),
        Regex::new(r"\bcobol\b").unwrap(),
        Regex::new(r"\bcoffeescript\b").unwrap(),
        Regex::new(r"\bcrystal\b").unwrap(),
        Regex::new(r"\bdart\b").unwrap(),
        Regex::new(r"\bdelphi\b").unwrap(),
        Regex::new(r"\belixir\b").unwrap(),
        Regex::new(r"\berlang\b").unwrap(),
        Regex::new(r"\bf\#(?:\s|/|$)|\bf[\s\-]?sharp\b").unwrap(),
        Regex::new(r"\bfortran\b").unwrap(),
        Regex::new(r"\bgdscript\b").unwrap(),
        Regex::new(r"\bgleam\b").unwrap(),
        Regex::new(r"\bgolang\b").unwrap(),
        Regex::new(r"\bgroovy\b").unwrap(),
        Regex::new(r"\bhaskell\b").unwrap(),
        Regex::new(r"\bhtml5?\b|\bcss3?\b").unwrap(),
        Regex::new(r"\bjava\b").unwrap(),
        Regex::new(r"\bjava(?:[\s\-]?script)\b|\bjs\b").unwrap(),
        Regex::new(r"\bjulia\b").unwrap(),
        Regex::new(r"\bkotlin\b").unwrap(),
        Regex::new(r"\blisp\b").unwrap(),
        Regex::new(r"\blua\b").unwrap(),
        Regex::new(r"\bmatlab\b").unwrap(),
        Regex::new(r"\bmojo\b").unwrap(),
        Regex::new(r"\bnim\b").unwrap(),
        Regex::new(r"\bnode[\s\.\-]?js\b").unwrap(),
        Regex::new(r"\bobjective[\s\-]?c\b").unwrap(),
        Regex::new(r"\bocaml\b").unwrap(),
        Regex::new(r"\bperl\b").unwrap(),
        Regex::new(r"\bphp\b").unwrap(),
        Regex::new(r"\bprolog\b").unwrap(),
        Regex::new(r"\b(?:micro|)python[1-3]?\b").unwrap(),
        Regex::new(r"\br\b").unwrap(),
        Regex::new(r"\braku\b").unwrap(),
        Regex::new(r"\bruby\b").unwrap(),
        Regex::new(r"\brust\b").unwrap(),
        Regex::new(r"\bsas\b").unwrap(),
        Regex::new(r"\bscala\b").unwrap(),
        Regex::new(r"\bsmalltalk\b").unwrap(),
        Regex::new(r"\bsolidity\b").unwrap(),
        Regex::new(r"\bsql\b").unwrap(),
        Regex::new(r"\bswift\b").unwrap(),
        Regex::new(r"\btypescript\b").unwrap(),
        Regex::new(r"\bvisual basic\b|\bvb\.net\b|\b\.net\b|\bvba\b").unwrap(),
        Regex::new(r"\bwebassembly\b").unwrap(),
        Regex::new(r"\bzig\b").unwrap(),
    ];
    // vec!["flow"],
    // vec!["hack"],
    // vec!["zephyr"],

    let start = std::time::Instant::now();
    let file = File::open("../dataset/num_comments.tsv.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let commented_vids: HashSet<String> = reader
        .lines()
        .skip(1)
        .filter_map(|read_line| {
            let line = read_line.unwrap();
            let mut split = line.splitn(2, "\t");
            let video_id = split.next().unwrap();
            let num_comments: f32 = split.next().unwrap().parse().unwrap();

            if num_comments > 1.0 { Some(video_id.to_owned()) } else { None }
        })
        .collect();

    println!(
        "collected commented videos in {:?} for {} videos",
        start.elapsed(),
        commented_vids.len()
    );

    let start = std::time::Instant::now();
    let file = File::open("../dataset/yt_metadata_en.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let video_map: HashMap<String, Vec<usize>> = reader
        .lines()
        .filter_map(|read_line| {
            let line = read_line.unwrap();
            let Ok(VideoEntry {
                display_id,
                categories,
                tags,
            }) = serde_json::from_str::<VideoEntry>(&line)
            else {
                return None;
            };

            if categories != "Education"
                || !commented_vids.contains(&display_id)
            {
                return None;
            }

            let lang: Vec<usize> = tags
                .split(",")
                .filter_map(|tag| {
                    for (i, lang) in langs.iter().enumerate() {
                        if lang.is_match(tag) {
                            return Some(i);
                        }
                    }
                    None
                })
                .collect();

            if lang.is_empty() { None } else { Some((display_id, lang)) }
        })
        .collect();

    println!(
        "constructed video map in {:?} for {} videos",
        start.elapsed(),
        video_map.len()
    );

    let file = File::create("languages.csv.gz").unwrap();
    let mut writer =
        BufWriter::new(GzEncoder::new(file, Compression::default()));
    writeln!(&mut writer, "author,language,count").unwrap();

    let mut start = std::time::Instant::now();
    let file = File::open("../dataset/youtube_comments.tsv.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let mut curr_author = 1u64;
    let mut curr_comments = HashMap::<(u64, u64), u64>::new();
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

                let total = curr_comments.iter().map(|(_, e)| e).sum::<u64>();
                if total >= MIN_COMMENTS {
                    table.data.extend(curr_comments.drain().map(
                        |((author, language), count)| [author, language, count],
                    ));
                }
                curr_comments.clear();

                if table.data.len() >= 50_000_000 {
                    writer.write_all(table.to_string().as_bytes()).unwrap();
                    table.data.clear();
                }
            }

            let Some(langs) = video_map.get(video_id) else {
                return;
            };

            langs.iter().for_each(|lang| {
                *curr_comments
                    .entry((line_author, *lang as u64))
                    .or_insert(0) += 1;
            });
        });

    writer.write_all(table.to_string().as_bytes()).unwrap();
}
