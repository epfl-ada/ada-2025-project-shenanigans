import os, re, time, json
from pathlib import Path
from datetime import datetime, timezone
from concurrent.futures import ThreadPoolExecutor

import numpy as np
import pandas as pd
import requests

API_KEY = os.getenv("YT_API_KEY") or globals().get("API_KEY", "")

BASE = "https://www.googleapis.com/youtube/v3"
CH_ID_RE = re.compile(r"^UC[A-Za-z0-9_-]{20,}$")
URL_CH_RE = re.compile(r"(?:https?://)?(?:www\.)?youtube\.com/channel/([^/?#]+)", re.I)


# ----------------------------
# helpers
# ----------------------------
UNIT_DEFAULT = 200
DEFAULT_ACTIVITY_MAPPING = {
    "inactive": "not_active",
    "Not Available": "not_active",
}

def normalize_activity_band(df: pd.DataFrame) -> pd.DataFrame:
    df = df.copy()

    if "activity_band_2025" in df.columns:
        df["activity_band"] = df["activity_band_2025"]
    elif "activity_band" not in df.columns:
        raise ValueError("need activity_band or activity_band_2025")

    df["activity_band"] = (
        df["activity_band"]
        .astype(str)
        .str.strip()
        .str.lower()
        .replace(DEFAULT_ACTIVITY_MAPPING)
    )
    return df
    
def validate_activity_inputs(df: pd.DataFrame) -> None:
    required = ["category_cc", "activity_band", "url", "subscribers_cc"]
    missing = [c for c in required if c not in df.columns]
    if missing:
        raise ValueError(f"df missing columns: {missing}")

def make_category_counts(df_cat: pd.DataFrame, cat_id: str, label: str) -> dict:
    counts = df_cat["activity_band"].value_counts(dropna=False)
    return {
        "id": cat_id,
        "label": label,
        "active": int(counts.get("active", 0)),
        "not_active": int(counts.get("not_active", 0)),
    }

def sample_representative_links(
    df_cat: pd.DataFrame,
    *,
    unit: int,
    keep_fields: list[str],
) -> dict:
    counts = df_cat["activity_band"].value_counts(dropna=False)

    n_icons = {
        band: int(round(counts.get(band, 0) / unit))
        for band in ["active", "not_active"]
    }

    out = {"active": [], "not_active": []}

    for band, n in n_icons.items():
        if n <= 0:
            continue

        picked = (
            df_cat[df_cat["activity_band"] == band]
            .sort_values("subscribers_cc", ascending=False)
            .head(n)
        )

        out[band] = [
            {k: r.get(k) for k in keep_fields}
            | {
                "url": r.get("url"),
                "category_cc": r.get("category_cc"),
            }
            for _, r in picked.iterrows()
        ]

    return out

def no_data() -> str:
    return "Not Available"

def channel_id_from_any(x):
    if x is None:
        return None
    s = str(x).strip()
    if CH_ID_RE.match(s):
        return s
    m = URL_CH_RE.search(s)
    return m.group(1) if m else None

def channel_url(cid: str) -> str:
    return f"https://www.youtube.com/channel/{cid}"

def classify_activity_from_last_upload(last_upload_iso: str | None, year: int) -> str:
    if not last_upload_iso or last_upload_iso == no_data():
        return no_data()
    try:
        y = int(str(last_upload_iso)[:4])
        return "active" if y == year else "inactive"
    except Exception:
        return no_data()

def is_quota_error(err: Exception) -> bool:
    s = str(err).lower()
    return any(k in s for k in [
        "quotaexceeded",
        "dailylimitexceeded",
        "userratelimitexceeded",
        "ratelimitexceeded",
        "exceeded your quota",
    ])

def yt_get(endpoint: str, params: dict, session: requests.Session, timeout=20, retries=5):
    url = f"{BASE}/{endpoint}"
    params = dict(params)
    params["key"] = API_KEY

    last_err = None
    for attempt in range(retries):
        try:
            r = session.get(url, params=params, timeout=timeout)
            if r.status_code == 200:
                return r.json()

            if r.status_code in (429, 500, 503):
                time.sleep(1.5 * (attempt + 1))
                continue

            last_err = RuntimeError(f"HTTP {r.status_code}: {r.text}")
            raise last_err

        except Exception as e:
            last_err = e
            time.sleep(1.0 * (attempt + 1))

    raise last_err


