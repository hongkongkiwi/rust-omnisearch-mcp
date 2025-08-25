# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of omnisearch-mcp Rust implementation
- Support for 13+ search providers and AI services
- Comprehensive test suite with 100+ tests
- GitHub Actions workflows for CI/CD and releases
- Cross-platform support (Linux, macOS, Windows)
- Automatic provider detection based on available API keys
- Detailed documentation for all providers
- Usage instructions for popular AI coding tools:
  - Claude Code (Anthropic)
  - Cursor
  - Windsurf (Codeium)
  - Cline (VSCode Extension)
  - Qwen Coder
  - Crush
  - Codex
  - OpenCoder
  - Gemini CLI

### Search Providers
- Tavily Search API (factual queries with citations)
- Google Custom Search
- Reddit Search (OAuth2 support)
- DuckDuckGo (no API key required)
- Baidu (via SerpApi)
- Bright Data Search
- Exa Search (AI-optimized)
- Brave Search

### AI Services
- Perplexity AI responses
- Kagi FastGPT
- Kagi Summarizer
- Jina Reader (URL to markdown)
- Jina Grounding (fact verification)
- Kagi Enrichment

### Content Processing
- Firecrawl scraping suite
- Tavily Extract
- Multiple extraction depth options
- Batch URL processing

## [0.1.0] - TBD

Initial release.
