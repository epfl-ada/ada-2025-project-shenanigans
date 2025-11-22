(function (root, factory) {
    if (typeof define === 'function' && define.amd) {
        // AMD. Register as an anonymous module.
        define(['exports', 'echarts'], factory);
    } else if (typeof exports === 'object' && typeof exports.nodeName !== 'string') {
        // CommonJS
        factory(exports, require('echarts'));
    } else {
        // Browser globals
        factory({}, root.echarts);
    }
}(this, function (exports, echarts) {
    var log = function (msg) {
        if (typeof console !== 'undefined') {
            console && console.error && console.error(msg);
        }
    };
    if (!echarts) {
        log('ECharts is not Loaded');
        return;
    }
    echarts.registerTheme('sheNaNigans', {
        "color": [
            "#dc8a78",
            "#dd7878",
            "#ea76cb",
            "#8839ef",
            "#d20f39",
            "#e64553",
            "#fe640b",
            "#df8e1d",
            "#40a02b",
            "#179299",
            "#04a5e5",
            "#209fb5",
            "#1e66f5",
            "#7287fd"
        ],
        "backgroundColor": "#1e1e2e",
        "textStyle": {},
        "title": {
            "textStyle": {
                "color": "#cdd6f4"
            },
            "subtextStyle": {
                "color": "#a6adc8"
            }
        },
        "line": {
            "itemStyle": {
                "borderWidth": 0
            },
            "lineStyle": {
                "width": 2
            },
            "symbolSize": 5,
            "symbol": "circle",
            "smooth": false
        },
        "radar": {
            "itemStyle": {
                "borderWidth": 0
            },
            "lineStyle": {
                "width": 2
            },
            "symbolSize": 5,
            "symbol": "circle",
            "smooth": false
        },
        "bar": {
            "itemStyle": {
                "barBorderWidth": 0,
                "barBorderColor": "#a6adc8"
            }
        },
        "pie": {
            "itemStyle": {
                "borderWidth": 0,
                "borderColor": "#a6adc8"
            }
        },
        "scatter": {
            "itemStyle": {
                "borderWidth": 0,
                "borderColor": "#a6adc8"
            }
        },
        "boxplot": {
            "itemStyle": {
                "borderWidth": 0,
                "borderColor": "#a6adc8"
            }
        },
        "parallel": {
            "itemStyle": {
                "borderWidth": 0,
                "borderColor": "#a6adc8"
            }
        },
        "sankey": {
            "itemStyle": {
                "borderWidth": 0,
                "borderColor": "#a6adc8"
            }
        },
        "funnel": {
            "itemStyle": {
                "borderWidth": 0,
                "borderColor": "#a6adc8"
            }
        },
        "gauge": {
            "itemStyle": {
                "borderWidth": 0,
                "borderColor": "#a6adc8"
            }
        },
        "candlestick": {
            "itemStyle": {
                "color": "#fd1050",
                "color0": "#0cf49b",
                "borderColor": "#fd1050",
                "borderColor0": "#0cf49b",
                "borderWidth": 1
            }
        },
        "graph": {
            "itemStyle": {
                "borderWidth": 0,
                "borderColor": "#a6adc8"
            },
            "lineStyle": {
                "width": 1,
                "color": "#a6adc8"
            },
            "symbolSize": 5,
            "symbol": "circle",
            "smooth": false,
            "color": [
                "#dc8a78",
                "#dd7878",
                "#ea76cb",
                "#8839ef",
                "#d20f39",
                "#e64553",
                "#fe640b",
                "#df8e1d",
                "#40a02b",
                "#179299",
                "#04a5e5",
                "#209fb5",
                "#1e66f5",
                "#7287fd"
            ],
            "label": {
                "color": "#cdd6f4"
            }
        },
        "map": {
            "itemStyle": {
                "areaColor": "#eee",
                "borderColor": "#444",
                "borderWidth": 0.5
            },
            "label": {
                "color": "#000"
            },
            "emphasis": {
                "itemStyle": {
                    "areaColor": "rgba(255,215,0,0.8)",
                    "borderColor": "#444",
                    "borderWidth": 1
                },
                "label": {
                    "color": "rgb(100,0,0)"
                }
            }
        },
        "geo": {
            "itemStyle": {
                "areaColor": "#eee",
                "borderColor": "#444",
                "borderWidth": 0.5
            },
            "label": {
                "color": "#000"
            },
            "emphasis": {
                "itemStyle": {
                    "areaColor": "rgba(255,215,0,0.8)",
                    "borderColor": "#444",
                    "borderWidth": 1
                },
                "label": {
                    "color": "rgb(100,0,0)"
                }
            }
        },
        "categoryAxis": {
            "axisLine": {
                "show": true,
                "lineStyle": {
                    "color": "#cdd6f4"
                }
            },
            "axisTick": {
                "show": true,
                "lineStyle": {
                    "color": "#cdd6f4"
                }
            },
            "axisLabel": {
                "show": true,
                "color": "#cdd6f4"
            },
            "splitLine": {
                "show": true,
                "lineStyle": {
                    "color": [
                        "#313244"
                    ]
                }
            },
            "splitArea": {
                "show": false,
                "areaStyle": {
                    "color": [
                        "#eeeeee"
                    ]
                }
            }
        },
        "valueAxis": {
            "axisLine": {
                "show": true,
                "lineStyle": {
                    "color": "#cdd6f4"
                }
            },
            "axisTick": {
                "show": true,
                "lineStyle": {
                    "color": "#cdd6f4"
                }
            },
            "axisLabel": {
                "show": true,
                "color": "#cdd6f4"
            },
            "splitLine": {
                "show": true,
                "lineStyle": {
                    "color": [
                        "#313244"
                    ]
                }
            },
            "splitArea": {
                "show": false,
                "areaStyle": {
                    "color": [
                        "#eeeeee"
                    ]
                }
            }
        },
        "logAxis": {
            "axisLine": {
                "show": true,
                "lineStyle": {
                    "color": "#cdd6f4"
                }
            },
            "axisTick": {
                "show": true,
                "lineStyle": {
                    "color": "#cdd6f4"
                }
            },
            "axisLabel": {
                "show": true,
                "color": "#cdd6f4"
            },
            "splitLine": {
                "show": true,
                "lineStyle": {
                    "color": [
                        "#313244"
                    ]
                }
            },
            "splitArea": {
                "show": false,
                "areaStyle": {
                    "color": [
                        "#eeeeee"
                    ]
                }
            }
        },
        "timeAxis": {
            "axisLine": {
                "show": true,
                "lineStyle": {
                    "color": "#cdd6f4"
                }
            },
            "axisTick": {
                "show": true,
                "lineStyle": {
                    "color": "#cdd6f4"
                }
            },
            "axisLabel": {
                "show": true,
                "color": "#cdd6f4"
            },
            "splitLine": {
                "show": true,
                "lineStyle": {
                    "color": [
                        "#313244"
                    ]
                }
            },
            "splitArea": {
                "show": false,
                "areaStyle": {
                    "color": [
                        "#eeeeee"
                    ]
                }
            }
        },
        "toolbox": {
            "iconStyle": {
                "borderColor": "#9399b2"
            },
            "emphasis": {
                "iconStyle": {
                    "borderColor": "#6c7086"
                }
            }
        },
        "legend": {
            "textStyle": {
                "color": "#cdd6f4"
            },
            "left": "center",
            "right": "auto",
            "top": "bottom",
            "bottom": "auto"
        },
        "tooltip": {
            "axisPointer": {
                "lineStyle": {
                    "color": "#585b70",
                    "width": "1"
                },
                "crossStyle": {
                    "color": "#585b70",
                    "width": "1"
                }
            }
        },
        "timeline": {
            "lineStyle": {
                "color": "#cdd6f4",
                "width": 1
            },
            "itemStyle": {
                "color": "#dc8a78",
                "borderWidth": 0
            },
            "controlStyle": {
                "color": "#cdd6f4",
                "borderColor": "#cdd6f4",
                "borderWidth": 0.5
            },
            "checkpointStyle": {
                "color": "#8839ef",
                "borderColor": "rgba(194,53,49, 0.5)"
            },
            "label": {
                "color": "#cdd6f4"
            },
            "emphasis": {
                "itemStyle": {
                    "color": "#dd7878"
                },
                "controlStyle": {
                    "color": "#cdd6f4",
                    "borderColor": "#cdd6f4",
                    "borderWidth": 0.5
                },
                "label": {
                    "color": "#cdd6f4"
                }
            }
        },
        "visualMap": {
            "color": [
                "#bf444c",
                "#d88273",
                "#f6efa6"
            ]
        },
        "markPoint": {
            "label": {
                "color": "#cdd6f4"
            },
            "emphasis": {
                "label": {
                    "color": "#cdd6f4"
                }
            }
        },
        "grid": {
            "left": "10%",
            "right": "10%",
            "top": 60,
            "bottom": 70
        }
    });
}));