use std::fs::File;
use std::io::BufReader;
use std::{collections::HashMap, io::BufRead};

use charming::component::Legend;
use charming::{
    Chart, HtmlRenderer,
    component::{Axis, Grid, Title},
    datatype::DataFrame,
    element::{
        AxisLabel, AxisType, Emphasis, EmphasisFocus, Formatter, JsFunction,
        TextAlign, Tooltip, Trigger,
    },
    series::{Series, bar},
    theme::Theme,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use flate2::read::GzDecoder;
use ndarray::prelude::*;
use noisy_float::prelude::*;
use serde::Deserialize;

use plot_utils::{COLOURS, PATHS, compute_histogram, xy_to_df};
use utils::{CATEGORIES, line_progress};

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
    category: usize,
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

        let cat_str = splits.next().unwrap();
        let Some(category) = CATEGORIES.iter().position(|&e| e == cat_str)
        else {
            return;
        };

        let date_str = splits.next().unwrap();
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
            category,
        });
    });

    let (view_thresh, sub_thresh, vid_thresh, act_thresh, cats): (
        Vec<N64>,
        Vec<N64>,
        Vec<N64>,
        Vec<N64>,
        Vec<usize>,
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
                category,
                ..
            } = points
                .iter()
                .min_by(|a, b| {
                    let delta_a = (a.timepoint - vid.upload_date).abs();
                    let delta_b = (b.timepoint - vid.upload_date).abs();
                    delta_a.cmp(&delta_b)
                })
                .unwrap();
            Some((
                n64(*views),
                n64(*subs),
                n64(*videos),
                n64(*activity),
                category,
            ))
        })
        .collect();

    let stats: Vec<(String, Vec<N64>)> =
        ["views", "subscribers", "videos", "activity"]
            .iter()
            .zip([view_thresh, sub_thresh, vid_thresh, act_thresh])
            .map(|(n, d)| (n.to_string(), d))
            .collect();

    // stats.iter().for_each(|(n, d)| {
    //     println!("{n}");
    //     describe(d);
    //     println!();
    // });

    let mut chart = Chart::new()
        .title(
            Title::new()
                .text("Channel Statistics at First Sponsored Video")
                .text_align(TextAlign::Center)
                .left("50%"),
        )
        .tooltip(Tooltip::new().trigger(Trigger::Item))
        // .animation_duration(1500.0)
        // .animation_easing(Easing::CubicInOut)
        .grid(Grid::new().right("57%").bottom("57%"))
        .grid(Grid::new().left("57%").bottom("57%"))
        .grid(Grid::new().right("57%").top("57%"))
        .grid(Grid::new().left("57%").top("57%"));

    let data: Vec<(String, Vec<(String, DataFrame, f64)>)> = stats
        .into_iter()
        .map(|(n, d)| {
            let d: Vec<N64> = d.into_iter().filter(|e| *e > 0.0).collect();
            let d_min = *d.iter().min().unwrap();
            let d_max = *d.iter().max().unwrap();

            let hist = (0..=CATEGORIES.len())
                .filter_map(|c| {
                    let filtered = if c == CATEGORIES.len() {
                        Array1::from_vec(d.clone())
                    } else {
                        Array1::from_iter(
                            d.iter().cloned().zip(cats.iter()).filter_map(
                                |(dp, dc)| {
                                    if *dc == c { Some(dp) } else { None }
                                },
                            ),
                        )
                    };

                    let cn = if c == CATEGORIES.len() {
                        "All".to_string()
                    } else {
                        CATEGORIES[c].to_string()
                    };

                    let log_y = n == "activity";
                    let (x, mut y) = compute_histogram(
                        filtered,
                        101,
                        true,
                        log_y,
                        Some(d_min),
                        Some(d_max),
                    );
                    let sum = y.sum();
                    if sum == 0.0 {
                        return None;
                    }
                    y.iter_mut().for_each(|e| *e /= sum);
                    Some((cn, xy_to_df(x, y), sum.raw()))
                })
                .collect();
            (n, hist)
        })
        .collect();

    let axis_label = AxisLabel::new().formatter(Formatter::Function(
        JsFunction::new_with_args(
            "value, index",
            r#"return value.toExponential().replace("+","");"#,
        ),
    ));

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
                        "log10 channel density"
                    } else {
                        "channel density"
                    })
                    .grid_index(i as f64),
            );
        for (cn, cd, ca) in d.into_iter().rev() {
            chart = chart
            .series(Series::Bar(
                bar::Bar::new()
                    .bar_width("100%")
                    .x_axis_index(i as f64)
                    .y_axis_index(i as f64)
                    .tooltip(Tooltip::new().trigger(Trigger::Item).formatter(
                        Formatter::Function(JsFunction::new_with_args(
                            "param",
                            &if n == "activity" {
                                format!(
                                    r#"
                                    const chan = Math.round(Math.pow(10, param.data[1] * {}));
                                    return chan.toString() + " channels";
                                    "#,
                                    ca
                                )
                            } else {
                                format!(
                                    r#"
                                    const chan = Math.round(param.data[1] * {});
                                    return chan.toString() + " channels";
                                    "#,
                                    ca
                                )
                            },
                        )),
                    ))
                    .emphasis(Emphasis::new().focus(EmphasisFocus::Series))
                    .name(if cn.is_empty() { "Misc".to_string() } else { cn })
                    .data(cd),
            ));
        }
    }

    let mut leg = vec![("All".to_string(), "circle".to_string())];
    leg.extend(CATEGORIES.iter().zip(PATHS).map(|(c, p)| {
        let name = if c.is_empty() { "Misc" } else { c };
        (name.to_string(), "path://".to_string() + p)
    }));
    chart = chart
        .legend(
            Legend::new()
                .top("bottom")
                .selected(leg.iter().map(|(e, _)| {
                    (e, ["All", "Entertainment"].contains(&e.as_str()))
                }))
                .data(leg),
        )
        .color(COLOURS.into_iter().rev().collect::<Vec<&str>>());

    let mut renderer = HtmlRenderer::new("big", 900, 600).theme(Theme::Custom(
        "sheNaNigans",
        include_str!("../../theme/sheNaNigans.js"),
    ));
    renderer.save(&chart, "big.html").unwrap();
}
