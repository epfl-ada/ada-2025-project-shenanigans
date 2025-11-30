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



SYSTEM_PROMPT = """You are a domain categorization expert analyzing YouTube creator economy links. Your goal is to use your knowledge and common sense to categorize the URLS inside the description.

Categories:
monetization/direct: where you can donate directly to the youtuber 
monetization/merch: where a youtuber sells merch 
monetization/sponsors: companies that are known to sponsor youtube videos
monetization/affiliates: websites where the youtuber can earn a commission
monetization/stores: the youtuber's store 
monetization/courses: the youtuber's courses 
url_shorteners: specific websites that redirect you to ANOTHER website. this does NOT mean the shortened form of a specific website, ie spoti.fi redirects to spotify: spoti.fi is NOT a url shortener
social_media: all forms where youtubers interact, including discord and twitch. 
content/music: anyplace youtubers can find and share music
content/video: anyplace youtubers can find and share videos
content/portfolios: youtuber's portfolios
gaming: any website thats gaming related
legal/reference: any website that can be used for sources or backing up what the user "did"
news: any news source 
entertainement: any entertainement source (for example, but not only: nba, nickelodeon)
utils: Any website that can be useful to the process of creating and maintaining a youtube channel
other: not part of other categories above
non-existing site: websites that no longer exist, and are not online. 
unsure: category for IF YOU ARE UNSURE! Try your best to categorize everything you can, only put it here if you truly don't know

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
    checkpoint_file: Path = repo_root/ "process" / "categorize_urls_LLM" /'sponsoreddomains_unshortened_checkpoint_v2.csv'
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
    repo_root/"dataset"/"sponsoreddomains_unshortened.csv.gz", 
    header=None,  
    names=['domain', 'count']  
    )
    
    #>100 appearances = 1.6k urls 
    domains = domains[domains['count'] >= 100].copy()
    # Filter domains with >= 10000 appearances
    #domains_filtered = domains[domains['count'] >= 10000].copy()
    #print(f"Domains with >=10000 appearances: {len(domains_filtered):,}")
    
    client = openai.OpenAI(api_key=api_key)
    
    print("\nCategorizing domains...")
    domains_categorized = categorize_domains_batch(
        domains, 
        client,
        delay=0.1 
    )
    
    # Save results
    output_file = repo_root/'sponsoreddomains_unshortened_final_v2.csv'
    domains_categorized.to_csv(output_file, index=False)
    print(f"\nResults saved to {output_file}")

    return domains_categorized

if __name__ == "__main__":
    domains_categorized = main()