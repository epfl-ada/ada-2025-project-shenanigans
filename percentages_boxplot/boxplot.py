from pathlib import Path
import numpy as np
import pickle
import plotly.graph_objects as go
from plotly.offline import plot
from typing import List, Tuple

categories = [
    "",
    "Autos & Vehicles",
    "Comedy",
    "Education",
    "Entertainment",
    "Film & Animation",
    "Gaming",
    "Howto & Style",
    "Music",
    "News & Politics",
    "Nonprofits & Activism",
    "People & Blogs",
    "Pets & Animals",
    "Science & Technology",
    "Sports",
    "Travel & Events",
]

data_path = Path("data")
plots_path = Path("plots")

def box_stats(values: List[float]) -> Tuple[float, float, float, float, float]:
    """Provide stats of a given list of values to construct the boxplot."""
    vals = np.asarray(values, dtype=float)
    vals = vals[np.isfinite(vals) & (vals > 0)]            
    if vals.size == 0:
        return (np.nan, np.nan, np.nan, np.nan, np.nan)
    q1, median, q3 = np.quantile(vals, [0.25, 0.5, 0.75])
    low = vals.min()
    high = vals.max()
    return (q1, median, q3, low, high)

labels = [categories[i+1] for i in range(15)] # +1 to avoid taking empty cat into account

# initialise empty lists to store values used to create the boxplot
money_q1, money_median, money_q3, money_low, money_high = [], [], [], [], []
non_q1, non_median, non_q3, non_low, non_high = [], [], [], [], []

for i in range(15):
    print(f"Category number {i+1}")
    with open(data_path / f"money_vc_{i+1}.pkl", "rb") as f:
        data = pickle.load(f)
    money = data["money_vc"]
    non_money = data["non_money_vc"]
    money = [m for m in money if m >= 100] # select only video with more than 100 views
    non_money = [n for n in non_money if n >= 100] 
    q1, median, q3, low, high = box_stats(money)
    money_q1.append(q1); money_median.append(median); money_q3.append(q3); money_low.append(low); money_high.append(high)
    q1, median, q3, low, high = box_stats(non_money)
    non_q1.append(q1); non_median.append(median); non_q3.append(q3); non_low.append(low); non_high.append(high)


# plotting
fig = go.Figure()

fig.add_trace(go.Box(
    name="money", x=labels,
    q1=money_q1, median=money_median, q3=money_q3,
    lowerfence=money_low, upperfence=money_high,
    boxpoints=False
))
fig.add_trace(go.Box(
    name="non_money", x=labels,
    q1=non_q1, median=non_median, q3=non_q3,
    lowerfence=non_low, upperfence=non_high,
    boxpoints=False
))

fig.update_layout(
    boxmode="group",
    title = "Boxplots of videos (>100 views) with and without patreon/paypal urls considering view count, per category",
    xaxis_title="category",
    yaxis_title="view count",
    showlegend=True
)
fig.update_yaxes(type="log") # log scale in y axis for readability

html_path = plots_path / "boxplots_money_all_cats_100.html" 
plot(fig, filename=html_path, auto_open=False, include_plotlyjs="cdn")
