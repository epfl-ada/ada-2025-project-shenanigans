import pandas as pd
import re
import pickle
from pathlib import Path 

# consider only patreon and paypal urls
money_income_re = re.compile(
    r"""(?ix)                           
    \b
    (?:https?://)?                     
    (?:www\.)?                          
    (?:[a-z0-9-]+\.)*                  
    (?:                               
      patreon\.com
      | paypal\.com
      | paypal\.me
    )
    (?:[/:?#]\S*)?                      
    \b
    """,
)

datapath = Path("../data") 

for cat_id in range(15):
    print(f"Category {cat_id+1}...") # +1 to avoid taking empty cat into account
    money_vc = []
    non_money_vc = []
    
    chunks = pd.read_json(
        datapath / "urls_cat_vc.jsonl.gz", 
        lines=True,
        chunksize=1_000_000,
        compression="gzip"
    )

    for i, chunk in enumerate(chunks):
        cat_chunk = chunk[chunk["category"]==cat_id+1] 
        print(f"chunk number {i}")
        for _, _, urls, _, vc in cat_chunk.itertuples():
            if any(money_income_re.search((url or "")) for url in urls):
                money_vc.append(vc)
                continue
            non_money_vc.append(vc)

    with open(datapath / f"money_vc_{cat_id+1}.pkl", "wb") as f:
        pickle.dump({"money_vc": money_vc, "non_money_vc": non_money_vc}, f)
