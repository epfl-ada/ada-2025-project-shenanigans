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

    # main title
    main_title = (
        "Weekly mean log(delta_metric + 1) around first sponsor appearance, "
        "averaged across multiple channels for each week"
    )

    # y-axis labels for the two metrics
    y_label_views = "Mean log(delta_views + 1)"
    y_label_subs = "Mean log(delta_subs + 1)"

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
    main_title_json = json.dumps(main_title)
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
      --bg: #020617;
      --panel: rgba(15, 23, 42, 0.85);
      --text: #e5e7eb;
      --muted: #94a3b8;
      --border: rgba(148, 163, 184, 0.3);
      --accent: #38bdf8;
      --pill-radius: 999px;
      --panel-radius: 14px;
      --font: "Inter", "Segoe UI", system-ui, -apple-system, BlinkMacSystemFont, sans-serif;
    }}

    * {{
      box-sizing: border-box;
    }}

    body {{
      margin: 0;
      padding: 24px;
      background: var(--bg);
      color: var(--text);
      font-family: var(--font);
      overflow-x: hidden;
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 12px;
    }}

    h1 {{
      margin: 0;
      font-size: 18px;
      font-weight: 600;
      text-align: center;
    }}

    .controls {{
      display: flex;
      gap: 10px;
      margin-bottom: 8px;
      margin-top: 6px;
      flex-wrap: wrap;
      justify-content: center;
    }}

    .toggle-button {{
      padding: 6px 14px;
      border-radius: var(--pill-radius);
      border: 1px solid var(--border);
      background: rgba(15, 23, 42, 0.7);
      color: var(--text);
      font-size: 12px;
      cursor: pointer;
      transition: all 0.18s ease;
    }}

    .toggle-button:hover {{
      transform: translateY(-1px);
      border-color: rgba(148, 163, 184, 0.65);
    }}

    .toggle-button.active {{
      background: var(--accent);
      color: #0b1120;
      box-shadow: 0 6px 20px rgba(56, 189, 248, 0.25);
      font-weight: 600;
    }}

    .chart-card {{
      background: var(--panel);
      border: 1px solid var(--border);
      border-radius: var(--panel-radius);
      padding: 14px;
      width: min(1200px, 100%);
      height: clamp(420px, 60vh, 640px);
      display: flex;
      flex-direction: column;
    }}

    canvas {{
      width: 100% !important;
      height: 100% !important;
      flex: 1;
      display: block;
    }}
  </style>
