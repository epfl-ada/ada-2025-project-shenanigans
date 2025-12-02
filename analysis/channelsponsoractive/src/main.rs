use std::collections::HashSet;
use std::collections::hash_map::HashMap;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use charming::component::{Feature, MagicType, MagicTypeType, Toolbox};
use charming::element::{AxisTick, ItemStyle, SplitLine};
use charming::{
    Chart, HtmlRenderer,
    component::{Axis, Legend, Title},
    element::{
        AxisLabel, AxisType, Easing, Emphasis, EmphasisFocus, Formatter,
        JsFunction, TextAlign, Tooltip, Trigger,
    },
    series::{Series, bar},
    theme::Theme,
};
use flate2::read::GzDecoder;
use plot_utils::COLOURS;
use serde::Deserialize;
use serde_json;

use utils::CATEGORIES;

#[derive(Deserialize)]
struct Videos {}

fn main() {
    let file = File::open("../process/sponsoredchannels.json.gz").unwrap();
    let reader = BufReader::new(GzDecoder::new(file));
    let sponsored: HashMap<String, Videos> =
        serde_json::from_reader(reader).unwrap();
    let sponsored: HashSet<String> =
        sponsored.into_iter().map(|(e, _)| e).collect();

    let file = File::open("../process/to_merge_kalan_analysis.csv").unwrap();
    let reader = BufReader::new(file);

    let mut counts = [[0u64; 4]; CATEGORIES.len()];
    reader
        .lines()
        .skip(1)
        .enumerate()
        .for_each(|(i, read_line)| {
            let line = read_line.unwrap();
            let mut splits = line.split(',');

            let cat = splits.next().unwrap();
            let channel_id = splits.nth(1).unwrap();
            let tmp = splits.last().unwrap();
            let active = match tmp {
                "active" => true,
                "not_active" => false,
                _ => {
                    println!("{i}, {tmp}");
                    return;
                }
            };

            let Some(cat_ind) = CATEGORIES.iter().position(|&e| e == cat)
            else {
                return;
            };

            if active {
                if sponsored.contains(channel_id) {
                    counts[cat_ind][0] += 1;
                } else {
                    counts[cat_ind][1] += 1;
                }
            } else {
                if sponsored.contains(channel_id) {
                    counts[cat_ind][2] += 1;
                } else {
                    counts[cat_ind][3] += 1;
                }
            }
        });

    let freqs: Vec<(String, Vec<f64>)> = counts
        .into_iter()
        .zip(CATEGORIES)
        .filter_map(|(a, n)| {
            let sum = a.iter().sum::<u64>() as f64;
            if sum == 0.0 {
                return None;
            }
            Some((
                n.to_string(),
                a.into_iter().map(|e| 100.0 * e as f64 / sum).collect(),
            ))
        })
        .collect();

    let mut chart = Chart::new()
        .title(
            Title::new()
                .text("Channels Active in 2025")
                .text_align(TextAlign::Center)
                .left("50%"),
        )
        .tooltip(Tooltip::new().trigger(Trigger::Item))
        .animation_duration(1500.0)
        .animation_easing(Easing::CubicInOut)
        .y_axis(
            Axis::new()
                .type_(AxisType::Category)
                .axis_tick(AxisTick::new().show(false))
                .split_line(SplitLine::new().show(false))
                .name("categories")
                .data(
                    freqs.iter().map(|(e, _)| e.replace('&', "&\n")).collect(),
                ),
        )
        .x_axis(
            Axis::new()
                .type_(AxisType::Value)
                .max("dataMax")
                .axis_label(AxisLabel::new().formatter(Formatter::Function(
                    JsFunction::new_with_args(
                        "param",
                        "return Math.round(param * 100) / 100;",
                    ),
                )))
                .name("% channels"),
        )
        .toolbox(
            Toolbox::new().feature(Feature::new().magic_type(
                MagicType::new().type_(vec![MagicTypeType::Stack]),
            )),
        );

    let tooltip = Tooltip::new().formatter(Formatter::Function(
        JsFunction::new_with_args(
            "param",
            r#"
            return Math.round(param.data * 100) / 100 + "%";
            "#,
        ),
    ));

    let names = [
        "active sponsored",
        "active",
        "inactive sponsored",
        "inactive",
    ];

    for (i, n) in names.iter().enumerate() {
        chart = chart.series(Series::Bar(
            bar::Bar::new()
                .stack("bars")
                .item_style(ItemStyle::new().border_radius(50.0))
                .emphasis(Emphasis::new().focus(EmphasisFocus::Series))
                .tooltip(tooltip.clone())
                .name(n.to_string())
                .data(freqs.iter().map(|(_, e)| e[i]).collect()),
        ))
    }

    chart = chart.legend(Legend::new().data(names.to_vec())).color(vec![
        COLOURS[12],
        COLOURS[10],
        COLOURS[4],
        COLOURS[5],
    ]);

    let mut renderer =
        HtmlRenderer::new("active", 900, 600).theme(Theme::Custom(
            "sheNaNigans",
            include_str!("../../theme/sheNaNigans.js"),
        ));
    renderer.save(&chart, "active.html").unwrap();
}
