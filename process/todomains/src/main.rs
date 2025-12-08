use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use regex::Regex;
use serde::Deserialize;
use serde_json;

use utils::line_progress;

#[derive(Deserialize)]
struct VideoUrls {
    urls: Vec<String>,
}

fn main() {
    let domain_pattern = Regex::new(
        r#"(?x)
        (?:www\.)?                      # www
        ([\-[:word:]@:%\+~\#=\.]{1,256} # domains and subdomains
        \.
        [[:alnum:]()]{1,6})             # top-level domain
        "#,
    )
    .unwrap();

    let file = File::open("sponsoredurls_unshortened.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let mut start = std::time::Instant::now();
    let mut domain_freqs = HashMap::<String, u64>::new();
    reader.lines().enumerate().for_each(|(i, read_line)| {
        line_progress(i, &mut start, 10_000_000);

        let line = read_line.unwrap();
        let VideoUrls { urls } = serde_json::from_str(&line).unwrap();

        urls.into_iter().for_each(|url| {
            let domain =
                domain_pattern.captures(&url).unwrap()[1].to_lowercase();
            *domain_freqs.entry(domain).or_insert(0) += 1;
        });
    });

    let file = File::create("sponsoreddomains_unshortened.csv.gz").unwrap();
    let mut writer =
        BufWriter::new(GzEncoder::new(file, Compression::default()));

    let mut domain_freqs: Vec<(String, u64)> =
        domain_freqs.drain().filter(|(_, e)| *e >= 3).collect();
    domain_freqs.sort_by_key(|(_, e)| *e);
    let buf = domain_freqs
        .drain(..)
        .rev()
        .fold(String::new(), |acc, (n, f)| {
            acc + &n + "," + &f.to_string() + "\n"
        });
    writer.write_all(buf.as_bytes()).unwrap();
}
