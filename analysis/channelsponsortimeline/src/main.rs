use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use charming::{
    Chart, HtmlRenderer,
    component::{Axis, Grid, GridTooltip, Title},
    datatype::DataFrame,
    element::{AxisType, ItemStyle, TextAlign, Tooltip, Trigger},
    series::{Series, bar},
    theme::Theme,
};
use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use ndarray::prelude::*;
use ndarray_stats::{HistogramExt, histogram};
use noisy_float::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Videos {
    sponsored: Vec<Video>,
    not_sponsored: Vec<Video>,
}

#[derive(Deserialize)]
struct Video {
    upload_date: DateTime<Utc>,
}

fn main() {
    let file = File::open("../process/sponsoredchannels.json.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let sponvids: HashMap<String, Videos> =
        serde_json::from_reader(reader).unwrap();

    let diffs: Vec<N64> = sponvids
        .values()
        .filter_map(|videos| {
            let Some(first_nspon) = videos.not_sponsored.first() else {
                return None;
            };
            let Some(first_spon) = videos.sponsored.first() else {
                return None;
            };
            if first_spon.upload_date < first_nspon.upload_date {
                None
            } else {
                Some(first_spon.upload_date - first_nspon.upload_date)
            }
        })
        .map(|e| n64(e.num_days() as f64))
        .collect();
    let diffs = Array1::from_vec(diffs);
    let diff_min = *diffs.iter().min().unwrap();
    let diff_max = *diffs.iter().max().unwrap();

    let lin = Array1::linspace(diff_min, diff_max, 100);
    let bins = histogram::Bins::new(lin.clone().into());
    let grid = histogram::Grid::from(vec![bins]);

    let histogram = diffs.insert_axis(Axis(1)).histogram(grid);

    let data: DataFrame = lin
        .iter()
        .zip(histogram.counts())
        .map(|(x, y)| vec![x.raw(), *y as f64].into())
        .collect();

    let chart = Chart::new()
        .title(
            Title::new()
                .text("Days Before First Sponsored Video")
                .text_align(TextAlign::Center)
                .left("50%"),
        )
        .tooltip(Tooltip::new().trigger(Trigger::Item))
        .animation_duration(1500.0)
        .x_axis(Axis::new().name("days"))
        .y_axis(Axis::new().name("channels").type_(AxisType::Log))
        .grid(Grid::new().tooltip(GridTooltip::new().trigger(Trigger::Axis)))
        .series(Series::Bar(
            bar::Bar::new()
                .bar_width(3)
                .item_style(ItemStyle::new().border_radius(50))
                .data(data),
        ));

    let mut renderer =
        HtmlRenderer::new("dtbar", 900, 600).theme(Theme::Custom(
            "sheNaNigans",
            include_str!("../../theme/sheNaNigans.js"),
        ));
    renderer.save(&chart, "dtbar.html").unwrap();
}
