# Configuration Guide

This guide covers how to configure omnisearch-mcp in different environments.

## Environment Variables

The server uses environment variables for API keys. You only need to set variables for the providers you want to use.

### Core Search Providers

```bash
export TAVILY_API_KEY="your-tavily-key"
export GOOGLE_API_KEY="your-google-key"
export GOOGLE_SEARCH_ENGINE_ID="your-search-engine-id"
export REDDIT_CLIENT_ID="your-reddit-client-id"
export REDDIT_CLIENT_SECRET="your-reddit-client-secret"
export REDDIT_USER_AGENT="YourApp/1.0"
export SERPAPI_API_KEY="your-serpapi-key"
export BRIGHTDATA_USERNAME="your-brightdata-username"
export BRIGHTDATA_PASSWORD="your-brightdata-password"
export EXA_API_KEY="your-exa-key"
export BRAVE_API_KEY="your-brave-key"
```

### AI and Processing Services

```bash
export PERPLEXITY_API_KEY="your-perplexity-key"
export KAGI_API_KEY="your-kagi-key"
export JINA_AI_API_KEY="your-jina-key"
export FIRECRAWL_API_KEY="your-firecrawl-key"
export FIRECRAWL_BASE_URL="http://localhost:3002"  # Optional, for self-hosted
```

## MCP Client Configuration

### Cline

Add to your Cline MCP settings:

```json
{
  "mcpServers": {
    "omnisearch-mcp": {
      "command": "/path/to/omnisearch-mcp",
      "env": {
        "TAVILY_API_KEY": "your-key",
        "GOOGLE_API_KEY": "your-key",
        // Add only the keys you have
      },
      "disabled": false,
      "autoApprove": []
    }
  }
}
```

### Claude Desktop

Add to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "omnisearch-mcp": {
      "command": "/path/to/omnisearch-mcp",
      "env": {
        "TAVILY_API_KEY": "your-key",
        // Add your API keys here
      }
    }
  }
}
```

### VS Code with Continue

Add to your Continue config:

```json
{
  "models": [
    {
      "title": "Claude with Omnisearch",
      "provider": "anthropic",
      "model": "claude-3-sonnet",
      "contextProviders": [
        {
          "name": "mcp",
          "params": {
            "servers": {
              "omnisearch-mcp": {
                "command": "/path/to/omnisearch-mcp",
                "env": {
                  "TAVILY_API_KEY": "your-key"
                  // Add your API keys
                }
              }
            }
          }
        }
      ]
    }
  ]
}
```

## Using .env Files

For development, you can use a `.env` file in your project root:

1. Create a `.env` file:

```bash
TAVILY_API_KEY=your-tavily-key
GOOGLE_API_KEY=your-google-key
# Add other keys as needed
```

2. Load before running:

```bash
source .env && cargo run
```

Or use a tool like `direnv` for automatic loading.

## Docker Configuration

If running in Docker:

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/omnisearch-mcp /usr/local/bin/
CMD ["omnisearch-mcp"]
```

Run with environment variables:

```bash
docker run -e TAVILY_API_KEY=your-key \
           -e GOOGLE_API_KEY=your-key \
           omnisearch-mcp
```

Or with an env file:

```bash
docker run --env-file .env omnisearch-mcp
```

## Systemd Service

For running as a system service on Linux:

1. Create service file `/etc/systemd/system/omnisearch-mcp.service`:

```ini
[Unit]
Description=Omnisearch MCP Server
After=network.target

[Service]
Type=simple
User=your-user
WorkingDirectory=/home/your-user
ExecStart=/usr/local/bin/omnisearch-mcp
Restart=always
Environment="TAVILY_API_KEY=your-key"
Environment="GOOGLE_API_KEY=your-key"
# Add other environment variables

[Install]
WantedBy=multi-user.target
```

2. Enable and start:

```bash
sudo systemctl enable omnisearch-mcp
sudo systemctl start omnisearch-mcp
```

## Security Best Practices

1. **Never commit API keys** to version control
2. **Use environment variables** or secure secret management
3. **Rotate keys regularly** if compromised
4. **Use minimal permissions** - only set keys for providers you need
5. **Consider using secret managers** like:
   - AWS Secrets Manager
   - HashiCorp Vault
   - Azure Key Vault
   - 1Password CLI

Example with 1Password CLI:

```bash
export TAVILY_API_KEY=$(op read "op://vault/tavily/api-key")
```

## Troubleshooting

### Check Available Providers

The server logs which providers are available on startup:

```
[INFO] Available providers:
  ✓ Tavily (API key found)
  ✓ Google (API key found)
  ✗ Reddit (missing credentials)
  ✓ DuckDuckGo (no key required)
```

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug omnisearch-mcp
```

### Common Issues

1. **Provider not available**: Check if API key is set correctly
2. **Authentication errors**: Verify API key is valid and has correct permissions
3. **Rate limiting**: Some providers have rate limits on free tiers
4. **Network issues**: Check firewall and proxy settings
