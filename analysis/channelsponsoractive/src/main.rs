use std::collections::HashSet;
use std::collections::hash_map::HashMap;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use charming::component::{Feature, Grid, MagicType, MagicTypeType, Toolbox};
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

    let freqs4: Vec<(String, Vec<f64>)> = counts
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

    println!("{}", serde_json::to_string_pretty(&freqs4).unwrap());

    let freqs2: Vec<(String, Vec<f64>)> = counts
        .into_iter()
        .zip(CATEGORIES)
        .filter_map(|(a, n)| {
            let sum = (a[0] + a[2]) as f64;
            if sum == 0.0 {
                return None;
            }
            Some((
                n.to_string(),
                vec![100.0 * a[0] as f64 / sum, 100.0 * a[2] as f64 / sum],
            ))
        })
        .collect();

    let freqsi: Vec<(String, Vec<f64>)> = counts
        .into_iter()
        .zip(CATEGORIES)
        .filter_map(|(a, n)| {
            let sum = (a[1] + a[3]) as f64;
            if sum == 0.0 {
                return None;
            }
            Some((
                n.to_string(),
                vec![100.0 * a[1] as f64 / sum, 100.0 * a[3] as f64 / sum],
            ))
        })
        .collect();

    let chart = Chart::new()
        .title(
            Title::new()
                .text("Channels Active in 2025")
                .text_align(TextAlign::Center)
                .left("50%"),
        )
        .tooltip(Tooltip::new().trigger(Trigger::Item))
        .animation_duration(500.0)
        .animation_easing(Easing::CubicInOut)
        .grid(Grid::new().bottom(50))
        .y_axis(
            Axis::new()
                .type_(AxisType::Category)
                .axis_tick(AxisTick::new().show(false))
                .split_line(SplitLine::new().show(false))
                .name("categories")
                .data(
                    freqs4.iter().map(|(e, _)| e.replace('&', "&\n")).collect(),
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

    let mut chart2 = chart.clone();
    let mut chart4 = chart;

    for (i, n) in names.iter().enumerate() {
        chart4 = chart4.series(Series::Bar(
            bar::Bar::new()
                .stack("bars")
                .item_style(ItemStyle::new().border_radius(50.0))
                .emphasis(Emphasis::new().focus(EmphasisFocus::Series))
                .tooltip(tooltip.clone())
                .name(n.to_string())
                .data(freqs4.iter().map(|(_, e)| e[i]).collect()),
        ))
    }

    for (i, n) in [names[0], names[2]].iter().enumerate() {
        chart2 = chart2.series(Series::Bar(
            bar::Bar::new()
                .stack("spon")
                .item_style(ItemStyle::new().border_radius(50.0))
                .emphasis(Emphasis::new().focus(EmphasisFocus::Series))
                .tooltip(tooltip.clone())
                .name(n.to_string())
                .data(freqs2.iter().map(|(_, e)| e[i]).collect()),
        ))
    }

    for (i, n) in [names[1], names[3]].iter().enumerate() {
        chart2 = chart2.series(Series::Bar(
            bar::Bar::new()
                .stack("nspon")
                .item_style(ItemStyle::new().border_radius(50.0))
                .emphasis(Emphasis::new().focus(EmphasisFocus::Series))
                .tooltip(tooltip.clone())
                .name(n.to_string())
                .data(freqsi.iter().map(|(_, e)| e[i]).collect()),
        ))
    }

    let cols = vec![COLOURS[12], COLOURS[10], COLOURS[4], COLOURS[5]];
    chart4 = chart4
        .legend(Legend::new().data(names.to_vec()))
        .color(cols);

    let cols = vec![COLOURS[12], COLOURS[4], COLOURS[10], COLOURS[5]];
    chart2 = chart2
        .legend(Legend::new().data(names.to_vec()))
        .color(cols);

    let mut renderer =
        HtmlRenderer::new("active", 900, 600).theme(Theme::Custom(
            "sheNaNigans",
            include_str!("../../theme/sheNaNigans.js"),
        ));
    renderer.save(&chart4, "active4.html").unwrap();

    let mut renderer =
        HtmlRenderer::new("active", 900, 600).theme(Theme::Custom(
            "sheNaNigans",
            include_str!("../../theme/sheNaNigans.js"),
        ));
    renderer.save(&chart2, "active2.html").unwrap();
}
