# Changelog

All notable changes to the KSeF MCP Server will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned

- Authentication tools for session token management
- Configuration file support for API endpoint selection
- Caching for frequently accessed data
- Enhanced error messages and recovery
- Logging and debugging capabilities

## [0.1.0] - 2025-12-28

### Added

- Initial release of KSeF MCP Server
- 12 MCP tools for KSeF API integration:
  - Session Management: `get_active_sessions`, `get_current_session`, `terminate_session`
  - Invoice Operations: `get_invoice`, `query_invoice_metadata`, `create_invoice_export`, `get_export_status`
  - Online Sessions: `create_online_session`, `close_online_session`, `submit_invoice`
  - System Info: `get_public_key_certificates`, `get_rate_limits`
- Modular crate structure with workspace:
  - `mcp-protocol`: MCP/JSON-RPC protocol implementation
  - `ksef-client`: Reusable KSeF API client library
  - `ksef-mcp-server`: Main MCP server binary
- Support for KSeF test environment API
- JSON-RPC 2.0 over stdio communication
- Comprehensive documentation:
  - README with quick start guide
  - Usage guide with examples
  - Publishing guide for mcpservers.org
- MIT License

### Technical Details

- Built with Rust 2021 edition
- Async/await support with Tokio runtime
- HTTP client using reqwest
- JSON serialization with serde
- Error handling with anyhow

## Release Notes

### v0.1.0 - Initial Release

This is the first public release of the KSeF MCP Server. It provides basic integration with the Polish KSeF e-invoicing system through the Model Context Protocol.

**Highlights:**

- Complete API coverage for KSeF v2 endpoints
- Works with any MCP-compatible client (Claude Desktop, etc.)
- Standalone `ksef-client` library for reuse in other projects
- Well-documented with usage examples

**Known Limitations:**

- Authentication must be handled externally
- Only supports KSeF test environment (configurable in code)
- No built-in caching or rate limiting
- Limited error recovery mechanisms

**Getting Started:**

1. Build: `cargo build --release -p ksef-mcp-server`
2. Configure in your MCP client
3. Start using the 12 available tools

See the [README](README.md) and [Usage Guide](../../doc/USAGE.md) for more details.

---

[Unreleased]: https://github.com/Flippico/ksef-mcp/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Flippico/ksef-mcp/releases/tag/v0.1.0
