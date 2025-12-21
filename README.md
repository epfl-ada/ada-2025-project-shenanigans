<h1 align="center">
    How indie YouTubers became industry players
    <br><br>
    <img src="./assets/shenanigans.svg" alt="sheNaNigans", style="width: 20rem;">
</h1>

> _Analysing content, upload, and sponsorship trends of YouTube channels over
> time._

## Abstract

At the beginning of YouTube, creators mainly posted for fun - making something
to entertain others. Back then, people didn't put much thought into making a
living out of it. Today, we have channels competing to get viewership for
monetization and partnering with big sponsors, turning content into profit.
Clearly, the creator focus has evolved from indie content to establishing a
profitable brand, which is a fantastic representation of professionalisation on
social media.

This project **investigated the professionalisation of YouTube channels** -
analysing their journey from indie to industry-like creators. We attempted to
quantify **_when_** we can see clear transitions in adopting professional
strategies, analyse **_how_** the creators were able to achieve this, and
explore the **_effect_** of this on the channels and audiences.

## Research Questions

### 1. How have channels evolved from indie to professional?

- How did the **YouTube ecosystem change** as it grew bigger?
- Did all categories **grow in the same way?**
- How did **different types of links** (content, social, monetization) spread
  and change over time?
- Which categories used **monetization links** the most?
- Did channels become professional all at once or gradually?

### 2. What enabled indie channels to become professional-minded?

- **How long** does one need to have a channel to obtain a sponsor?
- **At what thresholds** (views, subscribers, videos, activity) do creators
  obtain their first sponsorship?
- Are certain **topic transitions** common after the first sponsorship?
- How does getting sponsored affect **channel longevity**?
- Are there **cohorts of channels** with the same/similar sponsors?

### 3. How does a focus on professionalism affect engagement?

- Are **like/dislike ratios**, **video durations**, **view counts** different
  between sponsored and unsponsored channels? Is it **different between
  categories**?
- Do the **number of views** and the **subscriber growth** change **after
  channels get sponsored**?

## Dataset enrichment

### Sponsor segment analysis

We used the [**SponsorBlock**](https://github.com/ajayyy/SponsorBlock)
crowdsourced dataset to label videos as sponsored. The crowdsourced dataset
started in 2019, but we found pretty uniform labelling for all years in the
YouNiverse dataset.

### Unshortening URLs

Many URLs in the dataset corresponded to shortened URLs (_e.g._
[bit.ly](https://bitly.com/)). We unshortened the subset of URLs from sponsored
videos.

### Channel and video activity tracking

The activity of channels in 2025 was verified using the
[**YouTube API**](https://developers.google.com/youtube/v3). This would enabled
comparing of stability between professional and indie creators.

## Methods

The bulk of the analysis is detailed [in this notebook](milestone_p3.ipynb).

## Initial timeline

```mermaid
gantt
    title sheNANigans ADA Project Timeline
    dateFormat YYYY-MM-DD
    tickInterval 1week
    section Exploration & Analysis
        Temporal Evolution : 2025-11-5, 14d
        Research Question 1 : 2025-11-12, 14d
        Research Question 2 : 2025-11-19, 14d
        Research Question 3 : 2025-11-26, 14d
    section Website implementation
        Foundations        : 2025-11-5, 7d
        Add temporal analysis  : 2025-11-12, 7d
        Add RQ 1 analysis  : 2025-11-19, 7d
        Add RQ 2 analysis  : 2025-11-26, 7d
        Add RQ 3 analysis  : 2025-12-3, 7d
        Add story telling  : 2025-12-3, 10d
        Finish up          : 2025-12-10, 2025-12-17
```

## Contributions

- _Ender Sari_: Focus on research question 1 and YouTube data enrichment. 
   Website implementation for question 1.
- _Kalan Walmsley_: Performance-intensive data preprocessing. Focus on research
  question 2. Website implementation. WebAssembly graph visualisation.
- _Danael Robert-Nicoud_: Focus on research question 3.
- _Veronika Wannack_: Website design and implementation.
- _Jan Tomasz Juraszek_: Website implementation and story.
