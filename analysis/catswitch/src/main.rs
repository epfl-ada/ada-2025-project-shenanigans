use std::{collections::HashMap, fs::File, io::BufReader};

use chrono::{DateTime, Duration, Utc};
use flate2::read::GzDecoder;
use serde::Deserialize;
use serde_json;
use utils::CATEGORIES;

#[derive(Deserialize)]
struct Videos {
    sponsored: Vec<Video>,
    not_sponsored: Vec<Video>,
}

#[derive(Deserialize)]
struct Video {
    display_id: String,
    upload_date: DateTime<Utc>,
}

fn main() {
    let file = File::open("../process/sponsoredchannels.json.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let sponvids: HashMap<String, Videos> =
        serde_json::from_reader(reader).unwrap();

    let file = File::open("../process/catmap.json.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let catmap: HashMap<String, usize> =
        serde_json::from_reader(reader).unwrap();

    let window = Duration::days(6 * 30);
    let transitions: HashMap<(usize, usize), u32> = sponvids
        .into_values()
        .filter_map(|videos| {
            let [ref first_spon, ..] = videos.sponsored[..] else {
                return None;
            };
            let spon_date = first_spon.upload_date;

            let before: Vec<usize> = videos
                .not_sponsored
                .iter()
                .filter(|e| {
                    (spon_date > e.upload_date)
                        && (spon_date - e.upload_date <= window)
                })
                .filter_map(|e| catmap.get(&e.display_id))
                .cloned()
                .collect();

            if before.is_empty() {
                return None;
            }

            let after: Vec<usize> = videos
                .sponsored
                .into_iter()
                .chain(videos.not_sponsored)
                .filter(|e| {
                    (spon_date <= e.upload_date)
                        && (e.upload_date - spon_date <= window)
                })
                .filter_map(|e| catmap.get(&e.display_id))
                .cloned()
                .collect();

            Some((before, after))
        })
        .map(|(b, a)| {
            let mut b_map = HashMap::<usize, u32>::new();
            b.into_iter().for_each(|e| {
                let entry = b_map.entry(e).or_insert(0);
                *entry += 1;
            });
            let (b_max, _) = b_map.drain().max_by_key(|(_, e)| *e).unwrap();

            let mut a_map = HashMap::<usize, u32>::new();
            a.into_iter().for_each(|e| {
                let entry = a_map.entry(e).or_insert(0);
                *entry += 1;
            });
            let (a_max, _) = a_map.drain().max_by_key(|(_, e)| *e).unwrap();

            (b_max, a_max)
        })
        .fold(HashMap::new(), |mut acc, e| {
            *acc.entry(e).or_insert(0) += 1;
            acc
        });

    transitions.into_iter().for_each(|((b, a), c)| {
        println!(
            r#"{{ source: "{}", target: "{}", value: {} }},"#,
            CATEGORIES[b],
            CATEGORIES[a],
            1.0 + (c as f64).log10()
        );
    });
}
