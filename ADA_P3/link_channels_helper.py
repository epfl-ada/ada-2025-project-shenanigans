# helper.py
from __future__ import annotations

from typing import Dict, List, Mapping
import pandas as pd
import duckdb

MERGE_MAP = {
    # monetization
    "monetization/stores": "monetization",
    "monetization/courses": "monetization",
    "monetization/affiliates": "monetization",
    "monetization/merch": "monetization",
    "monetization/direct": "monetization",
    "monetization/sponsors": "monetization",

    # content
    "content/video": "content",
    "content/music": "content",
    "content/portfolios": "content",

    # social & entertainment
    "social_media": "social",
    "gaming": "entertainment",
    "entertainement": "entertainment",  

    # utility / infrastructure
    "url_shorteners": "infrastructure",
    "utils": "infrastructure",
    "legal/reference": "infrastructure",

    # news
    "news": "news",

    # problematic / unknown
    "other": "other",
    "unsure": "other",
    "non-existing site": "other",
}

def add_merged_category(domains_df: pd.DataFrame) -> pd.DataFrame:
    """
    Add a 'merged_category' column using MERGE_MAP.
    Returns a copy (does not mutate input).
    """
    if "category" not in domains_df.columns:
        raise ValueError("domains_df must contain a 'category' column.")
    df = domains_df.copy()
    df["merged_category"] = df["category"].map(MERGE_MAP).fillna("other")
    return df

GROUP_TO_MERGED: Dict[str, List[str]] = {
    "monetization": ["monetization"],
    "content": ["content"],
    "social": ["social"],
    "entertainment": ["entertainment"],
    "infrastructure": ["infrastructure"],
    "news": ["news"],
}

def build_group_conditions_merged(
    domains_df: pd.DataFrame,
    group_to_merged: Mapping[str, List[str]] = GROUP_TO_MERGED,
    urls_col_sql: str = "u.urls",
) -> Dict[str, str]:
    """
    Build DuckDB SQL condition strings for each group.

    Example output for group 'monetization':
        CAST(u.urls AS VARCHAR) ILIKE '%amzn.to%' OR CAST(u.urls AS VARCHAR) ILIKE '%amazon.com%'

    If a group has no domains, condition = 'FALSE'.
    """
    if "merged_category" not in domains_df.columns:
        raise ValueError("domains_df must contain a 'merged_category' column.")
    if "domain" not in domains_df.columns:
        raise ValueError("domains_df must contain a 'domain' column.")

    def make_condition(domain: str) -> str:
        safe = str(domain).replace("'", "''")
        return f"CAST({urls_col_sql} AS VARCHAR) ILIKE '%{safe}%'"

    conditions: Dict[str, str] = {}

    for group, merged_cats in group_to_merged.items():
        doms = (
            domains_df.loc[domains_df["merged_category"].isin(merged_cats), "domain"]
            .dropna()
            .astype(str)
            .unique()
            .tolist()
        )

        conditions[group] = "FALSE" if not doms else " OR ".join(make_condition(d) for d in doms)

    return conditions


def category_stats_multi(
    meta_path: str,
    urls_path: str,
    yt_category: str,
    domains_df: pd.DataFrame,
    group_to_merged: Mapping[str, List[str]] = GROUP_TO_MERGED,
    show_progress: bool = True,
) -> pd.DataFrame:
    """
    For a given YouTube category (m.categories = yt_category), compute per-year stats:

      - videos_total
      - videos_with_urls
      - videos_with_monetization
      - videos_with_content
      - videos_with_social
      - videos_with_entertainment
      - videos_with_infrastructure
      - videos_with_news
    """
    if show_progress:
        duckdb.query("PRAGMA enable_progress_bar;")

    group_conditions = build_group_conditions_merged(domains_df, group_to_merged)

    cond_monet = group_conditions.get("monetization", "FALSE")
    cond_cont  = group_conditions.get("content", "FALSE")
    cond_soc   = group_conditions.get("social", "FALSE")
    cond_ent   = group_conditions.get("entertainment", "FALSE")
    cond_infra = group_conditions.get("infrastructure", "FALSE")
    cond_news  = group_conditions.get("news", "FALSE")

    cat = yt_category.replace("'", "''")  # escape quotes for SQL literal

    base_cte = f"""
    WITH base AS (
        SELECT
            CAST(strftime(CAST(m.upload_date AS TIMESTAMP), '%Y') AS INTEGER) AS year,
            m.display_id,
            (u.urls IS NOT NULL) AS has_urls,

            (u.urls IS NOT NULL AND ({cond_monet})) AS has_monetization,
            (u.urls IS NOT NULL AND ({cond_cont}))  AS has_content,
            (u.urls IS NOT NULL AND ({cond_soc}))   AS has_social,
            (u.urls IS NOT NULL AND ({cond_ent}))   AS has_entertainment,
            (u.urls IS NOT NULL AND ({cond_infra})) AS has_infrastructure,
            (u.urls IS NOT NULL AND ({cond_news}))  AS has_news

        FROM read_json_auto('{meta_path}', format='newline_delimited') AS m
        LEFT JOIN read_json_auto('{urls_path}', format='newline_delimited') AS u
            USING (display_id)
        WHERE m.categories = '{cat}'
    )
    """

    query = base_cte + """
    SELECT
        year,
        COUNT(*) AS videos_total,
        SUM(CASE WHEN has_urls THEN 1 ELSE 0 END) AS videos_with_urls,
        SUM(CASE WHEN has_monetization THEN 1 ELSE 0 END) AS videos_with_monetization,
        SUM(CASE WHEN has_content THEN 1 ELSE 0 END) AS videos_with_content,
        SUM(CASE WHEN has_social THEN 1 ELSE 0 END) AS videos_with_social,
        SUM(CASE WHEN has_entertainment THEN 1 ELSE 0 END) AS videos_with_entertainment,
        SUM(CASE WHEN has_infrastructure THEN 1 ELSE 0 END) AS videos_with_infrastructure,
        SUM(CASE WHEN has_news THEN 1 ELSE 0 END) AS videos_with_news
    FROM base
    GROUP BY year
    ORDER BY year;
    """

    return duckdb.query(query).df()
