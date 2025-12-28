# KSeF MCP Server - Configuration Guide

This guide covers all configuration options for the KSeF MCP Server.

## Table of Contents

- [MCP Client Configuration](#mcp-client-configuration)
- [Environment Variables](#environment-variables)
- [API Endpoint Configuration](#api-endpoint-configuration)
- [Authentication Setup](#authentication-setup)
- [Advanced Configuration](#advanced-configuration)
- [Platform-Specific Setup](#platform-specific-setup)

## MCP Client Configuration

### Claude Desktop

#### macOS

**Config Location:**
```
~/Library/Application Support/Claude/claude_desktop_config.json
```

**Basic Configuration:**
```json
{
  "mcpServers": {
    "ksef": {
      "command": "/Users/username/ksef-mcp/target/release/ksef-mcp"
    }
  }
}
```

**With Additional Options:**
```json
{
  "mcpServers": {
    "ksef": {
      "command": "/Users/username/ksef-mcp/target/release/ksef-mcp",
      "args": [],
      "env": {
        "KSEF_API_URL": "https://api-test.ksef.mf.gov.pl/v2"
      }
    }
  }
}
```

#### Windows

**Config Location:**
```
%APPDATA%\Claude\claude_desktop_config.json
```

**Configuration:**
```json
{
  "mcpServers": {
    "ksef": {
      "command": "C:\\Users\\username\\ksef-mcp\\target\\release\\ksef-mcp.exe"
    }
  }
}
```

**Important:** Use double backslashes `\\` in Windows paths.

#### Linux

**Config Location:**
```
~/.config/Claude/claude_desktop_config.json
```

**Configuration:**
```json
{
  "mcpServers": {
    "ksef": {
      "command": "/home/username/ksef-mcp/target/release/ksef-mcp"
    }
  }
}
```

### Other MCP Clients

For other MCP-compatible clients, refer to their documentation for configuring stdio-based MCP servers.

**Standard Configuration:**
- **Protocol:** stdio
- **Transport:** JSON-RPC 2.0
- **Command:** Path to `ksef-mcp` binary

## Environment Variables

Future versions will support environment variable configuration. Planned variables:

### KSEF_API_URL

Override the default KSeF API endpoint.

**Values:**
- `https://api-test.ksef.mf.gov.pl/v2` - Test environment (default)
- `https://api.ksef.mf.gov.pl/v2` - Production environment

**Example:**
```bash
export KSEF_API_URL="https://api.ksef.mf.gov.pl/v2"
```

**In MCP Config:**
```json
{
  "mcpServers": {
    "ksef": {
      "command": "/path/to/ksef-mcp",
      "env": {
        "KSEF_API_URL": "https://api.ksef.mf.gov.pl/v2"
      }
    }
  }
}
```

### KSEF_SESSION_TOKEN

Pre-configure a session token for authentication.

**Example:**
```bash
export KSEF_SESSION_TOKEN="your-session-token-here"
```

**In MCP Config:**
```json
{
  "mcpServers": {
    "ksef": {
      "command": "/path/to/ksef-mcp",
      "env": {
        "KSEF_SESSION_TOKEN": "your-session-token-here"
      }
    }
  }
}
```

### KSEF_LOG_LEVEL

Control logging verbosity (planned feature).

**Values:**
- `error` - Only errors
- `warn` - Warnings and errors
- `info` - Informational messages (default)
- `debug` - Detailed debugging
- `trace` - Very detailed tracing

**Example:**
```bash
export KSEF_LOG_LEVEL="debug"
```

### RUST_LOG

Standard Rust logging configuration.

**Example:**
```bash
export RUST_LOG="ksef_mcp_server=debug,ksef_client=info"
```

## API Endpoint Configuration

### Switching Between Test and Production

The server defaults to the KSeF test environment. To use production:

#### Option 1: Environment Variable (Future)

```json
{
  "mcpServers": {
    "ksef": {
      "command": "/path/to/ksef-mcp",
      "env": {
        "KSEF_API_URL": "https://api.ksef.mf.gov.pl/v2"
      }
    }
  }
}
```

#### Option 2: Code Modification (Current)

1. Open `crates/ksef-client/src/lib.rs`
2. Find `DEFAULT_API_BASE_URL`
3. Change to:
   ```rust
   const DEFAULT_API_BASE_URL: &str = "https://api.ksef.mf.gov.pl/v2";
   ```
4. Rebuild:
   ```bash
   cargo build --release -p ksef-mcp-server
   ```

### Custom API Endpoints

For testing or development with custom endpoints:

**Modify ksef-client:**
```rust
// In crates/ksef-client/src/lib.rs
const DEFAULT_API_BASE_URL: &str = "https://your-custom-endpoint.com/v2";
```

## Authentication Setup

### Session Token Management

Currently, authentication must be handled externally. Future versions will include authentication tools.

**Steps:**
1. Authenticate with KSeF using their standard flow
2. Obtain session token
3. Configure in environment:
   ```bash
   export KSEF_SESSION_TOKEN="your-token"
   ```

### Authentication Methods Supported by KSeF

**1. Token Authentication**
- Requires NIP and token
- Suitable for automated systems

**2. Certificate Authentication**
- Uses qualified digital signature
- Requires certificate file

**3. KSeF Token**
- Special KSeF-issued token
- For authorized integrators

### Storing Credentials Securely

**Best Practices:**
1. Never commit credentials to version control
2. Use environment variables or secret managers
3. Rotate tokens regularly
4. Use separate credentials for test/production

**macOS Keychain:**
```bash
# Store token
security add-generic-password -a "ksef-mcp" -s "ksef-token" -w "your-token"

# Retrieve token
KSEF_SESSION_TOKEN=$(security find-generic-password -a "ksef-mcp" -s "ksef-token" -w)
```

## Advanced Configuration

### Timeout Configuration

Currently hardcoded, planned for future configuration:

```json
{
  "mcpServers": {
    "ksef": {
      "command": "/path/to/ksef-mcp",
      "env": {
        "KSEF_REQUEST_TIMEOUT": "30",
        "KSEF_CONNECT_TIMEOUT": "10"
      }
    }
  }
}
```

### Proxy Configuration

For environments behind corporate proxies:

**HTTP Proxy:**
```bash
export HTTP_PROXY="http://proxy.company.com:8080"
export HTTPS_PROXY="http://proxy.company.com:8080"
```

**In MCP Config:**
```json
{
  "mcpServers": {
    "ksef": {
      "command": "/path/to/ksef-mcp",
      "env": {
        "HTTP_PROXY": "http://proxy.company.com:8080",
        "HTTPS_PROXY": "http://proxy.company.com:8080"
      }
    }
  }
}
```

**With Authentication:**
```bash
export HTTPS_PROXY="http://username:password@proxy.company.com:8080"
```

### TLS/SSL Configuration

The server uses system certificates by default.

**Custom Certificates (Future):**
```json
{
  "mcpServers": {
    "ksef": {
      "command": "/path/to/ksef-mcp",
      "env": {
        "SSL_CERT_FILE": "/path/to/cert.pem",
        "SSL_CERT_DIR": "/path/to/certs"
      }
    }
  }
}
```

## Platform-Specific Setup

### macOS

**Installation:**
```bash
# From Homebrew (when available)
brew install ksef-mcp

# From source
git clone https://github.com/Flippico/ksef-mcp.git
cd ksef-mcp
cargo build --release -p ksef-mcp-server
```

**Binary Location:**
```
/opt/homebrew/bin/ksef-mcp  # Homebrew (Apple Silicon)
/usr/local/bin/ksef-mcp     # Homebrew (Intel)
~/ksef-mcp/target/release/ksef-mcp  # From source
```

**Claude Desktop Config:**
```json
{
  "mcpServers": {
    "ksef": {
      "command": "/opt/homebrew/bin/ksef-mcp"
    }
  }
}
```

### Windows

**Installation:**
```powershell
# From source
git clone https://github.com/Flippico/ksef-mcp.git
cd ksef-mcp
cargo build --release -p ksef-mcp-server
```

**Binary Location:**
```
C:\Users\username\ksef-mcp\target\release\ksef-mcp.exe
```

**Claude Desktop Config:**
```json
{
  "mcpServers": {
    "ksef": {
      "command": "C:\\Users\\username\\ksef-mcp\\target\\release\\ksef-mcp.exe"
    }
  }
}
```

### Linux

**Installation:**
```bash
# From package manager (when available)
# Ubuntu/Debian
sudo apt install ksef-mcp

# Arch
yay -S ksef-mcp

# From source
git clone https://github.com/Flippico/ksef-mcp.git
cd ksef-mcp
cargo build --release -p ksef-mcp-server
```

**Binary Location:**
```
/usr/bin/ksef-mcp  # System package
~/.cargo/bin/ksef-mcp  # Cargo install
~/ksef-mcp/target/release/ksef-mcp  # From source
```

**Claude Desktop Config:**
```json
{
  "mcpServers": {
    "ksef": {
      "command": "/usr/bin/ksef-mcp"
    }
  }
}
```

### Docker

**Dockerfile (Example):**
```dockerfile
FROM rust:1.70 as builder
WORKDIR /build
COPY . .
RUN cargo build --release -p ksef-mcp-server

FROM debian:bookworm-slim
COPY --from=builder /build/target/release/ksef-mcp /usr/local/bin/
CMD ["ksef-mcp"]
```

**Build and Run:**
```bash
docker build -t ksef-mcp .
docker run -i ksef-mcp
```

## Configuration Examples

### Development Environment

```json
{
  "mcpServers": {
    "ksef-test": {
      "command": "/path/to/ksef-mcp",
      "env": {
        "KSEF_API_URL": "https://api-test.ksef.mf.gov.pl/v2",
        "KSEF_LOG_LEVEL": "debug"
      }
    }
  }
}
```

### Production Environment

```json
{
  "mcpServers": {
    "ksef": {
      "command": "/usr/local/bin/ksef-mcp",
      "env": {
        "KSEF_API_URL": "https://api.ksef.mf.gov.pl/v2",
        "KSEF_LOG_LEVEL": "error"
      }
    }
  }
}
```

### Behind Corporate Proxy

```json
{
  "mcpServers": {
    "ksef": {
      "command": "/path/to/ksef-mcp",
      "env": {
        "KSEF_API_URL": "https://api.ksef.mf.gov.pl/v2",
        "HTTPS_PROXY": "http://proxy.company.com:8080",
        "NO_PROXY": "localhost,127.0.0.1"
      }
    }
  }
}
```

### Multiple Instances

```json
{
  "mcpServers": {
    "ksef-test": {
      "command": "/path/to/ksef-mcp",
      "env": {
        "KSEF_API_URL": "https://api-test.ksef.mf.gov.pl/v2"
      }
    },
    "ksef-prod": {
      "command": "/path/to/ksef-mcp",
      "env": {
        "KSEF_API_URL": "https://api.ksef.mf.gov.pl/v2",
        "KSEF_SESSION_TOKEN": "prod-token"
      }
    }
  }
}
```

## Troubleshooting Configuration

### Config File Not Found

**Problem:** Claude Desktop doesn't load configuration.

**Solutions:**
1. Verify config file location
2. Check file permissions
3. Ensure valid JSON syntax
4. Restart Claude Desktop

**Validate JSON:**
```bash
# macOS/Linux
cat ~/Library/Application\ Support/Claude/claude_desktop_config.json | jq .

# Windows (PowerShell)
Get-Content $env:APPDATA\Claude\claude_desktop_config.json | ConvertFrom-Json
```

### Permission Denied

**Problem:** Binary cannot be executed.

**Solution:**
```bash
chmod +x /path/to/ksef-mcp
```

### Path Issues

**Problem:** Binary not found.

**Solutions:**
1. Use absolute paths (not relative or `~`)
2. Check binary exists: `ls -l /path/to/ksef-mcp`
3. Verify architecture matches (arm64 vs x86_64)

### Environment Variables Not Working

**Problem:** Environment variables not passed to server.

**Solution:**
Ensure they're in the `env` object within the MCP config, not system environment.

## Best Practices

1. **Use Absolute Paths:** Never use relative paths or `~` in MCP configs
2. **Validate JSON:** Check syntax before restarting client
3. **Separate Environments:** Use different configs for test/production
4. **Secure Credentials:** Never hardcode tokens in config files
5. **Version Control:** Add config files to `.gitignore`
6. **Documentation:** Document custom configurations for your team

## Future Configuration Options

Planned for future releases:

- Configuration file support (`ksef-mcp.toml`)
- Runtime configuration via MCP resources
- Dynamic API endpoint switching
- Certificate-based authentication
- Request/response caching
- Retry policies
- Custom headers

## See Also

- [Usage Guide](./USAGE.md)
- [API Reference](./API.md)
- [Main README](../README.md)
- [Claude Desktop Documentation](https://claude.ai/desktop)
