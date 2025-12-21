import json
from pathlib import Path
from typing import Dict

html_path = Path("../../figures/rq3")


def create_second_plot_rq3(
    second_plot_data: Dict,
    filename: Path,
) -> None:
    """
    Create an HTML file for the second plot of rq3 (dumbbell plot).
    """

    data_json = json.dumps(second_plot_data, ensure_ascii=False)

    title_main = "Dumbbell plot - sponsored vs nonsponsored videos for different metrics"

    title_main_json = json.dumps(title_main, ensure_ascii=False)

    html_content = f"""<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="UTF-8" />
  <title>RQ3 - dumbbell plot</title>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
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
      background: --bg;
      background: var(--bg);
      color: var(--text);
      font-family: var(--font);
      overflow-x: hidden;
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 10px;
    }}

    h1 {{
      margin: 0;
      font-size: 18px;
      font-weight: 600;
      text-align: center;
    }}

    h2 {{
      margin: 0;
      font-size: 13px;
      font-weight: 400;
      color: var(--muted);
      text-align: center;
    }}

    .controls {{
      display: flex;
      gap: 10px;
      margin-top: 10px;
      margin-bottom: 6px;
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
      padding: 12px;
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
    <button class="toggle-button active" data-metric="ratio">like/dislike ratio</button>
    <button class="toggle-button" data-metric="duration">duration</button>
    <button class="toggle-button" data-metric="views">view count</button>
  </div>

  <div class="chart-card">
    <canvas id="dumbbellChart"></canvas>
  </div>

  <script>
    const dumbbellData = {data_json};

    const mainTitle = {title_main_json};

    document.getElementById("mainTitle").textContent = mainTitle;

    const categories = dumbbellData.categories;

    const metricsConfig = {{
      ratio: {{
        label: "median like/dislike ratio",
        axisLabel: "like/dislike ratio [%]",
        unit: ""
      }},
      duration: {{
        label: "median duration",
        axisLabel: "duration [s]",
        unit: ""
      }},
      views: {{
        label: "median view count",
        axisLabel: "view count",
        unit: ""
      }}
    }};

    const metricData = {{
      ratio: {{
        nonSponsored: dumbbellData.metrics.ratio.non_sponsored,
        sponsored: dumbbellData.metrics.ratio.sponsored
      }},
      duration: {{
        nonSponsored: dumbbellData.metrics.duration.non_sponsored,
        sponsored: dumbbellData.metrics.duration.sponsored
      }},
      views: {{
        nonSponsored: dumbbellData.metrics.views.non_sponsored,
        sponsored: dumbbellData.metrics.views.sponsored
      }}
    }};

    let currentMetric = "ratio";
    let hoverIndex = null; 

    function buildDatasets(metricKey) {{
      const nonSponsored = metricData[metricKey].nonSponsored;
      const sponsored = metricData[metricKey].sponsored;

      const dataNon = nonSponsored.map((value, i) => ({{
        x: value,
        y: categories[i],
        category: categories[i],
        type: "Not sponsored"
      }}));

      const dataSpon = sponsored.map((value, i) => ({{
        x: value,
        y: categories[i],
        category: categories[i],
        type: "Sponsored"
      }}));

      return {{ dataNon, dataSpon }};
    }}

    const initial = buildDatasets(currentMetric);
    const ctx = document.getElementById("dumbbellChart").getContext("2d");

    const dumbbellChart = new Chart(ctx, {{
      type: "scatter",
      data: {{
        datasets: [
          {{
            label: "Not sponsored",
            data: initial.dataNon,
            backgroundColor: "#38bdf8",
            borderColor: "#bae6fd",
            pointRadius: 5,
            pointHoverRadius: 8
          }},
          {{
            label: "Sponsored",
            data: initial.dataSpon,
            backgroundColor: "#f472b6",
            borderColor: "#fce7f3",
            pointRadius: 5,
            pointHoverRadius: 8
          }}
        ]
      }},
      options: {{
        responsive: true,
        maintainAspectRatio: false,
        indexAxis: "y",
        interaction: {{
          mode: "nearest",
          intersect: false,
          axis: "xy"
        }},
        animation: {{
          duration: 700,
          easing: "easeOutCubic"
        }},
        plugins: {{
          legend: {{
            position: "bottom",
            labels: {{
              color: "#e5e7eb",
              font: {{ size: 12 }},
              usePointStyle: true,
              pointStyle: "circle"
            }}
          }},
          tooltip: {{
            backgroundColor: "rgba(15, 23, 42, 0.96)",
            borderColor: "rgba(148, 163, 184, 0.6)",
            borderWidth: 1,
            padding: 10,
            displayColors: false,
            titleColor: "#e5e7eb",
            bodyColor: "#e5e7eb",
            callbacks: {{
              title: ctx => {{
                const c = ctx[0].raw;
                return `${{c.category}} (${{c.type}})`;
              }},
              label: ctx => {{
                const chart = ctx.chart;
                const datasetIndex = ctx.datasetIndex;
                const dataIndex = ctx.dataIndex;

                const value = ctx.raw.x;
                const twinDatasetIndex = datasetIndex === 0 ? 1 : 0;
                const twinValue = chart.data.datasets[twinDatasetIndex].data[dataIndex].x;

                const conf = metricsConfig[currentMetric];
                const v = value.toFixed(2) + conf.unit;
                const vTwin = twinValue.toFixed(2) + conf.unit;

                const delta = (value - twinValue);
                const sign = delta >= 0 ? "+" : "";
                const deltaText = `${{sign}}${{delta.toFixed(2)}}${{conf.unit}}`;

                const otherLabel = twinDatasetIndex === 0 ? "Not sponsored" : "Sponsored";

                return [
                  `Value (${{ctx.raw.type}}) : ${{v}}`,
                  `Value (${{otherLabel}}) : ${{vTwin}}`,
                  `Difference : ${{deltaText}}`
                ];
              }}
            }}
          }}
        }},
        onHover: (event, _elements, chart) => {{
          const points = chart.getElementsAtEventForMode(
            event,
            "nearest",
            {{ intersect: false }},
            true
          );

          const canvas = chart.canvas;

          if (points.length > 0) {{
            hoverIndex = points[0].index;
            canvas.style.cursor = "pointer";
          }} else {{
            hoverIndex = null;
            canvas.style.cursor = "default";
          }}

          chart.draw();
        }},
        scales: {{
          y: {{
            type: "category",
            labels: categories,
            offset: true,
            grid: {{
              color: "rgba(15, 23, 42, 0.9)"
            }},
            ticks: {{
              color: "#94a3b8",
              font: {{ size: 11 }}
            }}
          }},
          x: {{
            title: {{
              display: true,
              text: metricsConfig[currentMetric].axisLabel,
              color: "#e5e7eb",
              font: {{ size: 13 }}
            }},
            grid: {{
              color: "rgba(15, 23, 42, 0.9)"
            }},
            ticks: {{
              color: "#94a3b8",
              font: {{ size: 11 }}
            }}
          }}
        }}
      }},
      plugins: [
        {{
          id: "dumbbellLines",
          beforeDatasetsDraw(chart) {{
            const {{ ctx }} = chart;
            const metaNon = chart.getDatasetMeta(0);
            const metaSpon = chart.getDatasetMeta(1);

            ctx.save();
            for (let i = 0; i < metaNon.data.length; i++) {{
              const p1 = metaNon.data[i];
              const p2 = metaSpon.data[i];

              const isHover = (i === hoverIndex);

              ctx.beginPath();
              ctx.moveTo(p1.x, p1.y);
              ctx.lineTo(p2.x, p2.y);
              ctx.lineWidth = isHover ? 4 : 2;
              ctx.strokeStyle = isHover
                ? "rgba(245, 158, 11, 1.0)"   
                : "rgba(148, 163, 184, 0.6)";
              ctx.stroke();
            }}
            ctx.restore();
          }}
        }}
      ]
    }});

    dumbbellChart.canvas.addEventListener("mouseleave", () => {{
      hoverIndex = null;
      dumbbellChart.canvas.style.cursor = "default";
      dumbbellChart.draw();
    }});

    const buttons = document.querySelectorAll(".toggle-button");

    buttons.forEach(btn => {{
      btn.addEventListener("click", () => {{
        const metricKey = btn.getAttribute("data-metric");
        if (metricKey === currentMetric) return;

        currentMetric = metricKey;

        buttons.forEach(b => b.classList.remove("active"));
        btn.classList.add("active");

        const updated = buildDatasets(currentMetric);
        dumbbellChart.data.datasets[0].data = updated.dataNon;
        dumbbellChart.data.datasets[1].data = updated.dataSpon;

        dumbbellChart.options.scales.x.title.text = metricsConfig[currentMetric].axisLabel;

        hoverIndex = null;
        dumbbellChart.update();
      }});
    }});
  </script>
</body>
</html>"""

    with open(filename, "w", encoding="utf-8") as f:
        f.write(html_content)

    print(f"{filename} has been generated")


if __name__ == "__main__":
    json_path = Path().cwd() / "rq3_dumbbell_data.json"
    with open(json_path, "r", encoding="utf-8") as f:
        second_plot_data = json.load(f)

    create_second_plot_rq3(
        second_plot_data,
        filename=html_path / "rq3_dumbbell.html"
    )
