<h1 align="center">
    How indie YouTubers became industry players
    <br><br>
    <img src="./assets/shenanigans.svg" alt="sheNaNigans", style="width: 20rem;">
</h1>

> _Analysing duration, uploads, and sponsorship trends over time._

## Abstract

Is there a better way to analyse social media professionalisation than the
YouTube case? On this platform, the creator economy has clearly evolved from
indie uploads to established businesses. We want to quantify and rigorously
analyse these patterns.

This project will investigate the professionalisation of YouTube creators --
analysing the journey from indie channels to industry-like settings. We will
first focus on temporal patterns in three key creator metrics: **video
duration**, **upload frequency**, and **engagement (views, likes, comments)**.
We will then dive into the **professionalisation of indie creators**. Using a
large panel of [**YouTube data**](https://github.com/epfl-dlab/YouNiverse)
spanning from 2005 to 2019, we will quantify how and when creators adopt
professional strategies, and what the effects are on their channels and
audiences. To do this, we identified
[**3 main research questions**](#research-questions) that we aim to answer with
**reproducible, data-driven methods**.

## Research Questions

### How have channels evolved from indie to professional?

- What is the **adoption curve** of sponsorships between different categories?
- When and why do we have a **rise of sponsorships**?
- Have **advertisement strategies** changed over time?
- Do **different channel sizes** employ **different strategies** (_e.g._
  multiple sponsors, recurring sponsors, brand partnerships)?

### What enabled indie channels to become professional-minded?

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

### How does a focus on professionalism affect engagement?

- Does the **number of views**, **subscriber growth**, **like/dislike ratio**
  change **after the first sponsor** appears? How?
- Is it **different between categories**? Is it **different between the years**?
- Do those metrics **change over time** (track **monetised vs non monetised
  channels**)?

## Dataset enrichment

- **Sponsor segment analysis**: Use the
  [**SponsorBlock**](https://github.com/ajayyy/SponsorBlock) crowdsourced
  dataset of sponsored videos: which videos are labelled as having **in-video
  sponsor segments**, what **kind of segments** are they (self-promotion, ad
  read, _etc._), **when** do the sponsor segments occur?

<!-- prettier-ignore-start -->
> [!NOTE]
> Limitations: the crowdsourced dataset started in 2019, thus early videos might
> not be labelled. The project consists in a chrome extension, which might
> attract a tech-savier audience and skew the video topics.
<!-- prettier-ignore-end -->

- **Unshortening URLs**: many URLs in the dataset correspond to shortened URLs
  (_e.g._ [bit.ly](https://bitly.com/)). We could unshorten these by using `GET`
  requests and analysing the responses.

<!-- prettier-ignore-start -->
> [!NOTE]
> Limitations: resolving these links might be unfeasible due to the volume of
> requests to make and parse.
<!-- prettier-ignore-end -->

- **Channel and video activity tracking**: check upload activity for all
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

1- Start with URL vs No-URL by Category and Year

For each category and year, calculate how many videos include at least one external URL compared with those that do not. Visualize these trends over time to see when URL usage begins to increase or accelerate within each category.

2-Classify Monetisation Types

Each URL is labelled by its purpose—affiliate, donation, crowdfunding, store, or other. A hybrid strategy is used: regular expressions for known domains, a LangChain LLM for unclear cases, and manual labelling when needed. Categories can also evolve over time as link usage changes.

3- Analyse Monetised Videos

For each category and year, calculate the share and count of different monetisation types (e.g., Amazon affiliate, other affiliate, donations) and track how these shares grow over time. Plot cumulative adoption curves showing when channels first begin using each type of link.

4-Track Adoption and Professionalisation

Analyse when channels in each category first start using URLs, donations, and affiliate links, and compare adoption timing across channel sizes. Estimate the subscriber and behaviour levels at which consistent uploads and longer videos begin to appear as signs of professionalisation.

5- Assess Impact and Early Signals

Compare views, subscriber growth, and engagement before and after the first monetised link to measure its effect, and analyse early behavioural patterns such as upload rhythm and runtime to see which factors predict upcoming monetisation.  Use the YouTube Data API to see if videos or channels are active, inactive, private, or deleted. For example, removing old videos or adding new external links to earlier uploads can indicate that creators are revising their content and monetisation strategies as they become more professional.

## Proposed timeline

```mermaid
gantt
    title ada project timeline
    dateFormat YYYY-MM-DD
    section research
        temporal evolution :a1, 2025-11-5, 7d
        research questions :after a1, 2025-12-10
    section website
        website            :2025-11-26, 2025-12-17
```

## Organisation within the team

- _Ender Sari_:

- _Kalan Walmsley_:

- _Veronika Wannack_:

- _Jan Tomasz Juraszek_:

- _Danael Robert-Nicoud_:
