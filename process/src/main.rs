use polars::prelude::*;
use std::num::NonZero;

fn main() {
    let lazy_comments = LazyFrame::scan_parquet(
        PlPath::new("../../dataset/my.parquet"),
        ScanArgsParquet::default(),
    )
    .unwrap()
    .tail(10_000_000);

    let lazy_videos = LazyFrame::scan_parquet(
        PlPath::new("../../dataset/yt_metadata_helper.parquet"),
        ScanArgsParquet::default(),
    )
    .unwrap()
    .select([col("display_id"), col("channel_id"), col("view_count")])
    .filter(col("view_count").gt(500_000.0))
    .select([col("display_id"), col("channel_id")]);

    let res = lazy_comments
        .join(
            lazy_videos,
            [col("video_id")],
            [col("display_id")],
            JoinArgs::default(),
        )
        .select([col("author"), col("channel_id")])
        .unique(None, UniqueKeepStrategy::Any);

    let res_clone = res.clone().rename(
        ["author", "channel_id"],
        ["author_clone", "channel_id_clone"],
        true,
    );
    // .unique(
    //     Some(Selector::ByName {
    //         names: Arc::new(["author_clone".into()]),
    //         strict: true,
    //     }),
    //     UniqueKeepStrategy::Any,
    // );

    let fin = res_clone
        .join_builder()
        .with(res)
        .join_where(vec![
            col("author").eq(col("author_clone")),
            col("channel_id").neq(col("channel_id_clone")),
        ])
        .select([col("channel_id"), col("channel_id_clone")])
        .group_by([col("channel_id"), col("channel_id_clone")])
        .agg([len()])
        .filter(col("len").gt(10))
        .drop(Selector::ByName {
            names: Arc::new(["len".into()]),
            strict: true,
        });

    // println!("{}", fin.clone().collect().unwrap());

    let mut csv_args = CsvWriterOptions::default();
    csv_args.batch_size = NonZero::new(1024usize).unwrap();
    csv_args.include_header = false;
    csv_args.serialize_options.separator = b' ';

    let mut sink_args = SinkOptions::default();
    sink_args.maintain_order = false;

    let _ = fin
        .sink_csv(
            SinkTarget::Path(PlPath::new("graph.csv")),
            csv_args,
            None,
            sink_args,
        )
        .unwrap()
        .collect();

    println!("Finished generating graph!");
}
