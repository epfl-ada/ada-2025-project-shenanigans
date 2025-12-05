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



SYSTEM_PROMPT = """
You will be given URLS that are found inside Youtube Descriptions. Your goal is to use your knowledge and common sense to categorize them by answering "yes" if you believe the URL has to do with monetization (merch, sponsor url, direct donation page, etc), or "no" if the URL is not for monetization purposes. 

Return ONLY "yes" for monetization or "no" otherwise."""

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
    checkpoint_file: Path = repo_root/ "process" / "categorize_urls_LLM" /'sponsoreddomains_unshortened_v3_testing_yes_no.csv'
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
    domains = domains[domains['count'] >= 10000].copy()
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
    output_file = repo_root/'sponsoreddomains_unshortened_v3_testing_yes_no.csv'
    domains_categorized.to_csv(output_file, index=False)
    print(f"\nResults saved to {output_file}")

    return domains_categorized

if __name__ == "__main__":
    domains_categorized = main()