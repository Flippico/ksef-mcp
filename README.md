# KSeF MCP Server

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

MCP (Model Context Protocol) server for the Polish national e-invoicing system KSeF (Krajowy System e-Faktur).

## Overview

This server enables AI assistants and automation tools to interact with the KSeF API, providing seamless integration with Poland's mandatory e-invoicing system operated by the Ministry of Finance.

**Key Features:**
- 12 MCP tools for complete KSeF API coverage
- Authentication session management
- Invoice querying, retrieval, and submission
- Export generation and status tracking
- System monitoring and rate limits
- Built with Rust for reliability and performance

## Quick Start

### Installation

**From source:**
```bash
git clone https://github.com/YOUR_USERNAME/ksef-mcp.git
cd ksef-mcp
cargo build --release
```

The binary will be at: `target/release/ksef-mcp`

**Using cargo install (when published):**
```bash
cargo install ksef-mcp-server
```

### Configuration

Add to your MCP client configuration (e.g., Claude Desktop):

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "ksef": {
      "command": "/absolute/path/to/ksef-mcp"
    }
  }
}
```

Restart your MCP client after configuration.

## Available Tools

### Session Management
- **get_active_sessions** - List active authentication sessions
- **get_current_session** - Get current session information
- **terminate_session** - Terminate a specific session

### Invoice Operations
- **get_invoice** - Retrieve invoice details by KSeF number
- **query_invoice_metadata** - Query invoice metadata with filters
- **create_invoice_export** - Create invoice exports
- **get_export_status** - Check export status

### Online Sessions
- **create_online_session** - Create new online session for invoice processing
- **close_online_session** - Close an active online session
- **submit_invoice** - Submit invoice XML to a session

### System Information
- **get_public_key_certificates** - Get Ministry of Finance public certificates
- **get_rate_limits** - Check API rate limits and usage

## Usage Examples

### In Claude Desktop

Once configured, you can interact naturally:

```
"Check my KSeF API rate limits"
"Get the public key certificates from KSeF"
"Query invoice metadata for the last 50 invoices"
"Get details for invoice 1234567890123-20231201-0F5A3B7C9D-E1"
```

### Direct API Usage

The server implements JSON-RPC 2.0 over stdio:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_rate_limits","arguments":{}}}' | ./ksef-mcp
```

## API Endpoint

The server connects to the KSeF test environment by default:
- **Test API:** `https://api-test.ksef.mf.gov.pl/v2`

For production use, modify the `DEFAULT_API_BASE_URL` in the ksef-client library.

## Authentication

Most KSeF endpoints require authentication via session token. The current version expects:
1. Authenticate with KSeF API through standard authentication flow
2. Obtain a session token
3. The server will use the token for API requests

**Note:** Authentication tools will be added in a future version.

## Project Structure

This server is part of a Cargo workspace:

```
ksef_mcp/
├── crates/
│   ├── mcp-protocol/      # MCP/JSON-RPC protocol implementation
│   ├── ksef-client/       # Reusable KSeF API client library
│   └── ksef-mcp-server/   # This MCP server binary
└── doc/                   # Documentation
```

The `ksef-client` library can be used independently in other Rust projects.

## Development

### Build

```bash
cargo build --release -p ksef-mcp-server
```

### Test

```bash
cargo test
```

### Lint

```bash
cargo clippy --all-targets --all-features
```

## Documentation

- [Usage Guide](../../doc/USAGE.md) - Detailed usage instructions
- [Publishing Guide](../../doc/PUBLISHING.md) - How to publish to mcpservers.org
- [KSeF API Docs](https://github.com/CIRFMF/ksef-docs) - Official KSeF API documentation
- [MCP Specification](https://modelcontextprotocol.io) - Model Context Protocol

## Requirements

- Rust 1.70 or later
- Internet connection to KSeF API
- Valid KSeF credentials for authenticated endpoints

## Use Cases

Perfect for:
- Polish businesses automating e-invoicing processes
- Integration with AI assistants for invoice management
- Building automation tools for KSeF compliance
- Developers needing programmatic access to KSeF

## Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

For bugs, open an issue with:
- Steps to reproduce
- Expected vs actual behavior
- System information

## License

MIT License - see [LICENSE](./LICENSE) file for details.

## Support

- **Issues:** [GitHub Issues](https://github.com/YOUR_USERNAME/ksef-mcp/issues)
- **KSeF API:** [Official Documentation](https://github.com/CIRFMF/ksef-docs)
- **MCP Protocol:** [modelcontextprotocol.io](https://modelcontextprotocol.io)

## Disclaimer

This is an unofficial integration with the Polish KSeF system. It is not affiliated with or endorsed by the Polish Ministry of Finance.
