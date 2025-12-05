import pandas as pd
from pathlib import Path
from typing import Dict, List
import json


datapath = Path("../../dataset")
output_path = Path.cwd() 


def aggregate_metric_by_category(df: pd.DataFrame, metric_col: str) -> pd.Series:
    """
    Aggregate using the following strategy: compute median of metric (per 
    channel in each category) and then compute median of these values per 
    category.
    """

    # median per (category, channel)
    per_channel = (
        df.groupby(["categories", "channel_id"])[metric_col]
        .median()
        .reset_index()
    )

    # median of previously computed medians, per category
    agg = (
        per_channel.groupby("categories")[metric_col]
        .median()
        .sort_values(ascending=False)
    )

    return agg


def generate_second_plot_data(df: pd.DataFrame) -> Dict:
    """
    Construct a dict that contains all the neccessary data for the second plot.
    """
    
    df = df.copy()

    assert df["sponsored"].dtype == bool

    cat_order = sorted(df["categories"].dropna().unique())
    # drop "Shows" category to keep only the 15 of interest
    cat_order.remove("Shows")

    # separate sponsored and non-sponsored videos
    df_non = df[~df["sponsored"]].copy()  
    df_spon = df[df["sponsored"]].copy()  

    def agg_reorder(df_agg: pd.DataFrame, metric_col: str) -> List[float]:
        """
        Applies aggregation as in `aggregate_metric_by_category` and reindex with 
        chosen category order.
        """
        assert not df_agg.empty

        agg_series = aggregate_metric_by_category(df_agg, metric_col=metric_col)

        agg_series = agg_series.reindex(cat_order)
        return agg_series.tolist()

    
    # like/dislike ratio
    ratio_non = agg_reorder(df_non, "like_dislike_ratio")
    ratio_spon = agg_reorder(df_spon, "like_dislike_ratio")

    # duration
    duration_non = agg_reorder(df_non, "duration")
    duration_spon = agg_reorder(df_spon, "duration")

    # view count
    views_non = agg_reorder(df_non, "view_count")
    views_spon = agg_reorder(df_spon, "view_count")

    second_plot_data = {
        "categories": cat_order,
        "metrics": {
            "ratio": {
                "non_sponsored": ratio_non,
                "sponsored": ratio_spon,
            },
            "duration": {
                "non_sponsored": duration_non,
                "sponsored": duration_spon,
            },
            "views": {
                "non_sponsored": views_non,
                "sponsored": views_spon,
            },
        },
    }

    return second_plot_data


if __name__ == "__main__":
    df = pd.read_csv(
        datapath / "data_rq3_second_plot_10.csv.gz",
        compression="gzip"
    )

    second_plot_data = generate_second_plot_data(df)

    output_json_path = output_path / "rq3_second_plot_data.json"
    with open(output_json_path, "w", encoding="utf-8") as f:
        json.dump(second_plot_data, f, ensure_ascii=False, indent=2)

    print(f"data for second plot of rq3 saved to {output_json_path}")
