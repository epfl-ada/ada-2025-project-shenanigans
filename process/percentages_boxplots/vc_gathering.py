import gzip as gz
import json
from pathlib import Path
from typing import Dict, List
import pandas as pd


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

datapath = Path("../../dataset")

meta_chunks = pd.read_json(
    datapath / "yt_metadata_en.jsonl.gz",
    lines=True,
    chunksize=1_000_000,
    compression="gzip"
)

video_to_vc: Dict[str, int] = {}

for i, meta_chunk in enumerate(meta_chunks):
    print("meta_chunk number ", i)
    for _, id, vc in meta_chunk[["display_id", "view_count"]].itertuples():
        try:
            video_to_vc[id] = vc
        except ValueError:
            continue

print("finished view count map!\n")

with gz.open(datapath / "urls_cat_vc.jsonl.gz", "wb") as out:
    urls_cat_chunks = pd.read_json(
        datapath / "urls_cat.jsonl.gz",
        lines=True,
        chunksize=1_000_000,
    )
    results: List[Dict[str, str | List[str] | int]] = []
    for i, urls_cat_chunk in enumerate(urls_cat_chunks):
        print("urls_cat_chunk number ", i)
        for _, id, urls, cat in urls_cat_chunk.itertuples():
            view_count = video_to_vc.get(id)
            if view_count is None:
                continue
            results.append(
                {"display_id": id, "urls": urls, "category": cat, "view_count": view_count}
            )

        if len(results) > 1_000_000:
            out.writelines((json.dumps(res) + "\n").encode("utf-8") for res in results)
            results.clear()
    out.writelines((json.dumps(res) + "\n").encode("utf-8") for res in results)
