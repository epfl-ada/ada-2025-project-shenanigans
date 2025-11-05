from pathlib import Path
import pandas as pd
import re
import plotly.graph_objects as go
from plotly.offline import plot


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

datapath = Path("data")
plots_path = Path("plots")

money_income_re = re.compile(
    r"""(?ix)                           
    \b
    (?:https?://)?                     
    (?:www\.)?                          
    (?:[a-z0-9-]+\.)*                  
    (?:                               
      patreon\.com
      | paypal\.com
      | paypal\.me
    )
    (?:[/:?#]\S*)?                      
    \b
    """,
)

chunks = pd.read_json(
    datapath / "urls_cat.jsonl.gz",
    lines=True,
    chunksize=1_000_000,
)

counter = {
    "": [0, 0],
    "Autos & Vehicles": [0, 0],
    "Comedy": [0, 0],
    "Education": [0, 0],
    "Entertainment": [0, 0],
    "Film & Animation": [0, 0],
    "Gaming": [0, 0],
    "Howto & Style": [0, 0],
    "Music": [0, 0],
    "News & Politics": [0, 0],
    "Nonprofits & Activism": [0, 0],
    "People & Blogs": [0, 0],
    "Pets & Animals": [0, 0],
    "Science & Technology": [0, 0],
    "Sports": [0, 0],
    "Travel & Events": [0, 0],
}

for i, chunk in enumerate(chunks):
    print("chunk number ", i)
    for _, vid_id, urls, cat_id in chunk.itertuples():
        if not (isinstance(cat_id, int) and 0 <= cat_id <= 15): 
            continue
        cat = categories[cat_id]
        counter[cat][0] += 1
        if not isinstance(urls, list):
            continue
        if not any(money_income_re.search((url or "")) for url in urls):
            continue
        counter[cat][1] += 1
        
print("counter: ", counter)
percentages = {key: (val2/val1)*100 for key, (val1, val2) in counter.items()}
print("percentages: ", percentages)

del percentages[""] # remove empty category 

items_sorted = sorted(percentages.items(), key=lambda item: item[1], reverse=True)
categories_plot = [cat for cat, _ in items_sorted]
values = [val for _, val in items_sorted]

# set x-axis limits for better readability
xmax = max(values) * 1.1
xaxis_layout = dict(range=[0, xmax], title="Percentage (%)")

# plotting
fig = go.Figure(
    data=[
        go.Bar(
            x=values,                 
            y=categories_plot,
            orientation="h",
            text=[f"{val:.2f}%" for val in values],
            textposition="auto",
            hovertemplate="%{y}: %{x:.2f}%<extra></extra>"
        )
    ],
    layout=go.Layout(
        title="Percentage of Videos with Money-related URLs by YouTube Category",
        xaxis=xaxis_layout,
        yaxis_title="Category",
        yaxis=dict(autorange="reversed"),
        margin=dict(l=160, r=40, t=60, b=60)
    )
)

html_path = plots_path / "money_percentages_per_cat.html"
plot(fig, filename=html_path, auto_open=False, include_plotlyjs="cdn")
