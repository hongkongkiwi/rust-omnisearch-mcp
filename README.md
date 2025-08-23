# omnisearch-mcp

A Model Context Protocol (MCP) server implemented in Rust that provides unified access to multiple search providers and AI tools. This server combines the capabilities of 13+ search providers and AI services to offer comprehensive search, AI responses, content processing, and enhancement features through a single interface.

This is a Rust implementation of the [mcp-omnisearch](https://github.com/spences10/mcp-omnisearch) project.

## Features

- **🔍 Search Providers**: Tavily, Google, Reddit, DuckDuckGo, Baidu, Bright Data, Exa, Brave
- **🤖 AI Services**: Perplexity AI, Kagi FastGPT
- **📄 Content Processing**: Jina Reader, Kagi Summarizer, Tavily Extract, Firecrawl suite
- **🔄 Enhancement Tools**: Kagi Enrichment, Jina Grounding

The server automatically detects available API keys and enables corresponding providers - you only need keys for the services you want to use.

## Quick Start

### Installation

```bash
# From source
git clone https://github.com/hongkongkiwi/rust-omnisearch-mcp.git
cd rust-omnisearch-mcp
cargo build --release

# Or using Cargo
cargo install omnisearch-mcp
```

### Basic Configuration

1. Set environment variables for the providers you want to use:

```bash
export TAVILY_API_KEY="your-tavily-key"
export GOOGLE_API_KEY="your-google-key"
# Add other keys as needed
```

2. Run the server:

```bash
omnisearch-mcp
```

## Usage with AI Coding Tools

### Claude Code (Anthropic)

Add to your Claude Code configuration:

```json
{
  "mcpServers": {
    "omnisearch-mcp": {
      "command": "/path/to/omnisearch-mcp",
      "env": {
        "TAVILY_API_KEY": "your-key",
        "GOOGLE_API_KEY": "your-key"
        // Add only the API keys you have
      }
    }
  }
}
```

### Cursor

Add to your Cursor settings (`~/.cursor/settings.json` or via Settings UI):

```json
{
  "mcp": {
    "servers": {
      "omnisearch-mcp": {
        "command": "/path/to/omnisearch-mcp",
        "env": {
          "TAVILY_API_KEY": "your-key"
        }
      }
    }
  }
}
```

### Windsurf (Codeium)

Add to your Windsurf configuration:

```json
{
  "mcpServers": {
    "omnisearch-mcp": {
      "command": "/path/to/omnisearch-mcp",
      "env": {
        "TAVILY_API_KEY": "your-key"
      }
    }
  }
}
```

### Cline (VSCode Extension)

Add to your Cline MCP settings in VSCode:

```json
{
  "mcpServers": {
    "omnisearch-mcp": {
      "command": "/path/to/omnisearch-mcp",
      "env": {
        "TAVILY_API_KEY": "your-key"
      }
    }
  }
}
```

### Qwen Coder

For Qwen Coder integration, add to your configuration file:

```json
{
  "mcp_servers": {
    "omnisearch": {
      "command": "/path/to/omnisearch-mcp",
      "environment": {
        "TAVILY_API_KEY": "your-key"
      }
    }
  }
}
```

### Crush

Add to your Crush configuration (`~/.crush/config.json`):

```json
{
  "mcp": {
    "servers": {
      "omnisearch-mcp": {
        "command": "/path/to/omnisearch-mcp",
        "env": {
          "TAVILY_API_KEY": "your-key",
          "GOOGLE_API_KEY": "your-key"
        }
      }
    }
  }
}
```

### Codex

For Codex MCP integration:

```json
{
  "mcpServers": {
    "omnisearch-mcp": {
      "command": "/path/to/omnisearch-mcp",
      "environment": {
        "TAVILY_API_KEY": "your-key"
      }
    }
  }
}
```

### OpenCoder

Add to your OpenCoder settings:

```json
{
  "mcp_config": {
    "servers": [
      {
        "name": "omnisearch-mcp",
        "command": "/path/to/omnisearch-mcp",
        "env": {
          "TAVILY_API_KEY": "your-key"
        }
      }
    ]
  }
}
```

### Gemini CLI

For Google's Gemini CLI tool:

```bash
# Add to your shell configuration (.bashrc, .zshrc, etc.)
export MCP_OMNISEARCH_PATH="/path/to/omnisearch-mcp"
export TAVILY_API_KEY="your-key"

# Or configure in Gemini settings
gemini config add-mcp omnisearch-mcp --command "/path/to/omnisearch-mcp"
```

### Other MCP-Compatible Tools

For any MCP-compatible tool, the general configuration pattern is:

```json
{
  "mcpServers": {
    "omnisearch-mcp": {
      "command": "/path/to/omnisearch-mcp",
      "env": {
        // Add your API keys here
        "TAVILY_API_KEY": "your-key",
        "GOOGLE_API_KEY": "your-key",
        "REDDIT_CLIENT_ID": "your-id",
        "REDDIT_CLIENT_SECRET": "your-secret"
      }
    }
  }
}
```

### Configuration Tips

1. **Path to executable**: 
   - If installed via cargo: `~/.cargo/bin/omnisearch-mcp`
   - If built from source: `/path/to/rust-omnisearch-mcp/target/release/omnisearch-mcp`

2. **API Keys**: Only include keys for services you want to use. The server automatically enables providers based on available keys.

3. **Testing**: After configuration, restart your AI coding tool and check if the omnisearch tools appear in the available MCP tools list.

## Documentation

- **[Provider Setup Guide](docs/providers.md)** - Detailed setup instructions for each provider
- **[Configuration Guide](docs/configuration.md)** - Complete configuration options and examples
- **[Development Guide](docs/development.md)** - Contributing, testing, and development setup

## Available Providers

### Search Providers
- **Tavily** - Factual search with citations
- **Google Custom Search** - Google's search API
- **Reddit** - Community discussions
- **DuckDuckGo** - Privacy-focused (no API key needed)
- **Baidu** - Chinese search via SerpApi
- **Bright Data** - Enterprise search
- **Exa** - AI-optimized search
- **Brave** - Privacy-focused with operators

### AI & Processing Services
- **Perplexity AI** - GPT-4/Claude with web search
- **Kagi** - FastGPT, Summarizer, Enrichment
- **Jina AI** - Reader, Grounding
- **Firecrawl** - Scraping, crawling, extraction

See the [Provider Setup Guide](docs/providers.md) for detailed configuration of each provider.

## Development

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Check code quality
cargo clippy
cargo fmt
```

See the [Development Guide](docs/development.md) for more details.

## License

MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Built on the [Model Context Protocol](https://github.com/modelcontextprotocol) and powered by multiple search and AI providers.