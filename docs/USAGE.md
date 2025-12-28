# KSeF MCP Server - Usage Guide

This guide covers installation, configuration, and usage of the KSeF MCP Server.

## Table of Contents

- [Installation](#installation)
- [Configuration](#configuration)
- [Using with MCP Clients](#using-with-mcp-clients)
- [Tool Reference](#tool-reference)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Installation

### From Source

1. **Clone the repository:**
   ```bash
   git clone https://github.com/Flippico/ksef-mcp.git
   cd ksef-mcp
   ```

2. **Build the server:**
   ```bash
   cargo build --release -p ksef-mcp-server
   ```

3. **Binary location:**
   ```
   target/release/ksef-mcp
   ```

### Using Cargo Install

Once published to crates.io:

```bash
cargo install ksef-mcp-server
```

The binary will be installed to `~/.cargo/bin/ksef-mcp`

### Pre-built Binaries

Download pre-built binaries from GitHub releases:
- macOS (Intel): `ksef-mcp-x86_64-apple-darwin`
- macOS (Apple Silicon): `ksef-mcp-aarch64-apple-darwin`
- Linux: `ksef-mcp-x86_64-unknown-linux-gnu`
- Windows: `ksef-mcp-x86_64-pc-windows-msvc.exe`

Make the binary executable (Unix-like systems):
```bash
chmod +x ksef-mcp
```

## Configuration

### MCP Client Configuration

The server is configured through your MCP client's configuration file.

#### Claude Desktop

**Location:**
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`
- Linux: `~/.config/Claude/claude_desktop_config.json`

**Configuration:**
```json
{
  "mcpServers": {
    "ksef": {
      "command": "/absolute/path/to/ksef-mcp",
      "args": [],
      "env": {}
    }
  }
}
```

**Important:** Use absolute paths, not relative paths or `~`.

#### Other MCP Clients

For other MCP-compatible clients, configure them to execute:
```bash
/path/to/ksef-mcp
```

The server communicates via stdio using JSON-RPC 2.0.

### Environment Variables

Optional environment variables (not yet implemented, planned for future versions):

```bash
# API endpoint override
export KSEF_API_URL="https://api.ksef.mf.gov.pl/v2"

# Session token
export KSEF_SESSION_TOKEN="your-session-token"

# Debug logging
export KSEF_LOG_LEVEL="debug"
```

## Using with MCP Clients

### Claude Desktop

1. **Configure the server** (see above)

2. **Restart Claude Desktop** completely

3. **Verify connection:**
   - Look for the tools icon (🔨) in Claude Desktop
   - Click to see available tools
   - You should see 12 KSeF tools

4. **Use natural language:**
   ```
   "Check my KSeF API rate limits"
   "Get the public key certificates"
   "List my active sessions"
   ```

### Direct JSON-RPC Usage

For testing or integration:

```bash
# Initialize
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | ./ksef-mcp

# List tools
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | ./ksef-mcp

# Call a tool
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_rate_limits","arguments":{}}}' | ./ksef-mcp
```

## Tool Reference

### Session Management Tools

#### get_active_sessions

Get a list of active authentication sessions.

**Parameters:**
- `pageSize` (integer, optional): 10-100, default 10
- `continuationToken` (string, optional): For pagination

**Example:**
```json
{
  "pageSize": 25,
  "continuationToken": "abc123..."
}
```

#### get_current_session

Get information about the current active session.

**Parameters:** None

#### terminate_session

Terminate a specific session.

**Parameters:**
- `referenceNumber` (string, required): Session reference number

**Example:**
```json
{
  "referenceNumber": "20251228-AU-1234567890-ABCDEF1234-56"
}
```

### Invoice Operations

#### get_invoice

Retrieve invoice details by KSeF number.

**Parameters:**
- `ksefNumber` (string, required): KSeF invoice identifier

**Example:**
```json
{
  "ksefNumber": "1234567890123-20251228-0F5A3B7C9D-E1"
}
```

#### query_invoice_metadata

Query invoice metadata with filtering options.

**Parameters:**
- `queryType` (string, required): Type of query
- `pageSize` (integer, optional): 10-100
- `continuationToken` (string, optional): Pagination token

**Example:**
```json
{
  "queryType": "incremental",
  "pageSize": 50
}
```

#### create_invoice_export

Create an export of invoices.

**Parameters:**
- `exportType` (string, required): Export type
- `parameters` (object, optional): Export parameters

**Example:**
```json
{
  "exportType": "full",
  "parameters": {
    "dateFrom": "2025-01-01",
    "dateTo": "2025-12-31"
  }
}
```

#### get_export_status

Get the status of an invoice export.

**Parameters:**
- `referenceNumber` (string, required): Export reference number

**Example:**
```json
{
  "referenceNumber": "20251228-EX-1234567890-ABCDEF1234-56"
}
```

### Online Session Operations

#### create_online_session

Create a new online session for invoice processing.

**Parameters:**
- `sessionType` (string, optional): Session type

**Example:**
```json
{
  "sessionType": "standard"
}
```

#### close_online_session

Close an active online session.

**Parameters:**
- `referenceNumber` (string, required): Session reference number

**Example:**
```json
{
  "referenceNumber": "20251228-SE-1234567890-ABCDEF1234-56"
}
```

#### submit_invoice

Submit an invoice to an active session.

**Parameters:**
- `sessionReferenceNumber` (string, required): Session reference
- `invoiceData` (string, required): Invoice XML data

**Example:**
```json
{
  "sessionReferenceNumber": "20251228-SE-1234567890-ABCDEF1234-56",
  "invoiceData": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Invoice>...</Invoice>"
}
```

### System Information

#### get_public_key_certificates

Get Ministry of Finance public key certificates.

**Parameters:** None

**Use Case:** Retrieve current encryption/verification certificates.

#### get_rate_limits

Get current API rate limits and usage.

**Parameters:** None

**Use Case:** Monitor API quota and remaining requests.

## Examples

### Example 1: Checking Rate Limits

**In Claude Desktop:**
```
Check my KSeF API rate limits
```

**Direct JSON-RPC:**
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_rate_limits","arguments":{}}}' | ./ksef-mcp
```

### Example 2: Getting Public Certificates

**In Claude Desktop:**
```
Get the KSeF public key certificates
```

**Direct JSON-RPC:**
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_public_key_certificates","arguments":{}}}' | ./ksef-mcp
```

### Example 3: Listing Active Sessions

**In Claude Desktop:**
```
Show me my active KSeF sessions
```

**Direct JSON-RPC:**
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_active_sessions","arguments":{"pageSize":20}}}' | ./ksef-mcp
```

### Example 4: Retrieving an Invoice

**In Claude Desktop:**
```
Get invoice details for KSeF number 1234567890123-20251228-0F5A3B7C9D-E1
```

**Direct JSON-RPC:**
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_invoice","arguments":{"ksefNumber":"1234567890123-20251228-0F5A3B7C9D-E1"}}}' | ./ksef-mcp
```

## Troubleshooting

### Server Not Starting

**Problem:** Server doesn't appear in Claude Desktop tools.

**Solutions:**
1. Check binary path is absolute
2. Verify binary is executable: `chmod +x /path/to/ksef-mcp`
3. Test manually: `./ksef-mcp`
4. Check config file syntax (valid JSON)
5. Restart Claude Desktop completely

### Authentication Errors

**Problem:** Getting 401 Unauthorized errors.

**Solution:**
Most KSeF endpoints require authentication. The current version doesn't include authentication tools. You need to:
1. Authenticate with KSeF separately
2. Obtain a session token
3. Wait for authentication support in future version

### Tools Not Visible

**Problem:** Tools don't appear in Claude Desktop.

**Solutions:**
1. Verify server is running (check Activity Monitor/Task Manager)
2. Check Claude Desktop logs
3. Ensure config file is in correct location
4. Restart Claude Desktop after config changes

### Network Errors

**Problem:** Cannot connect to KSeF API.

**Solutions:**
1. Check internet connection
2. Verify firewall isn't blocking connections
3. Test API endpoint: `curl https://api-test.ksef.mf.gov.pl/v2/`
4. Check proxy settings if behind corporate proxy

### Invalid Tool Arguments

**Problem:** Tool calls fail with validation errors.

**Solutions:**
1. Check parameter types match specification
2. Ensure required parameters are provided
3. Verify date formats: `YYYY-MM-DD`
4. Check reference number format matches KSeF pattern

## Advanced Usage

### Custom API Endpoint

To use production KSeF API instead of test:

1. Modify `crates/ksef-client/src/lib.rs`
2. Change `DEFAULT_API_BASE_URL` to:
   ```rust
   const DEFAULT_API_BASE_URL: &str = "https://api.ksef.mf.gov.pl/v2";
   ```
3. Rebuild: `cargo build --release -p ksef-mcp-server`

### Debugging

Enable debug output:

```bash
# Set log level (when implemented)
RUST_LOG=debug ./ksef-mcp
```

### Using as a Library

The ksef-client can be used independently:

```rust
use ksef_client::KsefClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = KsefClient::new();
    client.set_session_token("your-token".to_string());

    let certs = client.get_public_key_certificates().await?;
    println!("{}", certs);

    Ok(())
}
```

## Support

- **Issues:** [GitHub Issues](https://github.com/Flippico/ksef-mcp/issues)
- **KSeF Docs:** [github.com/CIRFMF/ksef-docs](https://github.com/CIRFMF/ksef-docs)
- **MCP Spec:** [modelcontextprotocol.io](https://modelcontextprotocol.io)

## Next Steps

- See [API.md](./API.md) for detailed API reference
- See [CONFIGURATION.md](./CONFIGURATION.md) for advanced configuration
- See main [README.md](../README.md) for project overview
