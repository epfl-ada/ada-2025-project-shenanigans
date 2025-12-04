import gzip
import json
import pickle
from pathlib import Path

import igraph as ig
import leidenalg as la

g_file = Path("domaingraph.json.gz")
with gzip.open(g_file) as f:
    j = json.loads(f.read())

edges = [e[:-1] for e in j["edges"] if e is not None]
weights = [e[-1] for e in j["edges"] if e is not None]

graph = ig.Graph(edges=edges, edge_attrs={"weight": weights})

l_file = Path("layout_fr.pkl")
if not l_file.exists():
    layout = graph.layout("graphopt", niter=1_000)
    pickle.dump(layout, l_file.open("wb"))
else:
    layout = pickle.load(l_file.open("rb"))

# ig.plot(graph, "tmp.png", layout=layout)

out = Path("layout.json")
json.dump(layout.coords, out.open("w"))

p_file = Path("leiden.pkl")
if not p_file.exists():
    parts = list(la.find_partition(graph, la.ModularityVertexPartition))
    pickle.dump(parts, p_file.open("wb"))
else:
    parts = pickle.load(p_file.open("rb"))

out = Path("leiden.json")
json.dump(parts, out.open("w"))
