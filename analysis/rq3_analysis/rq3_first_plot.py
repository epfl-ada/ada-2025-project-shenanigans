import numpy as np
import pandas as pd
from pathlib import Path
import json
from typing import Dict, List

datapath = Path("../../dataset")
html_path = Path("../../figures/rq3")

df = pd.read_csv(
    datapath / "data_rq3_first_plot.csv.gz",
    compression="gzip"
)


# gather/transform data for plotting

df_keep = df[df["delta_weeks"].between(-26, 26, inclusive="both")].copy()

print("total number of channels considered: ", df_keep["channel_id"].nunique())
print("total number of rows considered: ", df_keep.shape[0])


def generate_plot_data(df: pd.DataFrame, metric_col: str) -> Dict[str, List[float]]:
    """
    metric_col: "delta_views" or "delta_subs"
    Returns a dict with keys:
    - delta_weeks
    - mean_log_val
    - ci_lower
    - ci_upper
    """
    df_temp = df.copy()

    log_col = f"log_{metric_col}"
    df_temp[log_col] = np.log1p(df_temp[metric_col].clip(lower=0))
    

    # aggregate across channels considering the relative week
    df_plot = (
        df_temp.groupby("delta_weeks", as_index=False)
        .agg(
            mean_log_val=(log_col, "mean"),
            std_log_val=(log_col, "std"),
            n=(log_col, "size")
        )
        .sort_values("delta_weeks")
    )

    # standard error and 95% CI
    df_plot["se"] = df_plot["std_log_val"] / np.sqrt(df_plot["n"])
    df_plot["ci_lower"] = df_plot["mean_log_val"] - (1.96 * df_plot["se"])
    df_plot["ci_upper"] = df_plot["mean_log_val"] + (1.96 * df_plot["se"])

    return {
        "delta_weeks": df_plot["delta_weeks"].tolist(),
        "mean_log_val": df_plot["mean_log_val"].tolist(),
        "ci_lower": df_plot["ci_lower"].tolist(),
        "ci_upper": df_plot["ci_upper"].tolist(),
    }


def create_combined_html(
    data_views: Dict[str, List[float]], 
    data_subs: Dict[str, List[float]], 
    filename: Path,
    ) -> None:
    """
    Create HTML file with data for "delta_views" and "delta_subs".
    """

    # titles and labels of the two metrics
    title_views = "Weekly mean log(delta_views + 1) around first sponsor appearance, averaged across multiple channels for each week"
    title_subs = "Weekly mean log(delta_subs + 1) around first sponsor appearance, averaged across multiple channels for each week"
    y_label_views = "log(delta_views + 1)"
    y_label_subs = "log(delta_subs + 1)"

    # limits along y-axis and label position of vertical line (at weeks=0)
    y_min_views = 10.2
    y_max_views = 11.4
    y_min_subs = 5.0
    y_max_subs = 6.2

    y_label_pos_views = 11.3
    y_label_pos_subs = 6.1

    # serialize to use in HTML 
    views_json = json.dumps(data_views)
    subs_json = json.dumps(data_subs)
    title_views_json = json.dumps(title_views)
    title_subs_json = json.dumps(title_subs)
    y_label_views_json = json.dumps(y_label_views)
    y_label_subs_json = json.dumps(y_label_subs)

    html_content = f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <title>RQ3 - views and subs</title>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/chartjs-plugin-annotation@1.4.0"></script>
  <style>
    :root {{
      /* Rosé Pine theme */  
      --rp-base:   #191724;
      --rp-surface:#1f1d2e;
      --rp-overlay:#26233a;
      --rp-muted:  #6e6a86;
      --rp-subtle: #908caa;
      --rp-text:   #e0def4;
      --rp-love:   #eb6f92;
      --rp-gold:   #f6c177;
      --rp-rose:   #ebbcba;
      --rp-pine:   #31748f;
      --rp-foam:   #9ccfd8;
      --rp-iris:   #c4a7e7;
    }}

    body {{
      margin: 0;
      padding: 0;
      width: 100vw;
      height: 100vh;
      background: var(--rp-base);
      color: var(--rp-text);
      font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      display: flex;
      flex-direction: column;
      justify-content: flex-start;
      align-items: center;
      box-sizing: border-box;
    }}

    .controls {{
      margin-top: 16px;
      margin-bottom: 8px;
      display: flex;
      gap: 8px;
    }}

    .toggle-button {{
      padding: 6px 12px;
      border-radius: 999px;
      border: 1px solid var(--rp-muted);
      background: var(--rp-surface);
      color: var(--rp-text);
      font-size: 13px;
      cursor: pointer;
      transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
    }}

    .toggle-button.active {{
      background: var(--rp-foam);
      color: var(--rp-base);
      border-color: var(--rp-foam);
      font-weight: 600;
    }}

    .chart-container {{
      width: 95vw;
      height: 80vh;
      position: relative;
      margin-bottom: 16px;
    }}

    canvas {{
      width: 100%;
      height: 100%;
      display: block;
    }}
  </style>
