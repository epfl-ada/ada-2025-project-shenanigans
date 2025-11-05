use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

const MIN_COMMENTS: u32 = 3;
const MAX_COMMENTS: u32 = 136;

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
    pub data: Vec<Vec<u32>>,
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
    let file = File::open("./cats.csv.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let file = File::create("cats_gteq3_lteq136.csv.gz").unwrap();
    let mut writer =
        BufWriter::new(GzEncoder::new(file, Compression::default()));
    writeln!(&mut writer, "{}", CATEGORIES.join(",")).unwrap();

    let mut start = std::time::Instant::now();
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
            let split = line.splitn(CATEGORIES.len(), ",");
            let comms: Vec<u32> = split.map(|e| e.parse().unwrap()).collect();
            assert_eq!(comms.len(), CATEGORIES.len());

            let count = comms.iter().sum::<u32>();
            if MIN_COMMENTS <= count && count <= MAX_COMMENTS {
                table.data.push(comms);
            }

            if table.data.len() >= 50_000_000 {
                writer.write_all(table.to_string().as_bytes()).unwrap();
                table.data.clear();
            }
        });

    writer.write_all(table.to_string().as_bytes()).unwrap();
}
