from pathlib import Path
import pandas as pd


datapath = Path("../../dataset")

SPONSORED_CHANNELS_FILE = datapath / "sponsoredchannels.json"
METADATA_FILE   = datapath / "yt_metadata_en.jsonl.gz"
TIMESERIES_FILE = datapath / "df_timeseries_en.tsv.gz"                      

output_path = datapath / "data_rq3_delta_views_subs.csv.gz"


def load_sponsor_videos() -> pd.DataFrame:
    """
    Create df with columns "channel_id" and "display_id" (to have all videos 
    from SponsorBlock, with their corresponding channel). 
    """
    # load sponsored channels and construct proper dataframe
    df_sc = pd.read_json(SPONSORED_CHANNELS_FILE, lines=True)
    
    df_sc = df_sc.T            
    df_sc.columns = ["video_ids"]
    df_sc.reset_index(inplace=True)
    df_sc.rename(columns={"index": "channel_id"}, inplace=True)

    # one line per "display_id" (keeping "channel_id" as a column)
    df_videos = df_sc.explode("video_ids").rename(columns={"video_ids": "display_id"})

    # remove duplicates (if any)
    df_videos = df_videos.drop_duplicates(subset=["channel_id", "display_id"])
    
    print("number of channels: ", df_videos["channel_id"].nunique())
    print("number of videos: ", len(df_videos["display_id"]))

    return df_videos


def join_with_metadata(
    df_step1: pd.DataFrame
    ) -> pd.DataFrame:
    """
    Gather metadata of videos appearing in SponsorBlock.
    """
    sponsor_ids = set(df_step1["display_id"].unique())
    selected_chunks = []

    meta_chunks = pd.read_json(
        METADATA_FILE,
        lines=True,
        chunksize=1_000_000,
        compression="gzip"
    )

    for i, chunk in enumerate(meta_chunks):
        print(f"chunk number {i}")
        # keep only videos of channels in SponsorBlock
        mask = chunk["display_id"].isin(sponsor_ids)
        subset = chunk.loc[mask, ["channel_id", "display_id", "upload_date"]]
        if not subset.empty:
            selected_chunks.append(subset)

    assert len(selected_chunks) > 0

    df_meta_sponsor = pd.concat(selected_chunks, ignore_index=True)

    # convert upload date
    df_meta_sponsor["upload_date"] = pd.to_datetime(df_meta_sponsor["upload_date"])

    return df_meta_sponsor


def load_timeseries(
    filtered_channel_ids: pd.DataFrame
    ) -> pd.DataFrame:
    """
    Load timeseries data for channels in SponsorBlock (keeping wanted columns 
    and changing date format).
    """
    df_ts = pd.read_csv(
        TIMESERIES_FILE, 
        sep="\t", 
        compression="gzip",
        usecols=["channel", "datetime", "views", "delta_views", "subs", "delta_subs"]
        )

    df_ts["datetime"] = pd.to_datetime(df_ts["datetime"])

    df_ts = df_ts.rename(
        columns={
            "channel": "channel_id"
        }
    )

    df_ts = df_ts[df_ts["channel_id"].isin(filtered_channel_ids)].copy()

    return df_ts


def find_first_sponsor_date(
    df_step2: pd.DataFrame, 
    df_step3: pd.DataFrame
    ) -> pd.DataFrame:
    """
    Find earliest "upload_date" per channels, append to timeseries data and 
    substract to "datetime".
    """
    min_per_channel = (
        df_step2[["channel_id", "upload_date"]]
        .groupby("channel_id")["upload_date"]
        .min()                      
    )    
    
    df_step3["sponsor_start"] = df_step3["channel_id"].map(min_per_channel)
    
    df_step3["delta_weeks"] = (
    (df_step3["datetime"] - df_step3["sponsor_start"]).dt.days // 7
    )
    
    return df_step3


def build_csv_first_plot_rq3(
    output_file: Path,
    verbose: bool = False
    ) -> None:
    """
    Process using all the four functions above.
    """

    df_sponsor_videos = load_sponsor_videos()
    print("step 1 done: sponsored videos loaded")
    if verbose:
        print("shape: ", df_sponsor_videos.shape)
        print("columns: ", df_sponsor_videos.columns)

    df_video = join_with_metadata(df_sponsor_videos)
    print("step 2 done: metadata gathered")
    if verbose:
        print("shape: ", df_video.shape)
        print("columns: ", df_video.columns)
    
    sponsored_channel_ids = df_sponsor_videos["channel_id"].unique()
    df_ts = load_timeseries(filtered_channel_ids=sponsored_channel_ids)
    print("step 3 done: timeseries loaded and filtered")
    if verbose:
        print("shape: ", df_ts.shape)
        print("columns: ", df_ts.columns)
    
    df = find_first_sponsor_date(df_video, df_ts)
    print("step 4 done: merged timeseries + metadata info")
    if verbose:
        print("shape: ", df.shape)
        print("columns: ", df.columns)

    df.to_csv(output_file, index=False, compression="gzip")
    print("file created:", output_file)
    print("shape of final dataframe:", df.shape)
    print("columns of final dataframe:", df.columns)


if __name__ == "__main__":
    build_csv_first_plot_rq3(output_path, verbose=False)
