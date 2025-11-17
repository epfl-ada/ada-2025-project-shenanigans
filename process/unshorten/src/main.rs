use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::sync::{Arc, Mutex};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use futures::{StreamExt, stream};
use reqwest;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use serde::Deserialize;
use serde_json;

#[derive(Deserialize)]
struct VideoUrls {
    urls: Vec<String>,
}

async fn unshorten(
    url: String,
    client: ClientWithMiddleware,
    map: Arc<Mutex<HashMap<String, String>>>,
) {
    if map.lock().unwrap().contains_key(&url) {
        return;
    };

    let Ok(response) = client.head(&url).send().await else {
        eprintln!("request failed: {url}");
        return;
    };
    if response.status().is_redirection() {
        let tmp = response
            .headers()
            .get(reqwest::header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        map.lock().unwrap().insert(url, tmp);
    } else {
        map.lock()
            .unwrap()
            .insert(url, response.status().to_string());
        eprintln!("{response:?}");
    }
}

#[tokio::main]
async fn main() {
    let file = File::open("urls.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let client = ClientBuilder::new(
        Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap(),
    )
    .with(RetryTransientMiddleware::new_with_policy(
        ExponentialBackoff::builder().build_with_max_retries(10),
    ))
    .build();

    let short_unshort = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    let tmp = Arc::clone(&short_unshort);

    let mut start = std::time::Instant::now();
    let urls = reader
        .lines()
        .enumerate()
        .flat_map(|(i, read_line)| {
            if i != 0 && i % 100_000 == 0 {
                println!("processed {i} lines in {:?}", start.elapsed());
                start = std::time::Instant::now();

                let file = File::create("bit_ly.csv.gz").unwrap();
                let mut writer = BufWriter::new(GzEncoder::new(
                    file,
                    Compression::default(),
                ));
                let buf = tmp
                    .lock()
                    .unwrap()
                    .iter()
                    .fold(String::new(), |acc, (s, l)| {
                        acc + &s + "," + &l + "\n"
                    });
                writer.write_all(buf.as_bytes()).unwrap();
            }

            let line = read_line.unwrap();
            let VideoUrls { urls } = serde_json::from_str(&line).unwrap();
            urls
        })
        .filter_map(|url| {
            if !url.contains("bit.ly") {
                return None;
            }

            if url.starts_with("http://") || url.starts_with("https://") {
                Some(url)
            } else {
                Some("http://".to_string() + &url)
            }
        });

    let reqs = stream::iter(urls)
        .map(|url| {
            let client = client.clone();
            let short_unshort = Arc::clone(&short_unshort);
            tokio::spawn(
                async move { unshorten(url, client, short_unshort).await },
            )
        })
        .buffer_unordered(32);

    reqs.collect::<Vec<_>>().await;

    let file = File::create("bit_ly.csv.gz").unwrap();
    let mut writer =
        BufWriter::new(GzEncoder::new(file, Compression::default()));
    let buf = short_unshort
        .lock()
        .unwrap()
        .drain()
        .fold(String::new(), |acc, (s, l)| acc + &s + "," + &l + "\n");
    writer.write_all(buf.as_bytes()).unwrap();
}
