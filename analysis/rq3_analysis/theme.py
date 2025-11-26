import plotly.graph_objects as go
import plotly.io as pio

shenanigans_colorway = [
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
    "#7287fd",
]

shenanigans_template = go.layout.Template(
    layout=dict(
        colorway=shenanigans_colorway,
        plot_bgcolor="#11111b",
        paper_bgcolor="#11111b",
        font=dict(
            family="Tex Gyre Adventor, sans-serif",
            color="#cdd6f4",
        ),
        title=dict(
            font=dict(color="#cdd6f4"),
        ),
        xaxis=dict(
            showgrid=True,
            gridcolor="#313244",
            zerolinecolor="#313244",
            color="#cdd6f4",
        ),
        yaxis=dict(
            showgrid=True,
            gridcolor="#313244",
            zerolinecolor="#313244",
            color="#cdd6f4",
        ),
        legend=dict(
            font=dict(color="#cdd6f4"),
            x=0.5,
            xanchor="center",
            y=-0.15,
            yanchor="top",
            orientation="h",
        ),
        margin=dict(l=80, r=80, t=80, b=80),
        hoverlabel=dict(
            font=dict(color="#cdd6f4"),
        ),
    )
)

pio.templates["shenanigans"] = shenanigans_template


def use_shenanigans_template():
    """
    Set "plotly+shenanigans" as the default template.
    """
    pio.templates["shenanigans"] = shenanigans_template
    pio.templates.default = "plotly+shenanigans"
