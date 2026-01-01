# MCP Protocol

Core protocol types for Model Context Protocol (MCP) servers.

## Overview

This library provides the fundamental types and utilities for implementing MCP servers using JSON-RPC 2.0 over stdio.

## Features

- JSON-RPC 2.0 request/response types
- MCP tool definitions
- Tool call results and content types
- Helper methods for creating responses

## Usage

```rust
use mcp_protocol::{JsonRpcRequest, JsonRpcResponse, ToolDefinition, ToolCallResult};
use serde_json::json;

// Create a success response
let response = JsonRpcResponse::success(
    Some(json!(1)),
    json!({"status": "ok"})
);

// Create an error response
let error = JsonRpcResponse::method_not_found(
    Some(json!(1)),
    "unknown_method"
);

// Define a tool
let tool = ToolDefinition::new(
    "my_tool",
    "Description of my tool",
    json!({
        "type": "object",
        "properties": {
            "param": {
                "type": "string",
                "description": "A parameter"
            }
        }
    })
);

// Create a tool result
let result = ToolCallResult::text("Tool executed successfully");
```

## Types

### JsonRpcRequest

Represents a JSON-RPC 2.0 request with method and optional parameters.

### JsonRpcResponse

Represents a JSON-RPC 2.0 response with either a result or error. Includes helper methods:
- `success()` - Create a successful response
- `error()` - Create an error response
- `method_not_found()` - Create a method not found error
- `invalid_params()` - Create an invalid params error
- `internal_error()` - Create an internal error

### ToolDefinition

Defines an MCP tool with name, description, and JSON schema for input validation.

### ToolCallResult

Represents the result of a tool call, containing content items.

### ToolContent

Enum representing different types of tool output content:
- `Text` - Plain text content

## License

This project is provided as-is for MCP server implementations.
