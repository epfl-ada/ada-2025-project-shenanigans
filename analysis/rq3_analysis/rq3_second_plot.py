import numpy as np
import pandas as pd
import plotly.graph_objects as go
from pathlib import Path
from theme import use_shenanigans_template
from typing import Tuple


# set the theme for the plots
use_shenanigans_template()

datapath = Path("../../dataset")
html_path = Path("../../figures/rq3")


def box_stats(values: pd.Series) -> Tuple[float, float, float, float, float]:
    """Provide stats of a given list of values to construct the boxplot."""
    vals = values[np.isfinite(values) & (values > 0)]            
    if vals.size == 0:
        return (np.nan, np.nan, np.nan, np.nan, np.nan)
    q1, median, q3 = np.quantile(vals, [0.25, 0.5, 0.75])
    low = vals.min()
    high = vals.max()
    return (q1, median, q3, low, high)


def second_plot_rq3(
    df: pd.DataFrame,
    value_of_interest: str,
    html_path: Path
) -> None:
    """
    Create and save the plot for the second plot of RQ3.
    
    Parameters
    ----------
    df: pandas DataFrame used for the plot
    value_of_interest: value for the y-axis (either "view_count" or "duration" or "like_dislike_ratio")
    html_path: path where to save the resulting html file

    Returns
    -------
    None
    """
    assert value_of_interest in {"view_count", "duration", "like_dislike_ratio"}
    
    
    ### gather/transform data for plotting

    sponsored = df[df["sponsored"] == 1][value_of_interest]
    not_sponsored = df[df["sponsored"] == 0][value_of_interest]
    
    print(f"df shape: {df.shape}")
    print("df sponsored count: {}".format(df["sponsored"].sum()))
    print("number channels: {}".format(df["channel_id"].nunique()))
    
    print(f"sponsored shape: {sponsored.shape}")
    print(f"not_spsonsored shape: {not_sponsored.shape}")
    
    q1_sp, median_sp, q3_sp, low_sp, high_sp = box_stats(sponsored)
    q1_not, median_not, q3_not, low_not, high_not = box_stats(not_sponsored)
    
    
    ### plotting
    
    fig = go.Figure()
    
    labels = ["yes", "no"]
    
    fig.add_trace(go.Box(
        name="sponsored",
        x=labels,
        q1=[q1_sp, q1_not],
        median=[median_sp, median_not],
        q3=[q3_sp, q3_not],
        lowerfence=[low_sp, low_not],
        upperfence=[high_sp, high_not],
        boxpoints=False
    ))
    
    fig.update_layout(
        boxmode="group",
        title = "Boxplots of videos (>100 views) with and without patreon/paypal urls considering view count, per category",
        xaxis_title="sponsored",
        yaxis_title=value_of_interest,
        showlegend=True
    )
        
    fig.update_yaxes(type="log") # log scale in y axis for readability

    
    
    fig.show()
    # plot(fig, filename=html_path, auto_open=False, include_plotlyjs="cdn")


if __name__ == "__main__":
    df = pd.read_csv(
        datapath / "data_rq3_second_plot_10.csv.gz",
        compression="gzip"
    )
        
    html_path_bp_views = html_path / "rq3_boxplot_views.html"
    html_path_bp_duration = html_path / "rq3_boxplot_duration.html"
    html_path_bp_likes = html_path / "rq3_boxplot_likes.html"
    
    # plot considering view_count
    second_plot_rq3(
        df=df, 
        value_of_interest="view_count", 
        html_path=str(html_path_bp_views)
        )
    
    # plot considering duration
    second_plot_rq3(
        df=df, 
        value_of_interest="duration", 
        html_path=str(html_path_bp_duration)
        )
    
    # plot considering like_dislike_ratio
    second_plot_rq3(
        df=df, 
        value_of_interest="like_dislike_ratio", 
        html_path=str(html_path_bp_likes)
        )
