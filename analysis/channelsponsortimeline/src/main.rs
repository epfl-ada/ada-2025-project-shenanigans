use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use charming::{
    Chart, HtmlRenderer,
    component::{Axis, Grid, Title},
    element::{
        AxisType, Easing, Emphasis, EmphasisFocus, Formatter, JsFunction,
        TextAlign, Tooltip, Trigger,
    },
    series::{Series, bar},
    theme::Theme,
};
use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use ndarray::prelude::*;
use noisy_float::prelude::*;
use serde::Deserialize;

use plot_utils::compute_histogram;

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

    let data = compute_histogram(diffs, 101, true, false);

    let chart = Chart::new()
        .title(
            Title::new()
                .text("Days Before First Sponsored Video")
                .text_align(TextAlign::Center)
                .left("50%"),
        )
        .tooltip(Tooltip::new().trigger(Trigger::Item))
        .animation_duration(1500.0)
        .animation_easing(Easing::CubicInOut)
        .x_axis(Axis::new().name("days").type_(AxisType::Log))
        .y_axis(Axis::new().name("channels"))
        .grid(Grid::new())
        .series(Series::Bar(
            bar::Bar::new()
                .bar_width("100%")
                .tooltip(Tooltip::new().trigger(Trigger::Item).formatter(
                    Formatter::Function(JsFunction::new_with_args(
                        "param",
                        r#"
                        return param.data[1].toString() + " channels";
                        "#,
                    )),
                ))
                .emphasis(Emphasis::new().focus(EmphasisFocus::Adjacency))
                .data(data),
        ));

    let mut renderer =
        HtmlRenderer::new("dtbar", 900, 600).theme(Theme::Custom(
            "sheNaNigans",
            include_str!("../../theme/sheNaNigans.js"),
        ));
    renderer.save(&chart, "dtbar.html").unwrap();
}
