use charming::{
    Chart, HtmlRenderer,
    component::{Axis, Legend, LegendSelectedMode, Title},
    datatype::CompositeValue,
    element::{
        AxisTick, AxisType, BackgroundStyle, Color, Emphasis, EmphasisFocus,
        Formatter, ItemStyle, JsFunction, TextAlign, Tooltip, Trigger,
    },
    series::{Bar, Series, bar},
    theme::Theme,
};
use flate2::read::GzDecoder;
use serde::Deserialize;
use serde_json;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{
    collections::{HashMap, HashSet},
    io::BufWriter,
};

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

const YEARS: [u16; 15] = [
    2005, 2006, 2007, 2008, 2009, 2010, 2011, 2012, 2013, 2014, 2015, 2016,
    2017, 2018, 2019,
];

#[derive(Deserialize)]
struct VideoEntry {
    display_id: String,
    categories: String,
    upload_date: String,
}

fn main() {
    let start = std::time::Instant::now();
    let file = File::open("../sponsorblock/sponsorTimes.csv").unwrap();
    let reader = BufReader::new(file);

    let sponsored: HashSet<String> = reader
        .lines()
        .skip(1)
        .filter_map(|read_line| {
            let line = read_line.unwrap();
            if line.starts_with(r#"","#) || line.starts_with(r#""""#) {
                return None;
            }
            if line.starts_with('"') {
                return Some(line.trim_start_matches('"').to_string());
            }
            if let Some((video_id, _)) = line.split_once(",") {
                Some(video_id.to_string())
            } else {
                None
            }
        })
        .collect();

    println!(
        "constructed sponsor set in {:?} for {} videos",
        start.elapsed(),
        sponsored.len()
    );

    let mut start = std::time::Instant::now();
    let file = File::open("../dataset/yt_metadata_en.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let mut year_spon = [[[0u32; 2]; 15]; 16];
    reader.lines().enumerate().for_each(|(i, read_line)| {
        if i != 0 && i % 10_000_000 == 0 {
            println!("processed {i} lines in {:?}", start.elapsed());
            start = std::time::Instant::now();
        }

        let line = read_line.unwrap();
        let Ok(VideoEntry {
            display_id,
            categories,
            upload_date,
        }) = serde_json::from_str(&line)
        else {
            return;
        };

        let Some(cat) = CATEGORIES.iter().position(|e| *e == categories) else {
            return;
        };

        let year: u16 = upload_date[..4].parse().unwrap();
        let y_ind = YEARS.iter().position(|e| *e == year).unwrap();
        if sponsored.contains(&display_id) {
            year_spon[cat][y_ind][0] += 1;
        } else {
            year_spon[cat][y_ind][1] += 1;
        }
    });

    let file = File::create("sbbar.json").unwrap();
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &year_spon).unwrap();

    let totals: [u32; YEARS.len()] = std::array::from_fn(|i| {
        year_spon.iter().map(|e| e[i].iter().sum::<u32>()).sum()
    });
    let js_fn = format!(
        r#"
        console.log(param);
        const totals = {:?};
        const count = Math.round(totals[param.dataIndex] * param.data);
        const label = count.toString()
            + " / "
            + totals[param.dataIndex]
            + " ("
            + param.data.toFixed(3)
            + "%)";
        return label;
        "#,
        totals
    );

    let data: Vec<Vec<f64>> = year_spon
        .iter()
        .map(|e| {
            e.iter()
                .zip(totals.iter())
                .map(|(f, u)| 100.0 * f[0] as f64 / *u as f64)
                .collect()
        })
        .collect();

    let tooltip =
        Tooltip::new()
            .trigger(Trigger::Item)
            .formatter(Formatter::Function(JsFunction::new_with_args(
                "param", &js_fn,
            )));
    let emphasis = Emphasis::new().focus(EmphasisFocus::Series);
    let bar_width = 10;
    let item_style = ItemStyle::new().border_radius(50);

    let chart = Chart::new()
        .title(
            Title::new()
                .text("SponsorBlock Videos")
                .text_align(TextAlign::Center)
                .left("50%"),
        )
        .tooltip(Tooltip::new().trigger(Trigger::Item))
        .x_axis(
            Axis::new()
                .name("year")
                .axis_tick(AxisTick::new().show(false))
                .type_(AxisType::Category)
                .data(YEARS.iter().map(|e| e.to_string()).collect()),
        )
        .y_axis(Axis::new().name("% videos").type_(AxisType::Value))
        .legend(Legend::new().top("bottom"))
        .series(Series::Bar(
            bar::Bar::new()
                .stack("videos")
                .name(CATEGORIES[0])
                .tooltip(tooltip.clone())
                .emphasis(emphasis.clone())
                .bar_width(bar_width)
                .item_style(item_style.clone())
                .data(data[0].clone()),
        ))
        .series(Series::Bar(
            bar::Bar::new()
                .stack("videos")
                .name(CATEGORIES[1])
                .tooltip(tooltip.clone())
                .emphasis(emphasis.clone())
                .bar_width(bar_width)
                .item_style(item_style.clone())
                .data(data[1].clone()),
        ))
        .series(Series::Bar(
            bar::Bar::new()
                .stack("videos")
                .name(CATEGORIES[2])
                .tooltip(tooltip.clone())
                .emphasis(emphasis.clone())
                .bar_width(bar_width)
                .item_style(item_style.clone())
                .data(data[2].clone()),
        ))
        .series(Series::Bar(
            bar::Bar::new()
                .stack("videos")
                .name(CATEGORIES[3])
                .tooltip(tooltip.clone())
                .emphasis(emphasis.clone())
                .bar_width(bar_width)
                .item_style(item_style.clone())
                .data(data[3].clone()),
        ))
        .series(Series::Bar(
            bar::Bar::new()
                .stack("videos")
                .name(CATEGORIES[4])
                .tooltip(tooltip.clone())
                .emphasis(emphasis.clone())
                .bar_width(bar_width)
                .item_style(item_style.clone())
                .data(data[4].clone()),
        ))
        .series(Series::Bar(
            bar::Bar::new()
                .stack("videos")
                .name(CATEGORIES[5])
                .tooltip(tooltip.clone())
                .emphasis(emphasis.clone())
                .bar_width(bar_width)
                .item_style(item_style.clone())
                .data(data[5].clone()),
        ))
        .series(Series::Bar(
            bar::Bar::new()
                .stack("videos")
                .name(CATEGORIES[6])
                .tooltip(tooltip.clone())
                .emphasis(emphasis.clone())
                .bar_width(bar_width)
                .item_style(item_style.clone())
                .data(data[6].clone()),
        ))
        .series(Series::Bar(
            bar::Bar::new()
                .stack("videos")
                .name(CATEGORIES[7])
                .tooltip(tooltip.clone())
                .emphasis(emphasis.clone())
                .bar_width(bar_width)
                .item_style(item_style.clone())
                .data(data[7].clone()),
        ))
        .series(Series::Bar(
            bar::Bar::new()
                .stack("videos")
                .name(CATEGORIES[8])
                .tooltip(tooltip.clone())
                .emphasis(emphasis.clone())
                .bar_width(bar_width)
                .item_style(item_style.clone())
                .data(data[8].clone()),
        ))
        .series(Series::Bar(
            bar::Bar::new()
                .stack("videos")
                .name(CATEGORIES[9])
                .tooltip(tooltip.clone())
                .emphasis(emphasis.clone())
                .bar_width(bar_width)
                .item_style(item_style.clone())
                .data(data[9].clone()),
        ))
        .series(Series::Bar(
            bar::Bar::new()
                .stack("videos")
                .name(CATEGORIES[10])
                .tooltip(tooltip.clone())
                .emphasis(emphasis.clone())
                .bar_width(bar_width)
                .item_style(item_style.clone())
                .data(data[10].clone()),
        ))
        .series(Series::Bar(
            bar::Bar::new()
                .stack("videos")
                .name(CATEGORIES[11])
                .tooltip(tooltip.clone())
                .emphasis(emphasis.clone())
                .bar_width(bar_width)
                .item_style(item_style.clone())
                .data(data[11].clone()),
        ))
        .series(Series::Bar(
            bar::Bar::new()
                .stack("videos")
                .name(CATEGORIES[12])
                .tooltip(tooltip.clone())
                .emphasis(emphasis.clone())
                .bar_width(bar_width)
                .item_style(item_style.clone())
                .data(data[12].clone()),
        ))
        .series(Series::Bar(
            bar::Bar::new()
                .stack("videos")
                .name(CATEGORIES[13])
                .tooltip(tooltip.clone())
                .emphasis(emphasis.clone())
                .bar_width(bar_width)
                .item_style(item_style.clone())
                .data(data[13].clone()),
        ))
        .series(Series::Bar(
            bar::Bar::new()
                .stack("videos")
                .name(CATEGORIES[14])
                .tooltip(tooltip.clone())
                .emphasis(emphasis.clone())
                .bar_width(bar_width)
                .item_style(item_style.clone())
                .data(data[14].clone()),
        ))
        .series(Series::Bar(
            bar::Bar::new()
                .stack("videos")
                .name(CATEGORIES[15])
                .tooltip(tooltip.clone())
                .emphasis(emphasis.clone())
                .bar_width(bar_width)
                .item_style(item_style.clone())
                .data(data[15].clone()),
        ));

    let mut renderer = HtmlRenderer::new("hello", 1000, 500).theme(Theme::Dark);
    renderer.save(&chart, "sbbar.html").unwrap();
}
