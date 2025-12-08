use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use charming::{
    Chart, HtmlRenderer,
    component::{Axis, Grid, Title},
    datatype::DataFrame,
    element::{
        AxisLabel, AxisType, Easing, Emphasis, EmphasisFocus, Formatter,
        JsFunction, TextAlign, Tooltip, Trigger,
    },
    series::{Series, bar},
    theme::Theme,
};
use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use ndarray::prelude::*;
use noisy_float::prelude::*;
use serde::Deserialize;

use plot_utils::{compute_histogram, describe, xy_to_df};

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

    let (diffs, nbef): (Vec<N64>, Vec<N64>) = sponvids
        .values()
        .filter_map(|videos| {
            let [ref first_nspon, ..] = videos.not_sponsored[..] else {
                return None;
            };
            let [ref first_spon, ..] = videos.sponsored[..] else {
                return None;
            };
            if first_spon.upload_date > first_nspon.upload_date {
                Some((
                    first_spon.upload_date - first_nspon.upload_date,
                    videos
                        .not_sponsored
                        .iter()
                        .filter(|e| e.upload_date < first_spon.upload_date)
                        .count(),
                ))
            } else {
                None
            }
        })
        .map(|(d, n)| (n64(d.num_days() as f64), n64(n as f64)))
        .collect();

    // ndarray_npy::write_npy(
    //     "days.npy",
    //     &Array1::from_iter(diffs.iter().map(|e| e.raw())),
    // )
    // .unwrap();

    let stats: Vec<(String, Array1<N64>)> = ["days", "videos"]
        .iter()
        .zip([diffs, nbef])
        .map(|(n, d)| (n.to_string(), Array1::from_vec(d)))
        .collect();

    stats.iter().for_each(|(n, d)| {
        println!("{n}");
        describe(&d);
    });

    let mut chart = Chart::new()
        .title(
            Title::new()
                .text("Time Before First Sponsor")
                .text_align(TextAlign::Center)
                .left("22.5%"),
        )
        .title(
            Title::new()
                .text("Number of Videos Before First Sponsor")
                .text_align(TextAlign::Center)
                .left("77.5%"),
        )
        .tooltip(Tooltip::new().trigger(Trigger::Item))
        .animation_duration(1500.0)
        .animation_easing(Easing::CubicInOut)
        .grid(Grid::new().right("55%"))
        .grid(Grid::new().left("55%"));

    let data: Vec<(String, DataFrame, f64)> = stats
        .into_iter()
        .map(|(n, d)| {
            println!("{}", d.len());
            let (x, mut y) = compute_histogram(d, 101, true, false, None, None);
            let sum = y.sum();
            y.iter_mut().for_each(|e| *e /= sum);
            (n, xy_to_df(x, y), sum.raw())
        })
        .collect();

    let axis_label = AxisLabel::new().formatter(Formatter::Function(
        JsFunction::new_with_args(
            "value, index",
            r#"return value.toExponential().replace("+","");"#,
        ),
    ));

    for (i, (n, d, a)) in data.into_iter().enumerate() {
        chart = chart
            .x_axis(
                Axis::new()
                    .name(n.clone())
                    .type_(AxisType::Log)
                    .axis_label(axis_label.clone())
                    .grid_index(i as f64),
            )
            .y_axis(Axis::new().name("channel density").grid_index(i as f64))
            .color(vec!["#eca9b5", "#eca9b5"])
            .series(Series::Bar(
                bar::Bar::new()
                    .bar_width("100%")
                    .x_axis_index(i as f64)
                    .y_axis_index(i as f64)
                    .tooltip(Tooltip::new().trigger(Trigger::Item).formatter(
                        Formatter::Function(JsFunction::new_with_args(
                            "param",
                            &format!(
                                r#"
                                const chan = Math.round(param.data[1] * {});
                                return chan.toString() + " channels";
                                "#,
                                a
                            ),
                        )),
                    ))
                    .emphasis(Emphasis::new().focus(EmphasisFocus::Adjacency))
                    .data(d),
            ));
    }

    let mut renderer =
        HtmlRenderer::new("dtbar", 1200, 600).theme(Theme::Custom(
            "sheNaNigans",
            include_str!("../../theme/sheNaNigans.js"),
        ));
    renderer.save(&chart, "dtbar.html").unwrap();
}