</head>
<body>
  <h1 id="mainTitle"></h1>

  <div class="controls">
    <button id="btnViews" class="toggle-button active">views</button>
    <button id="btnSubs" class="toggle-button">subscribers</button>
  </div>

  <div class="chart-card">
    <canvas id="rq3Chart"></canvas>
  </div>

  <script>
    const viewsData = {views_json};
    const subsData = {subs_json};

    const mainTitle = {main_title_json};
    const yLabelViews = {y_label_views_json};
    const yLabelSubs = {y_label_subs_json};

    const yMinViews = {y_min_views};
    const yMaxViews = {y_max_views};
    const yMinSubs  = {y_min_subs};
    const yMaxSubs  = {y_max_subs};

    const yLabelPosViews = {y_label_pos_views};
    const yLabelPosSubs  = {y_label_pos_subs};

    document.getElementById('mainTitle').textContent = mainTitle;

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

    const snapHoverPlugin = {{
      id: 'snapHoverPlugin',
      afterEvent(chart, args) {{
        const event = args.event;
        const xScale = chart.scales.x;
        if (!xScale) return;

        function startLineAnimation() {{
          if (chart.$lineRaf) {{
            cancelAnimationFrame(chart.$lineRaf);
          }}

          const animate = () => {{
            if (chart.$snapX == null || chart.$snapXTarget == null) {{
              chart.$lineRaf = null;
              return;
            }}

            const dx = chart.$snapXTarget - chart.$snapX;

            if (Math.abs(dx) < 0.5) {{
              chart.$snapX = chart.$snapXTarget;
              chart.$lineRaf = null;
              chart.draw();
              return;
            }}

            chart.$snapX += dx * 0.25;
            chart.draw();
            chart.$lineRaf = requestAnimationFrame(animate);
          }};

          chart.$lineRaf = requestAnimationFrame(animate);
        }}

        if (event.type === 'mousemove') {{
          const area = chart.chartArea;
          if (!area) return;

          if (event.x < area.left || event.x > area.right ||
              event.y < area.top  || event.y > area.bottom) {{
            chart.$snapX = null;
            chart.$snapXTarget = null;
            if (chart.$lineRaf) {{
              cancelAnimationFrame(chart.$lineRaf);
              chart.$lineRaf = null;
            }}
            chart.setActiveElements([]);
            chart.tooltip.setActiveElements([], {{ x: 0, y: 0 }});
            return;
          }}

          const xValue = xScale.getValueForPixel(event.x);
          if (xValue == null || isNaN(xValue)) return;

          const data = chart.data.datasets[2].data;
          if (!data || !data.length) return;

          let nearestIndex = 0;
          let minDist = Math.abs(xValue - data[0].x);
          for (let i = 1; i < data.length; i++) {{
            const d = Math.abs(xValue - data[i].x);
            if (d < minDist) {{
              minDist = d;
              nearestIndex = i;
            }}
          }}

          const meta = chart.getDatasetMeta(2);
          const elem = meta.data[nearestIndex];
          if (!elem) return;

          if (chart.$snapX == null) {{
            chart.$snapX = elem.x;
          }}

          chart.$snapXTarget = elem.x;
          startLineAnimation();

          chart.setActiveElements([{{ datasetIndex: 2, index: nearestIndex }}]);
          chart.tooltip.setActiveElements(
            [{{ datasetIndex: 2, index: nearestIndex }}],
            {{ x: elem.x, y: event.y }}
          );

        }} else if (event.type === 'mouseout') {{
          chart.$snapX = null;
          chart.$snapXTarget = null;
          if (chart.$lineRaf) {{
            cancelAnimationFrame(chart.$lineRaf);
            chart.$lineRaf = null;
          }}
          chart.setActiveElements([]);
          chart.tooltip.setActiveElements([], {{ x: 0, y: 0 }});
        }}
      }}
    }};

    const hoverLinePlugin = {{
      id: 'hoverLinePlugin',
      afterDatasetsDraw(chart) {{
        const x = chart.$snapX;
        if (x == null) return;

        const {{ ctx, chartArea: {{ top, bottom }} }} = chart;

        ctx.save();
        ctx.beginPath();
        ctx.moveTo(x, top);
        ctx.lineTo(x, bottom);
        ctx.lineWidth = 1;
        ctx.setLineDash([2, 2]);
        ctx.strokeStyle = '#38bdf8';
        ctx.stroke();
        ctx.restore();
      }}
    }};

    const ctx = document.getElementById('rq3Chart').getContext('2d');

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
            backgroundColor: 'rgba(148, 163, 184, 0.25)',
          }},
          {{
            label: yLabelViews,
            data: initial.meanPoints,
            borderColor: '#38bdf8',
            borderWidth: 2,
            pointRadius: 3,
            pointHoverRadius: 7,
            pointBackgroundColor: '#38bdf8',
            pointBorderColor: '#38bdf8',
            pointHoverBackgroundColor: '#38bdf8',
            pointHoverBorderColor: '#e5e7eb',
            pointHoverBorderWidth: 2,
            tension: 0,
          }}
        ]
      }},
      options: {{
        responsive: true,
        maintainAspectRatio: false,
        interaction: {{
          mode: 'x',
          intersect: false,
          axis: 'xy'
        }},
        plugins: {{
          legend: {{
            display: true,
            position: 'bottom',
            labels: {{
              color: '#e5e7eb',
              font: {{
                size: 12
              }},
              filter: function(item) {{
                return item.text !== 'CI upper';
              }},
              usePointStyle: true,
              pointStyle: function(context) {{
                return context.datasetIndex === 2 ? 'circle' : 'rect';
              }},
            }}
          }},
          tooltip: {{
            backgroundColor: 'rgba(15, 23, 42, 0.96)',
            titleColor: '#e5e7eb',
            bodyColor: '#e5e7eb',
            borderColor: 'rgba(148, 163, 184, 0.6)',
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
          annotation: {{
            annotations: {{
              vline: {{
                type: 'line',
                xMin: 0,
                xMax: 0,
                yMin: yMinViews,
                yMax: yMaxViews,
                borderColor: '#38bdf8',
                borderWidth: 3,
                borderDash: [5, 5]
              }},
              vlineLabel: {{
                type: 'label',
                xValue: 0,
                yValue: yLabelPosViews,
                backgroundColor: 'rgba(15, 23, 42, 0.95)',
                borderWidth: 1,
                borderColor: 'rgba(148, 163, 184, 0.6)',
                borderRadius: 6,
                color: '#e5e7eb',
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
              color: '#e5e7eb',
              font: {{
                size: 13
              }}
            }},
            ticks: {{
              color: '#94a3b8',
              callback: function(value) {{
                const validTicks = [-25, -20, -15, -10, -5, 0, 5, 10, 15, 20, 25];
                if (validTicks.includes(value)) {{
                  return value;
                }}
                return '';
              }}
            }},
            grid: {{
              color: 'rgba(15, 23, 42, 0.85)',
              lineWidth: 1
            }}
          }},
          y: {{
            min: yMinViews,
            max: yMaxViews,
            title: {{
              display: true,
              text: yLabelViews,
              color: '#e5e7eb',
              font: {{
                size: 13
              }}
            }},
            ticks: {{
              color: '#94a3b8'
            }},
            grid: {{
              color: 'rgba(15, 23, 42, 0.85)',
              lineWidth: 1
            }}
          }}
        }}
      }},
      plugins: [snapHoverPlugin, hoverLinePlugin]
    }});

    function updateChart(metricKey) {{
      const isViews = metricKey === 'views';
      const metricData = isViews ? viewsData : subsData;
      const labelY    = isViews ? yLabelViews : yLabelSubs;

      const pts = buildPoints(metricData);

      rq3Chart.data.datasets[0].data = pts.upperPoints;
      rq3Chart.data.datasets[1].data = pts.lowerPoints;
      rq3Chart.data.datasets[2].data = pts.meanPoints;
      rq3Chart.data.datasets[2].label = labelY;

      const yMin = isViews ? yMinViews : yMinSubs;
      const yMax = isViews ? yMaxViews : yMaxSubs;
      const yLabelPos = isViews ? yLabelPosViews : yLabelPosSubs;

      rq3Chart.options.scales.y.title.text = labelY;
      rq3Chart.options.scales.y.min = yMin;
      rq3Chart.options.scales.y.max = yMax;

      rq3Chart.options.plugins.annotation.annotations.vline.yMin = yMin;
      rq3Chart.options.plugins.annotation.annotations.vline.yMax = yMax;
      rq3Chart.options.plugins.annotation.annotations.vlineLabel.yValue = yLabelPos;

      rq3Chart.update();

      rq3Chart.$snapX = null;
      rq3Chart.$snapXTarget = null;
      if (rq3Chart.$lineRaf) {{
        cancelAnimationFrame(rq3Chart.$lineRaf);
        rq3Chart.$lineRaf = null;
      }}
      rq3Chart.setActiveElements([]);
      rq3Chart.tooltip.setActiveElements([], {{ x: 0, y: 0 }});
    }}

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
