import os
import re
import time
from concurrent.futures import ThreadPoolExecutor, as_completed

import pandas as pd
import requests

API_KEY = os.getenv("YT_API_KEY") or globals().get("API_KEY", "")
BASE = "https://www.googleapis.com/youtube/v3"
VID_RE = re.compile(r"^[A-Za-z0-9_-]{11}$")


def _extract_vid(x):
    if x is None:
        return None
    s = str(x).strip()
    if VID_RE.fullmatch(s):
        return s
    m = re.search(r"(?:v=)([A-Za-z0-9_-]{11})", s) or re.search(
        r"(youtu\.be/|/embed/|/shorts/)([A-Za-z0-9_-]{11})", s
    )
    return (m.group(1 if m.lastindex == 1 else 2)) if m else None


def _oembed_status(vid):
    try:
        r = requests.get(
            "https://www.youtube.com/oembed",
            params={
                "url": f"https://www.youtube.com/watch?v={vid}",
                "format": "json",
            },
            timeout=10,
        )
        if r.status_code == 200:
            return "ok"
        if r.status_code == 401:
            return "private"
        if r.status_code in (404, 410):
            return "removed"
        return f"http_{r.status_code}"
    except Exception:
        return "error"


def get_video_title_and_status(video_ids, api_key):
    ids = (
        pd.Series(video_ids)
        .dropna()
        .astype(str)
        .map(_extract_vid)
        .dropna()
        .unique()
        .tolist()
    )
    rows, missing = [], []
    with requests.Session() as s:
        for i in range(0, len(ids), 50):
            chunk = ids[i : i + 50]
            r = s.get(
                f"{BASE}/videos",
                params={
                    "part": "snippet,status",
                    "id": ",".join(chunk),
                    "key": api_key,
                },
                timeout=20,
            )
            r.raise_for_status()
            items = {it["id"]: it for it in r.json().get("items", [])}
            for vid in chunk:
                it = items.get(vid)
                if not it:
                    missing.append(vid)
                    continue
                title = (it.get("snippet") or {}).get("title")
                privacy = (it.get("status") or {}).get("privacyStatus")
                status = (
                    "ok"
                    if privacy in ("public", "unlisted")
                    else ("private" if privacy == "private" else "unknown")
                )
                rows.append(
                    {"video_id": vid, "video_title": title, "yt_status": status}
                )
            time.sleep(0.03)
    for vid in missing:
        rows.append(
            {
                "video_id": vid,
                "video_title": None,
                "yt_status": _oembed_status(vid),
            }
        )
    out = pd.DataFrame(rows)
    if out.empty:
        out = pd.DataFrame(
            columns=["video_id", "video_title", "yt_status", "is_available"]
        )
    out["is_available"] = out["yt_status"].eq("ok")
    return out


def _uploads_playlist_ids(channel_ids, api_key):
    ids = pd.Series(channel_ids).dropna().astype(str).unique().tolist()
    out = {}
    with requests.Session() as s:
        for i in range(0, len(ids), 50):
            chunk = ids[i : i + 50]
            r = s.get(
                f"{BASE}/channels",
                params={
                    "part": "contentDetails",
                    "id": ",".join(chunk),
                    "key": api_key,
                },
                timeout=20,
            )
            r.raise_for_status()
            for it in r.json().get("items", []):
                out[it["id"]] = it["contentDetails"]["relatedPlaylists"][
                    "uploads"
                ]
            time.sleep(0.03)
    return out


def _latest_from_uploads(pid, api_key):
    r = requests.get(
        f"{BASE}/playlistItems",
        params={
            "part": "contentDetails,snippet",
            "playlistId": pid,
            "maxResults": 50,
            "key": api_key,
        },
        timeout=20,
    )
    if not r.ok:
        return None, pd.NaT, None
    items = r.json().get("items", [])
    best = None
    for it in items:
        sn = it.get("snippet") or {}
        cd = it.get("contentDetails") or {}
        vid = cd.get("videoId") or (sn.get("resourceId") or {}).get("videoId")
        ts = cd.get("videoPublishedAt") or sn.get("publishedAt")
        title = sn.get("title")
        if not (vid and ts):
            continue
        ts = pd.to_datetime(ts, utc=True, errors="coerce")
        if pd.isna(ts):
            continue
        if best is None or ts > best[0]:
            best = (ts, vid, title)
    if not best:
        return None, pd.NaT, None
    return best[1], best[0], best[2]


def get_channels_latest_bulk(
    channel_ids, api_key, tz="Europe/Zurich", workers=32
):
    ch2pid = _uploads_playlist_ids(channel_ids, api_key)
    recs = []
    with ThreadPoolExecutor(max_workers=workers) as ex:
        futs = {
            ex.submit(_latest_from_uploads, pid, api_key): cid
            for cid, pid in ch2pid.items()
            if pd.notna(pid)
        }
        for f in as_completed(futs):
            cid = futs[f]
            try:
                vid, ts_utc, title = f.result()
            except Exception:
                vid, ts_utc, title = None, pd.NaT, None
            recs.append(
                {
                    "channel_id": cid,
                    "last_video_id": vid,
                    "last_upload_utc": ts_utc,
                    "last_video_title": title,
                }
            )
    out = pd.DataFrame(recs)
    if out.empty:
        out = pd.DataFrame(
            columns=[
                "channel_id",
                "last_video_id",
                "last_upload_utc",
                "last_video_title",
                "last_upload_local",
            ]
        )
    out["last_upload_utc"] = pd.to_datetime(
        out["last_upload_utc"], utc=True, errors="coerce"
    )
    out["last_upload_local"] = out["last_upload_utc"].dt.tz_convert(tz)
    return out


def extend_dataframe_with_titles_status_and_latest(
    b: pd.DataFrame, api_key: str, tz="Europe/Zurich"
) -> pd.DataFrame:
    df = b.copy()
    if "video_id" not in df.columns and "display_id" in df.columns:
        df = df.rename(columns={"display_id": "video_id"})
    if "video_id" in df.columns:
        vdf = get_video_title_and_status(df["video_id"], api_key)
        df = df.merge(vdf, on="video_id", how="left")
    if "channel_id" in df.columns:
        cdf = get_channels_latest_bulk(df["channel_id"], api_key, tz=tz)
        df = df.merge(
            cdf[
                [
                    "channel_id",
                    "last_upload_utc",
                    "last_upload_local",
                    "last_video_title",
                ]
            ],
            on="channel_id",
            how="left",
        )
    return df
