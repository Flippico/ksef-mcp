# KSeF MCP Server - API Reference

Complete reference for all MCP tools provided by the KSeF MCP Server.

## Table of Contents

- [Overview](#overview)
- [Authentication](#authentication)
- [Session Management](#session-management)
- [Invoice Operations](#invoice-operations)
- [Online Sessions](#online-sessions)
- [System Information](#system-information)
- [Error Handling](#error-handling)
- [Rate Limits](#rate-limits)

## Overview

The KSeF MCP Server provides 12 tools that map to the KSeF API v2 endpoints:

| Category | Tools Count | Authentication Required |
|----------|-------------|------------------------|
| Session Management | 3 | Yes |
| Invoice Operations | 4 | Yes |
| Online Sessions | 3 | Yes |
| System Information | 2 | No |

**API Base URL:**
- Test: `https://api-test.ksef.mf.gov.pl/v2`
- Production: `https://api.ksef.mf.gov.pl/v2`

## Authentication

Most tools require authentication via session token. Authentication is handled through KSeF's authentication endpoints (not yet exposed as MCP tools).

**Supported Authentication Methods:**
- Token-based authentication
- Certificate-based authentication (qualified signature)
- KSeF token authentication

**Session Token Format:**
```
Header: SessionToken: {token-value}
```

## Session Management

Tools for managing authentication sessions.

---

### get_active_sessions

Get a paginated list of active authentication sessions.

**Method:** `tools/call`
**Tool Name:** `get_active_sessions`

**Parameters:**

| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| pageSize | integer | No | 10 | Number of results per page (10-100) |
| continuationToken | string | No | - | Token for pagination |

**Request Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_active_sessions",
    "arguments": {
      "pageSize": 25,
      "continuationToken": "abc123xyz"
    }
  }
}
```

**Response Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"sessions\": [...],\n  \"continuationToken\": \"...\"\n}"
      }
    ]
  }
}
```

**KSeF API Endpoint:** `GET /online/session/list`

---

### get_current_session

Get information about the current active authentication session.

**Method:** `tools/call`
**Tool Name:** `get_current_session`

**Parameters:** None

**Request Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_current_session",
    "arguments": {}
  }
}
```

**Response Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"sessionToken\": \"...\",\n  \"expiresAt\": \"...\"\n}"
      }
    ]
  }
}
```

**KSeF API Endpoint:** `GET /online/session/status`

---

### terminate_session

Terminate a specific authentication session by reference number.

**Method:** `tools/call`
**Tool Name:** `terminate_session`

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| referenceNumber | string | Yes | Session reference number |

**Request Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "terminate_session",
    "arguments": {
      "referenceNumber": "20251228-AU-1234567890-ABCDEF1234-56"
    }
  }
}
```

**Response Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"status\": \"terminated\"\n}"
      }
    ]
  }
}
```

**KSeF API Endpoint:** `DELETE /online/session/terminate/{referenceNumber}`

---

## Invoice Operations

Tools for querying, retrieving, and exporting invoices.

---

### get_invoice

Retrieve full invoice details by KSeF number.

**Method:** `tools/call`
**Tool Name:** `get_invoice`

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| ksefNumber | string | Yes | KSeF invoice identifier |

**Request Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_invoice",
    "arguments": {
      "ksefNumber": "1234567890123-20251228-0F5A3B7C9D-E1"
    }
  }
}
```

**Response Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"invoiceData\": \"...\",\n  \"metadata\": {...}\n}"
      }
    ]
  }
}
```

**KSeF Number Format:**
```
{NIP}-{YYYYMMDD}-{HASH}-{CHECKSUM}
Example: 1234567890123-20251228-0F5A3B7C9D-E1
```

**KSeF API Endpoint:** `GET /online/invoice/get/{ksefNumber}`

---

### query_invoice_metadata

Query invoice metadata with filtering and pagination.

**Method:** `tools/call`
**Tool Name:** `query_invoice_metadata`

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| queryType | string | Yes | Type of query (e.g., "incremental", "range") |
| pageSize | integer | No | Results per page (10-100) |
| continuationToken | string | No | Pagination token |

**Request Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "query_invoice_metadata",
    "arguments": {
      "queryType": "incremental",
      "pageSize": 50
    }
  }
}
```

**Query Types:**
- `incremental` - Get new invoices since last query
- `range` - Get invoices within date range

**KSeF API Endpoint:** `POST /online/query/invoice/metadata`

---

### create_invoice_export

Create an export of invoices based on specified criteria.

**Method:** `tools/call`
**Tool Name:** `create_invoice_export`

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| exportType | string | Yes | Type of export |
| parameters | object | No | Export-specific parameters |

**Request Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "create_invoice_export",
    "arguments": {
      "exportType": "full",
      "parameters": {
        "dateFrom": "2025-01-01",
        "dateTo": "2025-12-31"
      }
    }
  }
}
```

**Export Types:**
- `full` - Complete invoice export
- `incremental` - Export new invoices

**Response:**
Returns a reference number to check export status.

**KSeF API Endpoint:** `POST /online/export/invoice`

---

### get_export_status

Check the status of a previously created invoice export.

**Method:** `tools/call`
**Tool Name:** `get_export_status`

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| referenceNumber | string | Yes | Export reference number |

