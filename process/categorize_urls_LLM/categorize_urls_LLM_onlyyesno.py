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



SYSTEM_PROMPT = """You are classifying URLs found in YouTube video descriptions.

Classify as "yes" (MONETIZATION) if the URL is likely used by the creator to earn money:
- Amazon links (amazon.com, amazon.de, amazon.co.uk, amzn.to, etc.) - creators use affiliate programs
- Affiliate/referral links and link shorteners that typically mask affiliate links
- Merchandise stores (teespring, spreadshirt, merch sites)
- Direct support/donation platforms (patreon, ko-fi, buymeacoffee, paypal.me)
- Sponsor landing pages or promo codes
- Creator's own product/course sales pages

Classify as "no" (NOT MONETIZATION) if the URL is:
- Social media profiles (twitter, instagram, facebook, tiktok)
- Community links (discord, reddit)
- Video platforms for content (youtube, twitch, youtu.be)
- Free content/streaming platforms (soundcloud, spotify)
- News/reference sites (wsj, wikipedia)
- General information or attribution (creativecommons.org)
- App store links

Respond with ONLY "yes" or "no"."""

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
    checkpoint_file: Path = repo_root/ "process" / "categorize_urls_LLM" /'sponsoreddomains_unshortened_v4_full_data.csv'
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
    repo_root/"dataset"/"full_urls.csv", 
    header=None,  
    names=['domain', 'count']  
    )
    
    #>100 appearances = 1.6k urls 
    #domains = domains[domains['count'] >= 10000].copy()
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
    output_file = repo_root/'sponsoreddomains_unshortened_v4_full_data.csv'
    domains_categorized.to_csv(output_file, index=False)
    print(f"\nResults saved to {output_file}")

    return domains_categorized

if __name__ == "__main__":
    domains_categorized = main()