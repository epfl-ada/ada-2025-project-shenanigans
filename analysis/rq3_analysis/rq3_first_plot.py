import numpy as np
import pandas as pd
import plotly.graph_objects as go
from pathlib import Path
from plotly.offline import plot
from theme import use_shenanigans_template


# set the theme for the plots
use_shenanigans_template()

datapath = Path("../../dataset")
html_path = Path("../../figures/rq3")


def first_plot_rq3(
    df: pd.DataFrame,
    value_of_interest: str,
    html_path: Path
) -> None:
    """
    Create and save the plot for the first subquestion of RQ3.
    
    Parameters
    ----------
    df: pandas DataFrame used for the plot
    value_of_interest: value for the y-axis (either "delta_views" or "delta_subs")
    html_path: path where to save the resulting html file

    Returns
    -------
    None
    """
    assert value_of_interest == "delta_views" or "delta_subs"
    
    
    ### gather/transform data for plotting

    df_keep = df[df["delta_weeks"].between(-26, 26, inclusive="both")].copy()
    
    log_val = f"log_{value_of_interest}"

    df_keep[log_val] = np.log1p(df_keep[value_of_interest].clip(lower=0))

    print("total number of channels considered: ", df_keep["channel_id"].nunique())

    # aggregate across channels considering the relative week
    df_plot = (
        df_keep.groupby("delta_weeks", as_index=False)
        .agg(
            mean_log_val=(log_val, "mean"),
            std_log_val=(log_val, "std"),
            n=(log_val, "size")
        )
        .sort_values("delta_weeks")
    )
    
    # standard error and 95% CI
    df_plot["se"] = df_plot["std_log_val"] / np.sqrt(df_plot["n"])   
    
    df_plot["ci_lower"] = df_plot["mean_log_val"] - (1.96 * df_plot["se"])
    df_plot["ci_upper"] = df_plot["mean_log_val"] + (1.96 * df_plot["se"])


    ### plotting
    
    fig = go.Figure()

    # plot lower and upper confidence intervals as a band around the mean
    fig.add_trace(
        go.Scatter(
            x=df_plot["delta_weeks"],
            y=df_plot["ci_upper"],
            mode="lines",
            line=dict(width=0),
            showlegend=False,
            hoverinfo="skip"
        )
    )

    fig.add_trace(
        go.Scatter(
            x=df_plot["delta_weeks"],
            y=df_plot["ci_lower"],
            mode="lines",
            line=dict(width=0),
            fill="tonexty",
            name="95% CI",
            hoverinfo="skip"
        )
    )

    # plot the mean line
    fig.add_trace(
        go.Scatter(
            x=df_plot["delta_weeks"],
            y=df_plot["mean_log_val"],
            mode="lines+markers",
            name=f"Mean log({value_of_interest} + 1)",
            hovertemplate=(
                "Week: %{x}<br>"  
                "Mean log(" + value_of_interest + " + 1): %{y:.3f}"  
                "<extra></extra>"  
            )
        )
    )
    
    # center x-axis around 0
    max_x = df_plot["delta_weeks"].max()
    fig.update_xaxes(
        range=[-max_x, max_x],
        title_text="Weeks relative to first sponsor",
        tickvals=[-25, -20, -15, -10, -5, 0, 5, 10, 15, 20, 25]
    )

    fig.update_yaxes(
        title_text=f"Mean log({value_of_interest} + 1)"
    )

    y_max = df_plot["mean_log_val"].max()

    vertical_line_color = "#ffffff"

    fig.add_vline(
        x=0,
        line_width=2,
        line_dash="solid",
        line_color=vertical_line_color
    )

    fig.add_annotation(
        x=0,
        y=y_max+0.05,
        xanchor="left",
        yanchor="bottom",
        text="First sponsor appearance (week 0)",
        showarrow=False,
        font=dict(color=vertical_line_color)
    )
    
    fig.update_layout(
        title = f"Weekly mean log({value_of_interest} + 1) around first sponsor appearance, averaged across multiple channels for each week"
    )

    plot(fig, filename=html_path, auto_open=False, include_plotlyjs="cdn")


if __name__ == "__main__":
    df = pd.read_csv(
        datapath / "data_rq3_first_plot.csv.gz",
        compression="gzip"
    )
    
    html_path_views = html_path / "rq3_vc.html"
    html_path_subs = html_path / "rq3_subs.html"
    
    # plot considering delta_views
    first_plot_rq3(
        df=df, 
        value_of_interest="delta_views", 
        html_path=str(html_path_views)
        )
    
    # plot considering delta_subs
    first_plot_rq3(
        df=df, 
        value_of_interest="delta_subs", 
        html_path=str(html_path_subs)
        )