</head>
<body>
  <div class="controls">
    <button id="btnViews" class="toggle-button active">Δ views</button>
    <button id="btnSubs" class="toggle-button">Δ subs</button>
  </div>

  <div class="chart-container">
    <canvas id="rq3Chart"></canvas>
  </div>

  <script>
    // data from Python
    const viewsData = {views_json};
    const subsData = {subs_json};

    const titleViews = {title_views_json};
    const titleSubs = {title_subs_json};
    const yLabelViews = {y_label_views_json};
    const yLabelSubs = {y_label_subs_json};

    // y-axis limits and label placement for vertical line at weeks=0
    const yMinViews = {y_min_views};
    const yMaxViews = {y_max_views};
    const yMinSubs  = {y_min_subs};
    const yMaxSubs  = {y_max_subs};

    const yLabelPosViews = {y_label_pos_views};
    const yLabelPosSubs  = {y_label_pos_subs};

    // helper to construct points from data
    function buildPoints(metricData) {{
      const deltaWeeks = metricData.delta_weeks;
      const meanLogVal = metricData.mean_log_val;
      const ciLower    = metricData.ci_lower;
      const ciUpper    = metricData.ci_upper;

      const upperPoints = deltaWeeks.map((x, i) => ({{ x: x, y: ciUpper[i] }}));
      const lowerPoints = deltaWeeks.map((x, i) => ({{ x: x, y: ciLower[i] }}));
      const meanPoints  = deltaWeeks.map((x, i) => ({{ x: x, y: meanLogVal[i] }}));

      return {{
        deltaWeeks,
        meanLogVal,
        upperPoints,
        lowerPoints,
        meanPoints
      }};
    }}

    const ctx = document.getElementById('rq3Chart').getContext('2d');

    // we land on delta_views plot by default
    let currentMetric = 'views';
    let initial = buildPoints(viewsData);

    const rq3Chart = new Chart(ctx, {{
      type: 'line',
      data: {{
        datasets: [
          {{
            label: 'CI upper',
            data: initial.upperPoints,
            borderWidth: 0,
            pointRadius: 0,
            fill: false,
          }},
          {{
            label: '95% CI',
            data: initial.lowerPoints,
            borderWidth: 0,
            pointRadius: 0,
            fill: {{ target: '-1' }},
            backgroundColor: 'rgba(110, 106, 134, 0.3)', 
          }},
          {{
            label: yLabelViews,
            data: initial.meanPoints,
            borderColor: '#9ccfd8', 
            borderWidth: 2,
            pointRadius: 4,
            pointHoverRadius: 7,
            pointBackgroundColor: '#9ccfd8',
            pointBorderColor: '#9ccfd8',
            pointHoverBackgroundColor: '#9ccfd8',
            pointHoverBorderColor: '#e0def4',
            pointHoverBorderWidth: 2,
            tension: 0,
          }}
        ]
      }},
      options: {{
        responsive: true,
        maintainAspectRatio: false,
        interaction: {{
          mode: 'nearest',
          intersect: false,
          axis: 'xy'
        }},
        plugins: {{
          legend: {{
            display: true,
            position: 'bottom',
            labels: {{
              color: '#e0def4',
              font: {{
                size: 12
              }},
              filter: function(item) {{
                // On cache l'entrée "CI upper"
                return item.text !== 'CI upper';
              }},
              usePointStyle: true,
              pointStyle: function(context) {{
                // Mean line: point rond; CI: rectangle
                return context.datasetIndex === 2 ? 'circle' : 'rect';
              }},
            }}
          }},
          tooltip: {{
            backgroundColor: '#26233a',
            titleColor: '#e0def4',
            bodyColor: '#e0def4',
            borderColor: '#6e6a86',
            borderWidth: 1,
            displayColors: false,
            padding: 10,
            bodyFont: {{
              size: 13
            }},
            callbacks: {{
              title: function() {{
                return '';
              }},
              label: function(context) {{
                // only for mean line
                if (context.datasetIndex === 2) {{
                  const week = context.parsed.x;
                  const value = context.parsed.y.toFixed(3);
                  const label = context.dataset.label || '';
                  return [`Week: ${{week}}`, `${{label}}: ${{value}}`];
                }}
                return null;
              }}
            }},
            filter: function(tooltipItem) {{
              return tooltipItem.datasetIndex === 2;
            }}
          }},
          title: {{
            display: true,
            text: titleViews,
            color: '#e0def4',
            font: {{
              size: 14
            }},
            padding: {{
              top: 10,
              bottom: 20
            }}
          }},
          annotation: {{
            annotations: {{
              vline: {{
                type: 'line',
                xMin: 0,
                xMax: 0,
                yMin: yMinViews,
                yMax: yMaxViews,
                borderColor: '#eb6f92',
                borderWidth: 2,
                borderDash: [5, 5]
              }},
              vlineLabel: {{
                type: 'label',
                xValue: 0,
                yValue: yLabelPosViews,
                backgroundColor: 'rgba(31, 29, 46, 0.8)',
                borderWidth: 1,
                borderColor: '#6e6a86',
                borderRadius: 4,
                color: '#e0def4',
                content: ['First sponsor appearance (week 0)'],
                font: {{
                  size: 11
                }},
                padding: 6
              }}
            }}
          }}
        }},
        scales: {{
          x: {{
            type: 'linear',
            min: -26,
            max: 26,
            title: {{
              display: true,
              text: 'Weeks relative to first sponsor',
              color: '#e0def4',
              font: {{
                size: 13
              }}
            }},
            ticks: {{
              color: '#908caa',
              callback: function(value) {{
                const validTicks = [-25, -20, -15, -10, -5, 0, 5, 10, 15, 20, 25];
                if (validTicks.includes(value)) {{
                  return value;
                }}
                return '';
              }}
            }},
            grid: {{
              color: '#26233a',
              lineWidth: 1
            }}
          }},
          y: {{
            min: yMinViews,
            max: yMaxViews,
            title: {{
              display: true,
              text: yLabelViews,
              color: '#e0def4',
              font: {{
                size: 13
              }}
            }},
            ticks: {{
              color: '#908caa'
            }},
            grid: {{
              color: '#26233a',
              lineWidth: 1
            }}
          }}
        }}
      }}
    }});

    // function to update plot when changing metric
    function updateChart(metricKey) {{
      const isViews = metricKey === 'views';
      const metricData = isViews ? viewsData : subsData;
      const labelY    = isViews ? yLabelViews : yLabelSubs;
      const title     = isViews ? titleViews : titleSubs;

      const pts = buildPoints(metricData);

      rq3Chart.data.datasets[0].data = pts.upperPoints;
      rq3Chart.data.datasets[1].data = pts.lowerPoints;
      rq3Chart.data.datasets[2].data = pts.meanPoints;
      rq3Chart.data.datasets[2].label = labelY;

      const yMin = isViews ? yMinViews : yMinSubs;
      const yMax = isViews ? yMaxViews : yMaxSubs;
      const yLabelPos = isViews ? yLabelPosViews : yLabelPosSubs;

      rq3Chart.options.plugins.title.text = title;
      rq3Chart.options.scales.y.title.text = labelY;
      rq3Chart.options.scales.y.min = yMin;
      rq3Chart.options.scales.y.max = yMax;

      rq3Chart.options.plugins.annotation.annotations.vline.yMin = yMin;
      rq3Chart.options.plugins.annotation.annotations.vline.yMax = yMax;
      rq3Chart.options.plugins.annotation.annotations.vlineLabel.yValue = yLabelPos;

      rq3Chart.update();
    }}

    // buttons to toggle between views and subs
    const btnViews = document.getElementById('btnViews');
    const btnSubs  = document.getElementById('btnSubs');

    btnViews.addEventListener('click', () => {{
      if (currentMetric !== 'views') {{
        currentMetric = 'views';
        btnViews.classList.add('active');
        btnSubs.classList.remove('active');
        updateChart('views');
      }}
    }});

    btnSubs.addEventListener('click', () => {{
      if (currentMetric !== 'subs') {{
        currentMetric = 'subs';
        btnSubs.classList.add('active');
        btnViews.classList.remove('active');
        updateChart('subs');
      }}
    }});
  </script>
</body>
</html>"""

    with open(filename, "w", encoding="utf-8") as f:
        f.write(html_content)
    print(f"{filename} has been generated")


# generate HTML file
if __name__ == "__main__":
    data_views = generate_plot_data(df_keep, "delta_views")
    data_subs = generate_plot_data(df_keep, "delta_subs")

    create_combined_html(
        data_views,
        data_subs,
        filename=html_path / "rq3_delta_views_subs.html"
    )
