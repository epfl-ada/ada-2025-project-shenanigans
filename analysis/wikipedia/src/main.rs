use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use charming::{
    Chart, HtmlRenderer,
    element::{Emphasis, ItemStyle, Tooltip, Trigger},
    series::{Pie, PieRoseType},
    theme::Theme,
};
use flate2::read::GzDecoder;
use regex::Regex;
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

#[derive(Deserialize)]
struct VideoEntry {
    display_id: String,
    categories: String,
}

#[derive(Deserialize)]
struct VideoUrls {
    display_id: String,
    urls: Vec<String>,
}

fn main() {
    let start = std::time::Instant::now();
    let file = File::open("../dataset/yt_metadata_en.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let catmap: HashMap<String, usize> = reader
        .lines()
        .filter_map(|read_line| {
            let line = read_line.unwrap();
            let Ok(VideoEntry {
                display_id,
                categories,
            }) = serde_json::from_str::<VideoEntry>(&line)
            else {
                return None;
            };

            if let Some(catind) =
                CATEGORIES.iter().position(|e| e == &categories)
            {
                Some((display_id, catind))
            } else {
                None
            }
        })
        .collect();

    println!(
        "constructed category map in {:?} for {} videos",
        start.elapsed(),
        catmap.len()
    );

    let wiki_pattern =
        Regex::new(r".*wikipedia\.org/wiki/([\-[:word:]@:%\+~\#=\.()?&\/]*)")
            .unwrap();

    let file = File::open("../process/urls.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let mut start = std::time::Instant::now();
    let mut domain_freqs =
        vec![HashMap::<String, u64>::new(); CATEGORIES.len()];
    reader
        .lines()
        .take(1_000_000)
        .enumerate()
        .for_each(|(i, read_line)| {
            if i != 0 && i % 10_000_000 == 0 {
                println!("processed {i} lines in {:?}", start.elapsed());
                start = std::time::Instant::now();
            }

            let line = read_line.unwrap();
            let VideoUrls { display_id, urls } =
                serde_json::from_str(&line).unwrap();

            let Some(&catind) = catmap.get(&display_id) else { return };

            urls.into_iter().for_each(|url| {
                let Some(caps) = wiki_pattern.captures(&url) else { return };
                let article = caps[1].to_string();
                *domain_freqs[catind].entry(article).or_insert(0) += 1;
            });
        });

    CATEGORIES.iter().zip(&mut domain_freqs).for_each(
        |(cat_name, cat_freqs)| {
            let mut sorted_domains: Vec<(String, u64)> =
                cat_freqs.drain().collect();
            sorted_domains.sort_by_key(|e| e.1);
            let most_domains: Vec<(f64, String)> = sorted_domains
                .drain(..)
                .rev()
                .take(100)
                .map(|e| ((e.1 as f64).log10(), e.0))
                .collect();

            let chart = Chart::new()
                .tooltip(Tooltip::new().trigger(Trigger::Item))
                .series(
                    Pie::new()
                        .name(*cat_name)
                        .rose_type(PieRoseType::Area)
                        .radius(vec!["50", "200"])
                        .center(vec!["50%", "50%"])
                        .item_style(ItemStyle::new().border_radius(8))
                        .data(most_domains)
                        .emphasis(
                            Emphasis::new().item_style(
                                ItemStyle::new()
                                    .shadow_blur(10)
                                    .shadow_offset_x(0)
                                    .shadow_color("rgba(0, 0, 0, 0.5)"),
                            ),
                        ),
                );

            let mut renderer =
                HtmlRenderer::new(*cat_name, 1000, 1000).theme(Theme::Dark);
            renderer
                .save(&chart, format!("wiki_{}.html", cat_name))
                .unwrap();
        },
    );
}
