<h1 align="center">
    How indie YouTubers became industry players
    <br><br>
    <img src="./assets/shenanigans.svg" alt="sheNaNigans", style="width: 20rem;">
</h1>

> _Analysing content, upload, and sponsorship trends of youtube channels over time._

## Abstracts

At the beginning of YouTube, creators mainly posted for fun - something to 
share with their friends, or to entertain others. Back then, people didn't
put much thought into making a living out of it. Fast forward to today, and
we have every channel competing to get as much viewership for monetization
and partnering with the biggest sponsors, turning enterntainment into profit.
Clearly, the creator focus has evolved from entertaining indie content to 
establishing a profitable brand, which is a fantastic representation of 
professionalisation on social media. Our goal is to quantify and analyse 
these evolution patterns.

This project will thus **investigate the professionalisation of YouTube channels** -
analysing their journey from indie to industry-like creators. Using a large set 
of [**YouTube data**](https://github.com/epfl-dlab/YouNiverse) spanning from 2005
to 2019, our first step will be on **temporal patterns** in three key creator metrics: 
*video duration*, *upload frequency*, and *engagement (views, likes, comments)*, 
in order to get a good first glance into the data, to serve as an introduction.

We will then focus on the **professionalisation of channels**, where we will quantify
***when*** we can see clear transitions in adopting professional strategies, analyse ***how*** the 
creators were able to achieve this, and explore the ***effect*** of this on the channels and
audiences. To achieve this, we identified [**3 main research questions**](#research-questions) 
that we aim to answer with **reproducible, data-driven methods**.

## Research Questions

### 1. How have channels evolved from indie to professional?

- What is the **adoption curve** of sponsorships between different categories?
- When and why do we have a **rise of sponsorships**?
- Have **advertisement strategies** changed over time?
- Do **different channel sizes** employ **different strategies** (_e.g._
  multiple sponsors, recurring sponsors, brand partnerships)?

### 2. What enabled indie channels to become professional-minded?

- At what **subscriber thresholds** do creators start showing **"industry-like"
  patterns** (sponsors, steady uploads, longer videos, higher engagement
  ratios)?
- How does being sponsored **change the channel**? Is channel **growth** (weekly
  subscribers / views) correlated with **production investment signals** such as
  upload density or runtime? Any other metrics (tags / length of title)?
- Are there **early behavioral indicators** (upload rhythm, engagement) that
  predict **professionalisation**?
- Are channel/video **categories and sponsors correlated**?
- Are there **cohorts of channels** with the same/similar sponsors?
- What are the **differences** between categories for **"professional"
  channels** (_e.g._ > 100k subscribers in sports vs education)? How are they
  different?
- Do **"overnight successes"** professionalise faster than slow, organic
  growers?
- Can we detect signs of professionalism by analysing **which videos were later
  removed**, using the
  [**YouTube API**](https://developers.google.com/youtube/v3)?

### 3. How does a focus on professionalism affect engagement?

- Does the **number of views**, **subscriber growth**, **like/dislike ratio**
  change **after the first sponsor** appears? How?
- Is it **different between categories**? Is it **different between the years**?
- Do those metrics **change over time** (track **monetised vs non monetised
  channels**)?

## Dataset enrichment

- #### **Sponsor segment analysis**   
  Use the [**SponsorBlock**](https://github.com/ajayyy/SponsorBlock) crowdsourced
  dataset of sponsored videos: which videos are labelled as having **in-video
  sponsor segments**, what **kind of segments** are they (self-promotion, ad
  read, _etc._), **when** do the sponsor segments occur?

<!-- prettier-ignore-start -->
> [!NOTE]
> *Limitations: the crowdsourced dataset started in 2019, thus early videos might
> not be labelled. The project consists in a chrome extension, which might
> attract a tech-savier audience and skew the video topics.
<!-- prettier-ignore-end -->

- #### **Unshortening URLs**:
  Many URLs in the dataset correspond to shortened URLs
  (_e.g._ [bit.ly](https://bitly.com/)). We could unshorten these by using `GET`
  requests and analysing the responses. That way we have access to nearly all URLs available.

<!-- prettier-ignore-start -->
> [!NOTE]
> Limitations: resolving these links might be unfeasible due to the volume of
> requests to make and parse.
<!-- prettier-ignore-end -->

- **Channel and video activity tracking**:    
  Check upload activity for all
  channels and re-query videos to flag active, inactive, deleted, or private
  content using the [**YouTube API**](https://developers.google.com/youtube/v3).
  This would enable comparing of stability between professional and indie
  creators.

<!-- prettier-ignore-start -->
> [!NOTE]
> Limitations: all channels/videos might not be able to be checked due to API
> limits.
<!-- prettier-ignore-end -->

## Methods

#### Data pre-processing:
- Clean and normalise the data such as empty dates, integers and strings missing
- Split the data for different categories
- Link extraction: Extract URLs from descriptions for each video, and unshorten certain URLs.
- Merge with other datasets, such as labelling certain videos as being sponsored by using SponsorBlock

#### Proffesionalism Analysis
-  Temporal analysis:
  - Use time series with moving averages to analyse trends in subscriber counts, views, etc
  - Check how different categories evolved over time
-  Sponsorship detection:
  - Map videos that have sponsors using the SponsorBlock labels,
  - Check if URLs contain links to product sale like amazon, or have donation links like paypal/patreon (using text and regex matching)
  - Combine upload regularity, count and video lengths to indicate if a channel tries to be more professional 
  - Use the same methods as for temporal analysis but concerning proffesional traits

#### Visualisation
- Create comparisons between categories, channels, and professionalism traits
- Temporal trend plots, 

## Proposed timeline

```mermaid
gantt
    title sheNANigans ADA Project Timeline
    dateFormat YYYY-MM-DD
    tickInterval 1week
    section exploration/analysis
        Temporal Evolution : 2025-11-5, 14d
        Research Question 1 : 2025-11-12, 14d
        Research Question 2 : 2025-11-19, 14d
        Research Question 3 : 2025-11-26, 14d
    section website implementation
        Foundations        : 2025-11-5, 7d
        Add temporal analysis  : 2025-11-12, 7d
        Add RQ 1 analysis  : 2025-11-19, 7d
        Add RQ 2 analysis  : 2025-11-26, 7d
        Add RQ 3 analysis  : 2025-12-3, 7d
        Add story telling  : 2025-12-3, 10d
        Finish up          : 2025-12-10, 2025-12-17
```

## Organisation within the team

- _Ender Sari_:

- _Kalan Walmsley_:

- _Veronika Wannack_:

- _Jan Tomasz Juraszek_:

- _Danael Robert-Nicoud_:
