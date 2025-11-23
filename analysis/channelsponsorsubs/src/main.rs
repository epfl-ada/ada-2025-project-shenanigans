use std::fs::File;
use std::io::BufReader;
use std::{collections::HashMap, io::BufRead};

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
use chrono::{DateTime, NaiveDateTime, Utc};
use flate2::read::GzDecoder;
use ndarray::prelude::*;
use noisy_float::prelude::*;
use serde::Deserialize;

use plot_utils::{compute_histogram, describe};
use utils::line_progress;

#[derive(Deserialize)]
struct Videos {
    sponsored: Vec<Video>,
}

#[derive(Deserialize)]
struct Video {
    upload_date: DateTime<Utc>,
}

#[derive(Debug)]
struct ChannelTimePoint {
    timepoint: DateTime<Utc>,
    views: f64,
    subs: f64,
    videos: f64,
    activity: f64,
}

fn main() {
    let start = std::time::Instant::now();
    let file = File::open("../process/sponsoredchannels.json.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let sponvids: HashMap<String, Videos> =
        serde_json::from_reader(reader).unwrap();
    println!(
        "loaded channel map in {:?}",
        std::time::Instant::now() - start
    );

    // channel category datetime views delta_views subs delta_subs videos delta_videos activity

    let mut start = std::time::Instant::now();
    let file = File::open("../dataset/df_timeseries_en.tsv.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let mut timeseries = HashMap::<String, Vec<ChannelTimePoint>>::new();
    reader.lines().enumerate().for_each(|(i, read_line)| {
        line_progress(i, &mut start, 1_000_000);

        let line = read_line.unwrap();
        let mut splits = line.split("\t");

        let channel_id = splits.next().unwrap();
        if !sponvids.contains_key(channel_id) {
            return;
        }

        let date_str = splits.nth(1).unwrap();
        let parsed =
            NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S")
                .unwrap();
        let timepoint = DateTime::<Utc>::from_naive_utc_and_offset(parsed, Utc);

        let views: f64 = splits.next().unwrap().parse().unwrap();
        let subs: f64 = splits.nth(1).unwrap().parse().unwrap();
        let videos: f64 = splits.nth(1).unwrap().parse().unwrap();
        let activity: f64 = splits.nth(1).unwrap().parse().unwrap();

        assert!(splits.count() == 0);

        let entry = timeseries.entry(channel_id.to_owned()).or_insert(vec![]);
        entry.push(ChannelTimePoint {
            timepoint,
            views,
            subs,
            videos,
            activity,
        });
    });

    let (view_thresh, sub_thresh, vid_thresh, act_thresh): (
        Vec<N64>,
        Vec<N64>,
        Vec<N64>,
        Vec<N64>,
    ) = sponvids
        .iter()
        .filter_map(|(channel_id, videos)| {
            let vid = videos.sponsored.first().unwrap();
            let Some(points) = timeseries.get(channel_id) else {
                return None;
            };

            let ChannelTimePoint {
                views,
                subs,
                videos,
                activity,
                ..
            } = points
                .iter()
                .min_by(|a, b| {
                    let delta_a = (a.timepoint - vid.upload_date).abs();
                    let delta_b = (b.timepoint - vid.upload_date).abs();
                    delta_a.cmp(&delta_b)
                })
                .unwrap();
            Some((n64(*views), n64(*subs), n64(*videos), n64(*activity)))
        })
        .collect();

    let stats: Vec<(String, Array1<N64>)> =
        ["views", "subscribers", "videos", "activity"]
            .iter()
            .zip([view_thresh, sub_thresh, vid_thresh, act_thresh])
            .map(|(n, d)| (n.to_string(), Array1::from_vec(d)))
            .collect();

    stats.iter().for_each(|(n, d)| {
        println!("{n}");
        describe(d);
        println!();
    });

    let mut chart = Chart::new()
        .title(
            Title::new()
                .text("Channel Statistics at First Sponsored Video")
                .text_align(TextAlign::Center)
                .left("50%"),
        )
        .tooltip(Tooltip::new().trigger(Trigger::Item))
        .animation_duration(1500.0)
        .animation_easing(Easing::CubicInOut)
        .grid(Grid::new().right("57%").bottom("57%"))
        .grid(Grid::new().left("57%").bottom("57%"))
        .grid(Grid::new().right("57%").top("57%"))
        .grid(Grid::new().left("57%").top("57%"));

    let data: Vec<(String, DataFrame)> = stats
        .into_iter()
        .map(|(n, d)| {
            let log_y = n == "activity";
            (n, compute_histogram(d, 101, true, log_y))
        })
        .collect();

    let axis_label = AxisLabel::new().formatter(Formatter::Function(
        JsFunction::new_with_args(
            "value, index",
            r#"return value.toExponential().replace("+","");"#,
        ),
    ));

    let log_y_tooltip_fn = r#"
        const chan = Math.round(Math.pow(10, param.data[1]));
        return chan.toString() + " channels";
        "#;
    let tooltip_fn = r#"
        return param.data[1].toString() + " channels";
        "#;

    for (i, (n, d)) in data.into_iter().enumerate() {
        chart = chart
            .x_axis(
                Axis::new()
                    .name(n.clone())
                    .type_(AxisType::Log)
                    .axis_label(axis_label.clone())
                    .grid_index(i as f64),
            )
            .y_axis(
                Axis::new()
                    .name(if n == "activity" {
                        "log10 channels"
                    } else {
                        "channels"
                    })
                    .grid_index(i as f64),
            )
            .series(Series::Bar(
                bar::Bar::new()
                    .bar_width("100%")
                    .x_axis_index(i as f64)
                    .y_axis_index(i as f64)
                    .tooltip(Tooltip::new().trigger(Trigger::Item).formatter(
                        Formatter::Function(JsFunction::new_with_args(
                            "param",
                            if n == "activity" {
                                log_y_tooltip_fn
                            } else {
                                tooltip_fn
                            },
                        )),
                    ))
                    .emphasis(Emphasis::new().focus(EmphasisFocus::Adjacency))
                    .data(d),
            ));
    }

    let mut renderer = HtmlRenderer::new("big", 900, 600).theme(Theme::Custom(
        "sheNaNigans",
        include_str!("../../theme/sheNaNigans.js"),
    ));
    renderer.save(&chart, "big.html").unwrap();
}
