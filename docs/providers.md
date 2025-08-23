# Provider Setup Guide

This guide provides detailed setup instructions for each search provider supported by omnisearch-mcp.

## Table of Contents

- [Tavily Search](#tavily-search)
- [Google Custom Search](#google-custom-search)
- [Reddit Search](#reddit-search)
- [DuckDuckGo Search](#duckduckgo-search)
- [Baidu Search](#baidu-search)
- [Bright Data SERP API](#bright-data-serp-api)
- [Exa Search](#exa-search)
- [Perplexity AI](#perplexity-ai)
- [Kagi Services](#kagi-services)
- [Jina AI Services](#jina-ai-services)
- [Brave Search](#brave-search)
- [Firecrawl Services](#firecrawl-services)

---

## Tavily Search

Optimized for factual information with strong citation support.

### Setup
1. Sign up at [Tavily](https://tavily.com)
2. Get your API key from the dashboard
3. Set environment variable: `TAVILY_API_KEY=your-api-key`

### Features
- Strong citation support
- Domain filtering (include_domains/exclude_domains)
- Optimized for technical and academic queries

---

## Google Custom Search

Google's powerful search capabilities with reliable results and snippets.

### Setup
1. Create a project in [Google Cloud Console](https://console.cloud.google.com)
2. Enable Custom Search API
3. Create credentials (API Key)
4. Create a Custom Search Engine at [Programmable Search Engine](https://programmablesearchengine.google.com)
5. Set environment variables:
   - `GOOGLE_API_KEY=your-google-api-key`
   - `GOOGLE_SEARCH_ENGINE_ID=your-search-engine-id`

### Features
- High-quality search results with snippets
- Domain filtering capabilities
- Reliable and fast response times

---

## Reddit Search

Access discussions and content from Reddit communities.

### Setup
1. Go to [Reddit Apps](https://www.reddit.com/prefs/apps)
2. Create a new app (select "script" type)
3. Note your client ID (under "personal use script")
4. Note your client secret
5. Set environment variables:
   - `REDDIT_CLIENT_ID=your-client-id`
   - `REDDIT_CLIENT_SECRET=your-client-secret`
   - `REDDIT_USER_AGENT=YourApp/1.0 by YourUsername`

### Features
- OAuth2 authentication
- Search by relevance, new, top
- Access to all public subreddits

---

## DuckDuckGo Search

Privacy-focused search without tracking.

### Setup
No API key required - works out of the box!

### Features
- Complete privacy - no tracking
- No authentication required
- Fast response times
- Good for general web searches

---

## Baidu Search

Access to China's leading search engine via SerpApi.

### Setup
1. Sign up at [SerpApi](https://serpapi.com)
2. Get your API key from the dashboard
3. Set environment variable: `SERPAPI_API_KEY=your-serpapi-key`

### Features
- Chinese language content
- Access to Baidu search results
- Reliable through SerpApi infrastructure

---

## Bright Data SERP API

High-quality search results with advanced filtering.

### Setup
1. Sign up at [Bright Data](https://brightdata.com)
2. Create a SERP API user
3. Get your credentials
4. Set environment variables:
   - `BRIGHTDATA_USERNAME=your-username`
   - `BRIGHTDATA_PASSWORD=your-password`

### Features
- Advanced filtering options
- High-quality search results
- Enterprise-grade reliability

---

## Exa Search

High-quality search with relevance scoring.

### Setup
1. Sign up at [Exa](https://exa.ai)
2. Get your API key from the dashboard
3. Set environment variable: `EXA_API_KEY=your-exa-key`

### Features
- Relevance scoring for results
- High-quality search results
- Optimized for AI applications

---

## Perplexity AI

Advanced response generation combining real-time web search with AI models.

### Setup
1. Sign up at [Perplexity](https://perplexity.ai)
2. Get your API key from settings
3. Set environment variable: `PERPLEXITY_API_KEY=your-perplexity-key`

### Features
- GPT-4 Omni and Claude 3 integration
- Real-time web search
- Contextual memory for follow-ups

---

## Kagi Services

Multiple services including FastGPT, Universal Summarizer, and Enrichment API.

### Setup
1. Sign up at [Kagi](https://kagi.com)
2. Get your API key from settings
3. Set environment variable: `KAGI_API_KEY=your-kagi-key`

### Services
- **FastGPT**: Quick AI answers (900ms response time)
- **Universal Summarizer**: Summarize pages, videos, podcasts
- **Enrichment API**: Specialized content indexes

---

## Jina AI Services

Content processing and fact verification services.

### Setup
1. Sign up at [Jina AI](https://jina.ai)
2. Get your API key
3. Set environment variable: `JINA_AI_API_KEY=your-jina-key`

### Services
- **Reader**: Clean content extraction with PDF support
- **Grounding**: Real-time fact verification

---

## Brave Search

Privacy-focused search with good technical coverage.

### Setup
1. Sign up at [Brave Search API](https://brave.com/search/api/)
2. Get your API key
3. Set environment variable: `BRAVE_API_KEY=your-brave-key`

### Features
- Privacy protection
- Native search operators support
- Good for technical documentation

---

## Firecrawl Services

Comprehensive web scraping and crawling services.

### Setup
1. Sign up at [Firecrawl](https://firecrawl.dev) or self-host
2. Get your API key
3. Set environment variables:
   - `FIRECRAWL_API_KEY=your-firecrawl-key`
   - `FIRECRAWL_BASE_URL=http://localhost:3002` (optional, for self-hosted)

### Services
- **Scrape**: Extract clean, LLM-ready data
- **Crawl**: Deep crawl websites
- **Map**: Fast URL discovery
- **Extract**: AI-powered structured data extraction
- **Actions**: Interactive content extraction

---

## Tips for Getting Started

1. **Start Small**: You don't need all providers - start with 1-2 that match your needs
2. **Free Tiers**: Many providers offer free tiers perfect for testing:
   - DuckDuckGo (no API key needed)
   - Tavily (1000 searches/month free)
   - SerpApi (100 searches/month free)
3. **Mix and Match**: Combine providers for best results:
   - Use Tavily for factual queries
   - Use Reddit for community discussions
   - Use Jina Reader for content extraction