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

video_to_cat: Dict[str, int] = {}

for i, meta_chunk in enumerate(meta_chunks):
    print("meta_chunk number ", i)
    for _, id, cat in meta_chunk[["display_id", "categories"]].itertuples():
        try:
            cat_id = categories.index(cat)
            video_to_cat[id] = cat_id
        except ValueError:
            continue

print("finished video map!\n")

with gz.open(datapath / "urls_cat.jsonl.gz", "wb") as out:
    urls_chunks = pd.read_json(
        datapath / "urls.jsonl.gz",
        lines=True,
        chunksize=1_000_000,
    )
    results: List[Dict[str, str | List[str] | int]] = []
    for i, urls_chunk in enumerate(urls_chunks):
        print("urls_chunk number ", i)
        for _, id, urls in urls_chunk.itertuples():
            category = video_to_cat.get(id)
            if category is None:
                continue
            results.append(
                {"display_id": id, "urls": urls, "category": category}
            )

        if len(results) > 1_000_000:
            out.writelines((json.dumps(res) + "\n").encode("utf-8") for res in results)
            results.clear()
    out.writelines((json.dumps(res) + "\n").encode("utf-8") for res in results)