# ----------------------------
# api: channels.list (batched)
# ----------------------------
def fetch_channels_core(channel_ids: list[str], session: requests.Session) -> dict[str, dict]:
    out = {}
    ids = pd.Series(channel_ids).dropna().astype(str).unique().tolist()

    def to_int(x):
        try:
            return int(x)
        except Exception:
            return None

    for i in range(0, len(ids), 50):
        chunk = ids[i:i+50]
        data = yt_get(
            "channels",
            {"part": "snippet,statistics,contentDetails", "id": ",".join(chunk), "maxResults": 50},
            session=session,
        )
        items = {it["id"]: it for it in data.get("items", [])}

        for cid in chunk:
            it = items.get(cid)
            if not it:
                out[cid] = {"has_api": False}
                continue

            sn = it.get("snippet") or {}
            st = it.get("statistics") or {}
            cd = it.get("contentDetails") or {}
            uploads = ((cd.get("relatedPlaylists") or {}).get("uploads"))

            thumbs = sn.get("thumbnails") or {}
            avatar = (
                (thumbs.get("high") or {}).get("url")
                or (thumbs.get("medium") or {}).get("url")
                or (thumbs.get("default") or {}).get("url")
            )

            out[cid] = {
                "has_api": True,
                "channel_title_2025": sn.get("title"),
                "channel_avatar_url_2025": avatar,
                "subscriber_count_2025": to_int(st.get("subscriberCount")),
                "video_count_2025": to_int(st.get("videoCount")),
                "view_count_2025": to_int(st.get("viewCount")),
                "uploads_playlist_id": uploads,
            }

        time.sleep(0.02)

    return out


# ----------------------------
# api: playlistItems.list (1 per channel, concurrent) -> latest video id
# ----------------------------
def _fetch_latest_video_id_one(uploads_pid: str, session: requests.Session) -> tuple[str | None, str | None]:
    data = yt_get(
        "playlistItems",
        {"part": "contentDetails,snippet", "playlistId": uploads_pid, "maxResults": 1},
        session=session,
    )
    items = data.get("items") or []
    if not items:
        return None, None
    it = items[0]
    cd = it.get("contentDetails") or {}
    sn = it.get("snippet") or {}
    return cd.get("videoId"), sn.get("title")

def fetch_latest_video_ids_concurrent(core_map: dict[str, dict], session: requests.Session, max_workers=15) -> dict[str, dict]:
    out = {cid: {"latest_video_id": None, "last_video_title_2025": None} for cid in core_map.keys()}

    tasks = []
    with ThreadPoolExecutor(max_workers=max_workers) as ex:
        for cid, info in core_map.items():
            pid = info.get("uploads_playlist_id")
            if not info.get("has_api") or not pid:
                continue
            tasks.append((cid, ex.submit(_fetch_latest_video_id_one, pid, session)))

        for cid, fut in tasks:
            try:
                vid, title = fut.result()
                out[cid] = {"latest_video_id": vid, "last_video_title_2025": title}
            except Exception:
                out[cid] = {"latest_video_id": None, "last_video_title_2025": None}

    return out


# ----------------------------
# api: videos.list (batched) -> publishedAt + title
# ----------------------------
def fetch_videos_snippet(video_ids: list[str], session: requests.Session) -> dict[str, dict]:
    out = {}
    vids = pd.Series(video_ids).dropna().astype(str).unique().tolist()

    for i in range(0, len(vids), 50):
        chunk = vids[i:i+50]
        data = yt_get(
            "videos",
            {"part": "snippet", "id": ",".join(chunk), "maxResults": 50},
            session=session,
        )
        for it in (data.get("items") or []):
            vid = it.get("id")
            sn = it.get("snippet") or {}
            out[vid] = {"publishedAt": sn.get("publishedAt"), "title": sn.get("title")}
        time.sleep(0.02)

    return out


# ----------------------------
# parquet cache helpers
# ----------------------------
def _cache_path(cache_dir: str | Path) -> Path:
    cache_dir = Path(cache_dir)
    cache_dir.mkdir(parents=True, exist_ok=True)
    return cache_dir / "yt_enrichment_cache.parquet"

def load_cache_parquet(cache_dir: str | Path) -> pd.DataFrame:
    path = _cache_path(cache_dir)
    if not path.exists():
        return pd.DataFrame()
    return pd.read_parquet(path)

def upsert_cache_parquet(cache_dir: str | Path, new_rows: pd.DataFrame) -> None:
    path = _cache_path(cache_dir)
    if new_rows.empty:
        return

    if path.exists():
        old = pd.read_parquet(path)
        combined = pd.concat([old, new_rows], ignore_index=True)
        combined = combined.drop_duplicates(subset=["_channel_id_norm"], keep="last")
    else:
        combined = new_rows.drop_duplicates(subset=["_channel_id_norm"], keep="last")

    combined.to_parquet(path, index=False)


