use std::fs::File;
use std::io::BufWriter;
use std::io::{BufRead, BufReader};
use std::path::Path;

use charming::{
    Chart, HtmlRenderer,
    component::{Axis, Grid, GridTooltip, Legend, Title},
    element::{
        AreaStyle, AxisTick, AxisType, BoundaryGap, Emphasis, EmphasisFocus,
        Formatter, JsFunction, TextAlign, Tooltip, Trigger,
    },
    series::Line,
    theme::Theme,
};
use flate2::read::GzDecoder;
use serde::Deserialize;
use serde_json;

use plot_utils::PATHS;
use utils::{CATEGORIES, YEARS, make_sponsor_set};

#[derive(Deserialize)]
struct VideoEntry {
    display_id: String,
    categories: String,
    upload_date: String,
}

fn process() -> [[[u32; 2]; 15]; 16] {
    let start = std::time::Instant::now();
    let file = File::open("../sponsorblock/sponsorTimes.csv").unwrap();
    let reader = BufReader::new(file);
    let sponsored = make_sponsor_set(reader).unwrap();

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

    year_spon
}

fn main() {
    let temp_path = Path::new("sbbar.json");
    let year_spon = if let Ok(file) = File::open(temp_path) {
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    } else {
        let res = process();
        let file = File::create(temp_path).unwrap();
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &res).unwrap();
        res
    };

    let totals: [u32; YEARS.len()] = std::array::from_fn(|i| {
        year_spon.iter().map(|e| e[i].iter().sum::<u32>()).sum()
    });

    let data: Vec<Vec<f64>> = year_spon
        .iter()
        .map(|e| {
            e.iter()
                .zip(totals.iter())
                .map(|(f, u)| 100.0 * f[0] as f64 / *u as f64)
                .collect()
        })
        .collect();

    let mut chart = Chart::new()
        .title(
            Title::new()
                .text("SponsorBlock Videos")
                .text_align(TextAlign::Center)
                .left("50%"),
        )
        .tooltip(Tooltip::new().trigger(Trigger::Item))
        .animation_duration(1500.0)
        .x_axis(
            Axis::new()
                .name("year")
                .axis_tick(AxisTick::new().show(false))
                .type_(AxisType::Category)
                .boundary_gap(BoundaryGap::CategoryAxis(false))
                .data(YEARS.iter().map(|e| e.to_string()).collect()),
        )
        .y_axis(Axis::new().name("% videos").type_(AxisType::Value))
        .grid(Grid::new().tooltip(GridTooltip::new().trigger(Trigger::Axis)))
        .legend(
            Legend::new().top("bottom").data(
                CATEGORIES
                    .iter()
                    .zip(PATHS)
                    .map(|(c, p)| {
                        let cn = if c.is_empty() { "Misc" } else { c };
                        (cn.to_string(), "path://".to_string() + p)
                    })
                    .collect(),
            ),
        );

    let tooltip_fn = format!(
        r#"
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
    let tooltip =
        Tooltip::new()
            .trigger(Trigger::Item)
            .formatter(Formatter::Function(JsFunction::new_with_args(
                "param",
                &tooltip_fn,
            )));

    for (&c, d) in CATEGORIES.iter().zip(data) {
        chart = chart.series(
            Line::new()
                .name(if c == "" { "Misc" } else { c })
                .stack("videos")
                .tooltip(tooltip.clone())
                .emphasis(Emphasis::new().focus(EmphasisFocus::Series))
                .area_style(AreaStyle::new().opacity(0.2))
                .symbol_size(8)
                .data(d),
        );
    }

    let mut renderer =
        HtmlRenderer::new("sbbar", 900, 600).theme(Theme::Custom(
            "sheNaNigans",
            include_str!("../../theme/sheNaNigans.js"),
        ));
    renderer.save(&chart, "sbbar.html").unwrap();
}
