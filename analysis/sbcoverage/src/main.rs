use std::collections::HashSet;
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
use utils::{CATEGORIES, YEARS};

#[derive(Deserialize)]
struct VideoEntry {
    display_id: String,
    categories: String,
    upload_date: String,
}

fn process() -> ([[[u32; 2]; 15]; 16], [[[u32; 2]; 15]; 16]) {
    let start = std::time::Instant::now();
    let file = File::open("../sponsorblock/sponsorTimes.csv").unwrap();
    let reader = BufReader::new(file);
    let mut labelled = HashSet::<String>::new();
    let sponsored: HashSet<String> = reader
        .lines()
        .skip(1)
        .filter_map(|read_line| {
            let mut line = read_line.unwrap();

            if line.starts_with(r#"","#) || line.starts_with(r#""""#) {
                return None;
            }
            if line.starts_with('"') {
                line = line.trim_start_matches('"').to_string();
            }

            let mut splits = line.split(',');
            let Some(video_id) = splits.next() else {
                return None;
            };
            let Some(cat) = splits.nth(9) else {
                return None;
            };

            labelled.insert(video_id.to_string());

            if cat == "sponsor" { Some(video_id.to_string()) } else { None }
        })
        .collect();

    println!(
        "constructed sponsor set in {:?} for {} videos",
        start.elapsed(),
        sponsored.len()
    );
    println!(
        "constructed labelled set in {:?} for {} videos",
        start.elapsed(),
        labelled.len()
    );

    let mut start = std::time::Instant::now();
    let file = File::open("../dataset/yt_metadata_en.jsonl.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));

    let mut year_spon = [[[0u32; 2]; 15]; 16];
    let mut year_lab = [[[0u32; 2]; 15]; 16];
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
        if labelled.contains(&display_id) {
            year_lab[cat][y_ind][0] += 1;
        } else {
            year_lab[cat][y_ind][1] += 1;
        }
    });

    (year_spon, year_lab)
}

fn main() {
    let temp_path = Path::new("sbbar.json");
    let (year_spon, year_lab) = if let Ok(file) = File::open(temp_path) {
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    } else {
        let res = process();
        let file = File::create(temp_path).unwrap();
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &res).unwrap();
        res
    };

    let totals_spon: [u32; YEARS.len()] = std::array::from_fn(|i| {
        year_spon.iter().map(|e| e[i].iter().sum::<u32>()).sum()
    });
    let data_spon: Vec<Vec<f64>> = year_spon
        .iter()
        .map(|e| {
            e.iter()
                .zip(totals_spon.iter())
                .map(|(f, u)| 100.0 * f[0] as f64 / *u as f64)
                .collect()
        })
        .collect();

    let totals_lab: [u32; YEARS.len()] = std::array::from_fn(|i| {
        year_lab.iter().map(|e| e[i].iter().sum::<u32>()).sum()
    });
    let data_lab: Vec<Vec<f64>> = year_lab
        .iter()
        .map(|e| {
            e.iter()
                .zip(totals_lab.iter())
                .map(|(f, u)| 100.0 * f[0] as f64 / *u as f64)
                .collect()
        })
        .collect();

    let x_axis = Axis::new()
        .name("year")
        .axis_tick(AxisTick::new().show(false))
        .type_(AxisType::Category)
        .boundary_gap(BoundaryGap::CategoryAxis(false))
        .data(YEARS.iter().map(|e| e.to_string()).collect());

    let y_axis = Axis::new().name("% videos").type_(AxisType::Value);

    let mut chart = Chart::new()
        .title(
            Title::new()
                .text("SponsorBlock All Categories")
                .text_align(TextAlign::Center)
                .left("22.5%"),
        )
        .title(
            Title::new()
                .text("SponsorBlock Sponsored Category")
                .text_align(TextAlign::Center)
                .left("77.5%"),
        )
        .tooltip(Tooltip::new().trigger(Trigger::Item))
        .animation_duration(1500.0)
        .grid(
            Grid::new()
                .right("55%")
                .tooltip(GridTooltip::new().trigger(Trigger::Axis)),
        )
        .grid(
            Grid::new()
                .left("55%")
                .tooltip(GridTooltip::new().trigger(Trigger::Axis)),
        )
        .x_axis(x_axis.clone().grid_index(0))
        .y_axis(y_axis.clone().grid_index(0))
        .x_axis(x_axis.grid_index(1))
        .y_axis(y_axis.grid_index(1))
        .legend(
            Legend::new().top("bottom").data(
                CATEGORIES
                    .iter()
                    .zip(PATHS)
                    .map(|(c, p)| (c.to_string(), "path://".to_string() + p))
                    .collect(),
            ),
        );

    let tooltip_fn_spon = format!(
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
        totals_spon
    );
    let tooltip =
        Tooltip::new()
            .trigger(Trigger::Item)
            .formatter(Formatter::Function(JsFunction::new_with_args(
                "param",
                &tooltip_fn_spon,
            )));

    for (&c, d) in CATEGORIES.iter().zip(data_spon) {
        chart = chart.series(
            Line::new()
                .name(c)
                .stack("spon")
                .tooltip(tooltip.clone())
                .emphasis(Emphasis::new().focus(EmphasisFocus::Series))
                .area_style(AreaStyle::new().opacity(0.2))
                .symbol_size(8)
                .x_axis_index(1)
                .y_axis_index(1)
                .data(d),
        );
    }

    let tooltip_fn_lab = format!(
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
        totals_lab
    );
    let tooltip =
        Tooltip::new()
            .trigger(Trigger::Item)
            .formatter(Formatter::Function(JsFunction::new_with_args(
                "param",
                &tooltip_fn_lab,
            )));

    for (&c, d) in CATEGORIES.iter().zip(data_lab) {
        chart = chart.series(
            Line::new()
                .name(c)
                .stack("lab")
                .tooltip(tooltip.clone())
                .emphasis(Emphasis::new().focus(EmphasisFocus::Series))
                .area_style(AreaStyle::new().opacity(0.2))
                .symbol_size(8)
                .x_axis_index(0)
                .y_axis_index(0)
                .data(d),
        );
    }

    let mut renderer =
        HtmlRenderer::new("sbbar", 1200, 600).theme(Theme::Custom(
            "sheNaNigans",
            include_str!("../../theme/sheNaNigans.js"),
        ));
    renderer.save(&chart, "sbbar.html").unwrap();
}
