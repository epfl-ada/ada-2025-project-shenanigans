from pathlib import Path
import pandas as pd
from typing import Tuple
from numpy.typing import NDArray


datapath = Path("../../dataset")

SPONSORED_CHANNELS_FILE = datapath / "sponsoredchannels.json"
METADATA_FILE   = datapath / "yt_metadata_en.jsonl.gz"

output_path = datapath / "data_rq3_second_plot_gt5.csv.gz"


def load_metadata() -> Tuple[pd.DataFrame, NDArray]:
    """
    Create df with metadata of all videos from channels found in SponsorBlock. 
    """
    # load sponsored channels and construct proper dataframe
    df_sc = pd.read_json(SPONSORED_CHANNELS_FILE, lines=True)
    df_sc = df_sc.T         
    df_sc.columns = ["video_ids"]
    df_sc.reset_index(inplace=True)
    df_sc.rename(columns={"index": "channel_id"}, inplace=True)
    
    # keep only channels with at least 5 videos in SponsorBlock
    df_gt10 = df_sc[df_sc["video_ids"].apply(len) >= 5]
    df_videos = df_gt10.explode("video_ids").rename(columns={"video_ids": "display_id"})
    
    # remove duplicates (if any)
    df_videos = df_videos.drop_duplicates(subset=["channel_id", "display_id"])
    
    print("number of channels: ", df_videos["channel_id"].nunique())
    print("number of videos: ", len(df_videos["display_id"]))

    sponsored_channel_ids = df_videos["channel_id"].unique()
    sponsored_videos_ids = df_videos["display_id"].unique()
    
    selected_chunks = []

    meta_chunks = pd.read_json(
        METADATA_FILE,
        lines=True,
        chunksize=1_000_000,
        compression="gzip"
    )

    # gather metadata for videos (sponsored or not) of channels in SponsorBlock
    for i, chunk in enumerate(meta_chunks):
        print(f"chunk number {i}")
        mask = chunk["channel_id"].isin(sponsored_channel_ids)
        subset = chunk.loc[mask, ["categories",
                                  "channel_id",
                                  "display_id",
                                  "dislike_count",
                                  "duration",
                                  "like_count",
                                  "view_count"
                                  ]
                           ]
        if not subset.empty:
            selected_chunks.append(subset)

    assert len(selected_chunks) > 0

    df_meta_sponsor = pd.concat(selected_chunks, ignore_index=True)

    return df_meta_sponsor, sponsored_videos_ids


def add_columns(
    df_step1: pd.DataFrame,
    sponsored_videos_ids: NDArray
    ) -> pd.DataFrame:
    """
    Add "like_dislike_ratio" and "sponsored" columns.
    """
    dislikes = df_step1["dislike_count"]
    likes = df_step1["like_count"]
    
    # get like/dislike ration as percentage
    df_step1["like_dislike_ratio"] = (likes/(likes+dislikes))*100
    
    df_step1["sponsored"] = df_step1["display_id"].isin(sponsored_videos_ids)

    return df_step1


def build_csv_second_plot_rq3(
    output_file: Path,
    verbose: bool = False
    ) -> None:
    """
    Process using the two functions above.
    """
    df_step1, sponsored_videos_ids = load_metadata()
    print("step 1 done: sponsored videos loaded")
    if verbose:
        print("shape: ", df_step1.shape)
        print("columns: ", df_step1.columns)

    df = add_columns(df_step1, sponsored_videos_ids)
    print("step 2 done: metadata gathered")
    if verbose:
        print("shape: ", df.shape)
        print("columns: ", df.columns)

    df.to_csv(output_file, index=False, compression="gzip")
    print("file created:", output_file)
    print("shape of final dataframe:", df.shape)
    print("columns of final dataframe:", df.columns)


if __name__ == "__main__":
    build_csv_second_plot_rq3(output_path, verbose=False)