**Request Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_export_status",
    "arguments": {
      "referenceNumber": "20251228-EX-1234567890-ABCDEF1234-56"
    }
  }
}
```

**Export Status Values:**
- `pending` - Export is being processed
- `ready` - Export is ready for download
- `failed` - Export failed

**KSeF API Endpoint:** `GET /online/export/{referenceNumber}/status`

---

## Online Sessions

Tools for managing online invoice processing sessions.

---

### create_online_session

Create a new online session for submitting invoices.

**Method:** `tools/call`
**Tool Name:** `create_online_session`

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| sessionType | string | No | Type of session to create |

**Request Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "create_online_session",
    "arguments": {
      "sessionType": "standard"
    }
  }
}
```

**Response:**
Returns session reference number for submitting invoices.

**KSeF API Endpoint:** `POST /online/session/init`

---

### close_online_session

Close an active online session.

**Method:** `tools/call`
**Tool Name:** `close_online_session`

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| referenceNumber | string | Yes | Session reference number |

**Request Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "close_online_session",
    "arguments": {
      "referenceNumber": "20251228-SE-1234567890-ABCDEF1234-56"
    }
  }
}
```

**KSeF API Endpoint:** `POST /online/session/finish/{referenceNumber}`

---

### submit_invoice

Submit an invoice to an active online session.

**Method:** `tools/call`
**Tool Name:** `submit_invoice`

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| sessionReferenceNumber | string | Yes | Session reference number |
| invoiceData | string | Yes | Invoice XML data |

**Request Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "submit_invoice",
    "arguments": {
      "sessionReferenceNumber": "20251228-SE-1234567890-ABCDEF1234-56",
      "invoiceData": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Invoice>...</Invoice>"
    }
  }
}
```

**Invoice Data:**
- Must be valid XML according to KSeF schema
- Must be properly signed if required
- Encoding: UTF-8

**Response:**
Returns KSeF number assigned to the submitted invoice.

**KSeF API Endpoint:** `POST /online/invoice/send/{sessionReferenceNumber}`

---

## System Information

Tools for system information (no authentication required).

---

### get_public_key_certificates

Retrieve Ministry of Finance public key certificates for encryption and verification.

**Method:** `tools/call`
**Tool Name:** `get_public_key_certificates`

**Parameters:** None

**Request Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_public_key_certificates",
    "arguments": {}
  }
}
```

**Response Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"certificates\": [...]\n}"
      }
    ]
  }
}
```

**Use Cases:**
- Encryption of sensitive data
- Signature verification
- Certificate rotation monitoring

**KSeF API Endpoint:** `GET /common/certificates`

---

### get_rate_limits

Get current API rate limits and usage statistics.

**Method:** `tools/call`
**Tool Name:** `get_rate_limits`

**Parameters:** None

**Request Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_rate_limits",
    "arguments": {}
  }
}
```

**Response Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"limit\": 1000,\n  \"remaining\": 750,\n  \"reset\": \"2025-12-28T15:00:00Z\"\n}"
      }
    ]
  }
}
```

**Use Cases:**
- Monitor API quota
- Prevent rate limit errors
- Plan batch operations

**KSeF API Endpoint:** `GET /common/rate-limit/status`

---

## Error Handling

### Standard MCP Errors

All tools may return standard MCP errors:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32600,
    "message": "Invalid request"
  }
}
```

**MCP Error Codes:**
- `-32700` - Parse error
- `-32600` - Invalid request
- `-32601` - Method not found
- `-32602` - Invalid parameters
- `-32603` - Internal error

### KSeF API Errors

KSeF API errors are returned in the tool response:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Error: 401 Unauthorized - Invalid session token"
      }
    ],
    "isError": true
  }
}
```

**Common KSeF Errors:**
- `400` - Bad Request (invalid parameters)
- `401` - Unauthorized (invalid/missing token)
- `403` - Forbidden (insufficient permissions)
- `404` - Not Found (resource doesn't exist)
- `429` - Too Many Requests (rate limit exceeded)
- `500` - Internal Server Error
- `503` - Service Unavailable

## Rate Limits

KSeF API enforces rate limits per session:

**Default Limits:**
- Requests per minute: 60
- Requests per hour: 1000
- Concurrent connections: 10

**Best Practices:**
1. Check rate limits with `get_rate_limits` before batch operations
2. Implement exponential backoff on 429 errors
3. Cache frequently accessed data
4. Use pagination efficiently

**Rate Limit Headers:**
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 750
X-RateLimit-Reset: 1703779200
```

## Reference Number Formats

KSeF uses standardized reference number formats:

**Authentication Session:**
```
{YYYYMMDD}-AU-{10digits}-{10hex}-{2digits}
Example: 20251228-AU-1234567890-ABCDEF1234-56
```

**Online Session:**
```
{YYYYMMDD}-SE-{10digits}-{10hex}-{2digits}
Example: 20251228-SE-1234567890-ABCDEF1234-56
```

**Export:**
```
{YYYYMMDD}-EX-{10digits}-{10hex}-{2digits}
Example: 20251228-EX-1234567890-ABCDEF1234-56
```

**KSeF Number (Invoice):**
```
{NIP}-{YYYYMMDD}-{10hex}-{2hex}
Example: 1234567890123-20251228-0F5A3B7C9D-E1
```

## Version Information

- **MCP Protocol Version:** 2024-11-05
- **KSeF API Version:** v2
- **Server Version:** 0.1.0

## Additional Resources

- [Usage Guide](./USAGE.md)
- [Configuration Guide](./CONFIGURATION.md)
- [KSeF API Documentation](https://github.com/CIRFMF/ksef-docs)
- [MCP Specification](https://modelcontextprotocol.io)
