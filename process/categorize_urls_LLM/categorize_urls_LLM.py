"""
Domain Categorization for YouNiverse Dataset
Categorizes YouTube comment URLs to track creator professionalization
"""

import os
import openai
import pandas as pd
from pathlib import Path
from typing import List
import time
from dotenv import load_dotenv
import pandas as pd
from pathlib import Path
import os 
from dotenv import load_dotenv

repo_root = Path(__file__).parent.parent.parent



SYSTEM_PROMPT = """You are a domain categorization expert analyzing YouTube creator economy links. 

Categories:
monetization/direct: buymeacoffee.com, paypal.com, gofundme.com
monetization/merch: redbubble.com
monetization/sponsors: nordvpn.com
monetization/affiliates: g2a.com, amzn.to, amazon.com, geni.us, go.magik.ly, apple.co, rstyle.me, ldli.co, unionforgamers.com
monetization/stores: etsy.com, ebay.com
monetization/courses: udemy.com, coursera.org
url_shorteners: goo.gl, bit.ly - ONLY for urls that redirect you elsewhere, NOT for shortened urls of known websites
social_media: twitch.tv, discord.gg, instagram.com, on.fb.me
content/music: soundcloud.com, spoti.fi, beatport.com, ncs.io
content/video: youtube.com, youtu.be, vimeo.com, dailymotion.com
content/portfolios: behance.net, artstation.com
gaming: steamcommunity.com, roblox.com
legal/reference: creativecommons.org, google.com, en.wikipedia.org
news: on.msnbc.com, aparchive.com
entertainement: nfl.com
utils: mediafire.com, drive.google.com, foxnews.com
other: not part of other categories above, for example e.lga.to
unsure: category for IF YOU ARE UNSURE! 

Return ONLY the category name from the list above."""

def categorize_single_domain(domain: str, client: openai.OpenAI) -> str:
    try:
        print(f"  → {domain}...", end=" ", flush=True)

        response = client.chat.completions.create(
            model="gpt-5-nano",
            messages=[
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user", "content": f"Categorize: {domain}"}
            ],
            temperature=1,
            #timeout=30
        )
        
        category = response.choices[0].message.content.strip()
        print(f"{category}")
        return category 
    
    except Exception as e:
        print(f"Error: {str(e)[:100]}")
        return "error"

def categorize_domains_batch(
    domains_df: pd.DataFrame, 
    client: openai.OpenAI,
    batch_size: int = 1,
    delay: float = 0.1,
    checkpoint_file: Path = repo_root/'domains_categorized_checkpoint.csv'
) -> pd.DataFrame:
    results = []
    total = len(domains_df)
    
    for idx, row in domains_df.iterrows():
        domain = row['domain']
        
        if idx % 10 == 0:
            print(f"Processing {idx}/{total}...")
        
        category = categorize_single_domain(domain, client)
        results.append(category)
        
        if (idx + 1) % 10 == 0 or (idx + 1) == total:
            temp_df = domains_df.iloc[:idx+1].copy()
            temp_df['category'] = results
            temp_df.to_csv(checkpoint_file, index=False)
        
        time.sleep(delay)
    
    domains_df['category'] = results
    return domains_df

def main():
    load_dotenv() 

    api_key = os.getenv("OPENAI_API_KEY")

    domains = pd.read_csv(
    repo_root/"dataset"/"domains.csv.gz", 
    header=None,  
    names=['domain', 'count']  
    )
    
    # Filter domains with >= 100000 appearances
    domains_filtered = domains[domains['count'] >= 100000].copy()
    print(f"Domains with >=100000 appearances: {len(domains_filtered):,}")
    
    client = openai.OpenAI(api_key=api_key)
    
    print("\nCategorizing domains...")
    domains_categorized = categorize_domains_batch(
        domains_filtered, 
        client,
        delay=0.1 
    )
    
    # Save results
    output_file = repo_root/'domains_categorized.csv'
    domains_categorized.to_csv(output_file, index=False)
    print(f"\nResults saved to {output_file}")

    return domains_categorized

if __name__ == "__main__":
    domains_categorized = main()