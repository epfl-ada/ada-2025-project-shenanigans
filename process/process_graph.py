import json
import math
import pickle
from pathlib import Path

import igraph as ig
import leidenalg as la

graph_path = Path("graph.pkl")
if graph_path.exists():
    g = pickle.load(graph_path.open("rb"))
else:
    g = ig.Graph().Read_Ncol("graph.csv")
    pickle.dump(g, graph_path.open("wb"))
    g.to_undirected()
    g.simplify()
    g = g.connected_components(mode="weak").giant()
print("Loaded graph!")

layout_path = Path("layout.pkl")
if layout_path.exists():
    layout = pickle.load(layout_path.open("rb"))
else:
    layout = g.layout("lgl")
    pickle.dump(layout, layout_path.open("wb"))
print("Computed layout!")

styled_path = Path("styled.pkl")
if styled_path.exists():
    g = pickle.load(styled_path.open("rb"))
else:
    n = 0.0
    avg = 0.0
    maxi = 0
    for v in g.vs:
        n += 1
        avg += v.degree()
        if v.degree() > maxi:
            maxi = v.degree()
        v["size"] = math.log(v.degree() + 1) + 1
        v["color"] = "rgb(156, 207, 216)"
        v["frame_color"] = "rgb(38, 35, 58)"

    print(maxi)
    avg /= n
    print(avg)

    for e in g.es:
        e["color"] = "rgb(110, 106, 134)"

    pickle.dump(g, styled_path.open("wb"))

print("finished edges")

out = {}
out["nodes"] = []
for i in range(len(g.vs)):
    atts = g.vs[i].attributes()
    node = {}
    node["id"] = i
    node["name"] = atts["name"]
    node["size"] = atts["size"]
    node["x"] = layout[i][0]
    node["y"] = layout[i][1]
    out["nodes"].append(node)

out["edges"] = []
for e in g.es:
    edge = {}
    edge["source"] = e.source
    edge["target"] = e.target
    out["edges"].append(edge)

json.dump(out, Path("graph.json").open("w"))
