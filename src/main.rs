use anyhow::{anyhow, Result};
use ksef_client::KsefClient;
use mcp_protocol::{JsonRpcRequest, JsonRpcResponse, ToolCallResult, ToolDefinition};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

struct McpServer {
    ksef_client: KsefClient,
}

impl McpServer {
    fn new() -> Self {
        Self {
            ksef_client: KsefClient::new(),
        }
    }

    async fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone();

        match request.method.as_str() {
            "initialize" => self.handle_initialize(id),
            "tools/list" => self.handle_list_tools(id),
            "tools/call" => self.handle_tool_call(id, request.params).await,
            _ => JsonRpcResponse::method_not_found(id, &request.method),
        }
    }

    fn handle_initialize(&self, id: Option<Value>) -> JsonRpcResponse {
        JsonRpcResponse::success(
            id,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "ksef-mcp-server",
                    "version": "0.1.0"
                }
            }),
        )
    }

    fn handle_list_tools(&self, id: Option<Value>) -> JsonRpcResponse {
        let tools = vec![
            ToolDefinition::new(
                "get_active_sessions",
                "Get list of active authentication sessions",
                json!({
                    "type": "object",
                    "properties": {
                        "pageSize": {
                            "type": "integer",
                            "description": "Number of results per page (10-100)",
                            "minimum": 10,
                            "maximum": 100,
                            "default": 10
                        },
                        "continuationToken": {
                            "type": "string",
                            "description": "Token for getting next page of results"
                        }
                    }
                }),
            ),
            ToolDefinition::new(
                "get_current_session",
                "Get information about the current active session",
                json!({"type": "object", "properties": {}}),
            ),
            ToolDefinition::new(
                "terminate_session",
                "Terminate a specific authentication session",
                json!({
                    "type": "object",
                    "properties": {
                        "referenceNumber": {
                            "type": "string",
                            "description": "Reference number of the session to terminate"
                        }
                    },
                    "required": ["referenceNumber"]
                }),
            ),
            ToolDefinition::new(
                "get_invoice",
                "Get invoice details by KSeF number",
                json!({
                    "type": "object",
                    "properties": {
                        "ksefNumber": {
                            "type": "string",
                            "description": "KSeF invoice number"
                        }
                    },
                    "required": ["ksefNumber"]
                }),
            ),
            ToolDefinition::new(
                "query_invoice_metadata",
                "Query invoice metadata with filtering and pagination",
                json!({
                    "type": "object",
                    "properties": {
                        "queryType": {
                            "type": "string",
                            "description": "Type of query (e.g., 'incremental', 'range')"
                        },
                        "pageSize": {
                            "type": "integer",
                            "description": "Number of results per page",
                            "minimum": 10,
                            "maximum": 100,
                            "default": 10
                        },
                        "continuationToken": {
                            "type": "string",
                            "description": "Token for getting next page of results"
                        }
                    }
                }),
            ),
            ToolDefinition::new(
                "create_invoice_export",
                "Create an export of invoices",
                json!({
                    "type": "object",
                    "properties": {
                        "exportType": {
                            "type": "string",
                            "description": "Type of export to create"
                        },
                        "parameters": {
                            "type": "object",
                            "description": "Export parameters"
                        }
                    },
                    "required": ["exportType"]
                }),
            ),
            ToolDefinition::new(
                "get_export_status",
                "Get status of an invoice export",
                json!({
                    "type": "object",
                    "properties": {
                        "referenceNumber": {
                            "type": "string",
                            "description": "Reference number of the export"
                        }
                    },
                    "required": ["referenceNumber"]
                }),
            ),
            ToolDefinition::new(
                "get_public_key_certificates",
                "Get Ministry of Finance public key certificates",
                json!({"type": "object", "properties": {}}),
            ),
            ToolDefinition::new(
                "get_rate_limits",
                "Get current API rate limits status",
                json!({"type": "object", "properties": {}}),
            ),
            ToolDefinition::new(
                "create_online_session",
                "Create a new online session for invoice processing",
                json!({
                    "type": "object",
                    "properties": {
                        "sessionType": {
                            "type": "string",
                            "description": "Type of session to create"
                        }
                    }
                }),
            ),
            ToolDefinition::new(
                "close_online_session",
                "Close an online session",
                json!({
                    "type": "object",
                    "properties": {
                        "referenceNumber": {
                            "type": "string",
                            "description": "Reference number of the session to close"
                        }
                    },
                    "required": ["referenceNumber"]
                }),
            ),
            ToolDefinition::new(
                "submit_invoice",
                "Submit an invoice to a session",
                json!({
                    "type": "object",
                    "properties": {
                        "sessionReferenceNumber": {
                            "type": "string",
                            "description": "Reference number of the session"
                        },
                        "invoiceData": {
                            "type": "string",
                            "description": "Invoice data in XML format"
                        }
                    },
                    "required": ["sessionReferenceNumber", "invoiceData"]
                }),
            ),
        ];

        JsonRpcResponse::success(id, json!({ "tools": tools }))
    }

    async fn handle_tool_call(&mut self, id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => return JsonRpcResponse::invalid_params(id, "Invalid params"),
        };

        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => return JsonRpcResponse::invalid_params(id, "Missing tool name"),
        };

        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        let result = self.execute_tool(tool_name, &arguments).await;

        match result {
            Ok(content) => JsonRpcResponse::success(id, json!(ToolCallResult::text(content))),
            Err(e) => JsonRpcResponse::internal_error(id, format!("Tool execution failed: {}", e)),
        }
    }

    async fn execute_tool(&mut self, tool_name: &str, args: &Value) -> Result<String> {
        match tool_name {
            "get_active_sessions" => {
                let page_size = args.get("pageSize").and_then(|v| v.as_i64()).unwrap_or(10);
                let continuation_token = args.get("continuationToken").and_then(|v| v.as_str());

                let result = self.ksef_client.get_active_sessions(page_size, continuation_token).await?;
                Ok(format!("Active sessions:\n{}", result))
            }
            "get_current_session" => {
                let result = self.ksef_client.get_current_session().await?;
                Ok(format!("Current session:\n{}", result))
            }
            "terminate_session" => {
                let reference_number = args
                    .get("referenceNumber")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing referenceNumber"))?;

                let result = self.ksef_client.terminate_session(reference_number).await?;
                Ok(format!("Session terminated:\n{}", result))
            }
            "get_invoice" => {
                let ksef_number = args
                    .get("ksefNumber")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing ksefNumber"))?;

                let result = self.ksef_client.get_invoice(ksef_number).await?;
                Ok(format!("Invoice details:\n{}", result))
            }
            "query_invoice_metadata" => {
                let result = self.ksef_client.query_invoice_metadata(args).await?;
                Ok(format!("Invoice metadata:\n{}", result))
            }
            "create_invoice_export" => {
                let result = self.ksef_client.create_invoice_export(args).await?;
                Ok(format!("Export created:\n{}", result))
            }
            "get_export_status" => {
                let reference_number = args
                    .get("referenceNumber")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing referenceNumber"))?;

                let result = self.ksef_client.get_export_status(reference_number).await?;
                Ok(format!("Export status:\n{}", result))
            }
            "get_public_key_certificates" => {
                let result = self.ksef_client.get_public_key_certificates().await?;
                Ok(format!("Public key certificates:\n{}", result))
            }
            "get_rate_limits" => {
                let result = self.ksef_client.get_rate_limits().await?;
                Ok(format!("Rate limits:\n{}", result))
            }
            "create_online_session" => {
                let result = self.ksef_client.create_online_session(args).await?;
                Ok(format!("Online session created:\n{}", result))
            }
            "close_online_session" => {
                let reference_number = args
                    .get("referenceNumber")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing referenceNumber"))?;

                let result = self.ksef_client.close_online_session(reference_number).await?;
                Ok(format!("Session closed:\n{}", result))
            }
            "submit_invoice" => {
                let session_ref = args
                    .get("sessionReferenceNumber")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing sessionReferenceNumber"))?;

                let invoice_data = args
                    .get("invoiceData")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing invoiceData"))?;

                let result = self.ksef_client.submit_invoice(session_ref, invoice_data).await?;
                Ok(format!("Invoice submitted:\n{}", result))
            }
            _ => Err(anyhow!("Unknown tool: {}", tool_name)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut server = McpServer::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Failed to parse request: {}", e);
                continue;
            }
        };

        let response = server.handle_request(request).await;

        let response_json = serde_json::to_string(&response)?;
        writeln!(stdout, "{}", response_json)?;
        stdout.flush()?;
    }

    Ok(())
}
