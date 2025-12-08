use std::{
    collections::{hash_map::HashMap, hash_set::HashSet},
    fs::File,
    io::{BufRead, BufReader, BufWriter},
};

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use regex::Regex;

use serde::Deserialize;
use utils::make_sponsor_set;

#[derive(Deserialize)]
struct VideoEntry {
    channel_id: String,
    display_id: String,
}

#[derive(Deserialize)]
struct VideoUrls {
    display_id: String,
    urls: Vec<String>,
}

fn main() {
    let file = File::open("../sponsorblock/sponsorTimes.csv").unwrap();
    let reader = BufReader::new(file);
    let sponsored = make_sponsor_set(reader).unwrap();

    let file = File::open("../dataset/yt_metadata_en.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let vidchan: HashMap<String, String> = reader
        .lines()
        .filter_map(|read_line| {
            let line = read_line.unwrap();

            let Ok(VideoEntry {
                channel_id,
                display_id,
            }) = serde_json::from_str(&line)
            else {
                return None;
            };

            if sponsored.contains(&display_id) {
                Some((display_id, channel_id))
            } else {
                None
            }
        })
        .collect();

    let domain_pattern = Regex::new(
        r#"(?x)
        (?:www\.)?                      # www
        ([\-[:word:]@:%\+~\#=\.]{1,256} # domains and subdomains
        \.
        [[:alnum:]()]{1,6})             # top-level domain
        "#,
    )
    .unwrap();

    let mut channeldoms = HashMap::<String, HashSet<String>>::new();
    let file = File::open("sponsoredurls_unshortened.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    reader.lines().for_each(|read_line| {
        let line = read_line.unwrap();

        let VideoUrls { display_id, urls } =
            serde_json::from_str(&line).unwrap();

        let channel_id = vidchan.get(&display_id).unwrap();

        let entry = channeldoms.entry(channel_id.clone()).or_default();
        entry.extend(
            urls.into_iter().map(|e| {
                domain_pattern.captures(&e).unwrap()[1].to_lowercase()
            }),
        );
    });

    let file = File::create("sponsoredchanneldomains.json.gz").unwrap();
    let writer = BufWriter::new(GzEncoder::new(file, Compression::default()));
    serde_json::to_writer(writer, &channeldoms).unwrap();

    let mut domchannels = HashMap::<String, HashSet<String>>::new();
    let file = File::open("sponsoredurls_unshortened.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    reader.lines().for_each(|read_line| {
        let line = read_line.unwrap();

        let VideoUrls { display_id, urls } =
            serde_json::from_str(&line).unwrap();

        let channel_id = vidchan.get(&display_id).unwrap();

        urls.into_iter().for_each(|e| {
            let domain = domain_pattern.captures(&e).unwrap()[1].to_lowercase();
            let entry = domchannels.entry(domain).or_default();
            entry.insert(channel_id.clone());
        });
    });

    let file = File::create("sponsoreddomainchannels.json.gz").unwrap();
    let writer = BufWriter::new(GzEncoder::new(file, Compression::default()));
    serde_json::to_writer(writer, &domchannels).unwrap();
}