# ----------------------------
# resumable main
# ----------------------------
def extend_df_with_youtube_fetch_resumable_parquet(
    df: pd.DataFrame,
    channel_col: str,
    year: int,
    max_workers: int = 15,
    cache_dir: str | Path = "cache/yt_parquet",
    resume: bool = True,
    chunk_size_channels: int = 2000,
) -> pd.DataFrame:
    if channel_col not in df.columns:
        raise ValueError(f"df missing channel column: {channel_col}")

    df_ext = df.copy()
    df_ext["_channel_id_norm"] = df_ext[channel_col].map(channel_id_from_any)

    ids_all = (
        pd.Series(df_ext["_channel_id_norm"])
        .dropna()
        .astype(str)
        .unique()
        .tolist()
    )
    if len(ids_all) == 0:
        raise ValueError("Could not parse any channel ids from your channel column.")

    cached_df = load_cache_parquet(cache_dir) if resume else pd.DataFrame()
    cached_ids = set(cached_df["_channel_id_norm"].astype(str).tolist()) if (resume and not cached_df.empty and "_channel_id_norm" in cached_df.columns) else set()

    remaining = [cid for cid in ids_all if cid not in cached_ids]
    print(f"total channels: {len(ids_all):,} | cached: {len(cached_ids):,} | remaining: {len(remaining):,}")

    fetched_at = datetime.now(timezone.utc).isoformat()
    fetched_rows_all = []

    with requests.Session() as session:
        for start in range(0, len(remaining), chunk_size_channels):
            chunk_ids = remaining[start:start + chunk_size_channels]
            print(f"processing chunk {start//chunk_size_channels + 1} | channels={len(chunk_ids):,}")

            try:
                core = fetch_channels_core(chunk_ids, session=session)
                latest_vids = fetch_latest_video_ids_concurrent(core, session=session, max_workers=max_workers)

                all_video_ids = [v["latest_video_id"] for v in latest_vids.values() if v.get("latest_video_id")]
                video_sn = fetch_videos_snippet(all_video_ids, session=session) if all_video_ids else {}

            except Exception as e:
                if is_quota_error(e):
                    print("stopping early due to quota/rate limit. progress is saved in parquet cache.")
                    break
                raise

            rows = []
            for cid in chunk_ids:
                info = dict(core.get(cid, {}) or {})
                lv = dict(latest_vids.get(cid, {}) or {})

                has_api = bool(info.get("has_api"))
                latest_video_id = lv.get("latest_video_id")

                published_at = None
                title = lv.get("last_video_title_2025")
                if latest_video_id and latest_video_id in video_sn:
                    published_at = video_sn[latest_video_id].get("publishedAt")
                    title = video_sn[latest_video_id].get("title") or title

                activity_band = classify_activity_from_last_upload(published_at, year) if has_api else no_data()

                rows.append({
                    "_channel_id_norm": cid,
                    "url": channel_url(cid),
                    "has_api": has_api,
                    "subs_fetched_at_utc": fetched_at,

                    "channel_title_2025": info.get("channel_title_2025"),
                    "channel_avatar_url_2025": info.get("channel_avatar_url_2025"),
                    "subscriber_count_2025": info.get("subscriber_count_2025"),
                    "video_count_2025": info.get("video_count_2025"),
                    "view_count_2025": info.get("view_count_2025"),

                    "uploads_playlist_id": info.get("uploads_playlist_id"),
                    "latest_video_id": latest_video_id,

                    "last_upload_utc_2025": published_at,
                    "last_video_title_2025": title,
                    "activity_band_2025": activity_band,
                })

            new_rows = pd.DataFrame(rows)
            upsert_cache_parquet(cache_dir, new_rows)
            fetched_rows_all.append(new_rows)

    # build final merge source = cache (preferred) + any rows fetched in this run
    final_cache = load_cache_parquet(cache_dir) if resume else pd.concat(fetched_rows_all, ignore_index=True)
    if final_cache.empty:
        raise ValueError("no cached or fetched rows available. nothing to merge.")

    df_ext = df_ext.merge(final_cache, on="_channel_id_norm", how="left")

    fill_cols = [
        "channel_title_2025", "channel_avatar_url_2025",
        "subscriber_count_2025", "video_count_2025", "view_count_2025",
        "last_upload_utc_2025", "last_video_title_2025",
        "activity_band_2025"
    ]
    for c in fill_cols:
        if c in df_ext.columns:
            df_ext[c] = df_ext[c].where(pd.notna(df_ext[c]), no_data())

    return df_ext