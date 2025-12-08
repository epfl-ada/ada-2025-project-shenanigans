use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};

use chrono::{DateTime, Utc};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};

use utils::make_sponsor_set;

#[derive(Deserialize)]
struct VideoEntry {
    channel_id: String,
    display_id: String,
    #[serde(with = "custom_date_format")]
    upload_date: DateTime<Utc>,
}

mod custom_date_format {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
            .map_err(serde::de::Error::custom)?;
        Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    }
}

#[derive(Default, Serialize)]
struct Videos {
    sponsored: Vec<Video>,
    not_sponsored: Vec<Video>,
}

#[derive(Serialize)]
struct Video {
    display_id: String,
    upload_date: DateTime<Utc>,
}

fn main() {
    let file = File::open("../sponsorblock/sponsorTimes.csv").unwrap();
    let reader = BufReader::new(file);
    let sponsored = make_sponsor_set(reader).unwrap();
    println!("found {} sponsored videos", sponsored.len());

    let process_line =
        |(i, read_line): (usize, Result<String, std::io::Error>)| {
            if i % 10_000_000 == 0 {
                println!("{i}");
            }

            let line = read_line.unwrap();
            serde_json::from_str::<VideoEntry>(&line).ok()
        };

    let mut sponvids = HashMap::<String, Videos>::new();

    let file = File::open("../dataset/yt_metadata_en.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    reader
        .lines()
        .enumerate()
        .filter_map(process_line)
        .for_each(|video_entry| {
            if !sponsored.contains(&video_entry.display_id) {
                return;
            }

            let VideoEntry {
                channel_id,
                display_id,
                upload_date,
            } = video_entry;

            let entry = sponvids.entry(channel_id).or_insert(Videos::default());
            entry.sponsored.push(Video {
                display_id,
                upload_date,
            });
        });
    println!("found {} sponsored channels", sponvids.len());

    let file = File::open("../dataset/yt_metadata_en.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    reader
        .lines()
        .enumerate()
        .filter_map(process_line)
        .for_each(|video_entry| {
            let Some(vids) = sponvids.get_mut(&video_entry.channel_id) else {
                return;
            };

            let VideoEntry {
                channel_id: _,
                display_id,
                upload_date,
            } = video_entry;

            if !sponsored.contains(&display_id) {
                vids.not_sponsored.push(Video {
                    display_id,
                    upload_date,
                });
            }
        });

    let tmp = sponvids
        .values()
        .map(|e| [e.sponsored.len(), e.not_sponsored.len()])
        .fold([0usize; 2], |acc, e| [acc[0] + e[0], acc[1] + e[1]]);
    println!(
        "for {} sponsored videos and {} not sponsored",
        tmp[0], tmp[1]
    );

    sponvids.values_mut().for_each(|e| {
        e.sponsored.sort_by_key(|v| v.upload_date);
        e.not_sponsored.sort_by_key(|v| v.upload_date);
    });

    let file = File::create("sponsoredchannels.json.gz").unwrap();
    let writer = BufWriter::new(GzEncoder::new(file, Compression::default()));
    serde_json::to_writer(writer, &sponvids).unwrap();
}
