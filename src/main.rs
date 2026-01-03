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

    async fn handle_request(&mut self, request: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = request.id.clone();

        // Handle notifications (no id = no response)
        if id.is_none() {
            match request.method.as_str() {
                "notifications/initialized" => {
                    eprintln!("Client initialized");
                    return None;
                }
                _ => {
                    eprintln!("Unknown notification: {}", request.method);
                    return None;
                }
            }
        }

        // Handle requests (with id = send response)
        Some(match request.method.as_str() {
            "initialize" => self.handle_initialize(id),
            "tools/list" => self.handle_list_tools(id),
            "tools/call" => self.handle_tool_call(id, request.params).await,
            _ => JsonRpcResponse::method_not_found(id, &request.method),
        })
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
                        "subjectType": {
                            "type": "string",
                            "description": "Subject type: Subject1 (seller), Subject2 (buyer), Subject3, SubjectAuthorized",
                            "enum": ["Subject1", "Subject2", "Subject3", "SubjectAuthorized"]
                        },
                        "dateRange": {
                            "type": "object",
                            "description": "Date range filter (max 3 months)",
                            "properties": {
                                "dateType": {
                                    "type": "string",
                                    "description": "Date type to filter by"
                                },
                                "from": {
                                    "type": "string",
                                    "description": "Start date (ISO 8601 format)"
                                },
                                "to": {
                                    "type": "string",
                                    "description": "End date (ISO 8601 format)"
                                }
                            },
                            "required": ["dateType", "from"]
                        },
                        "ksefNumber": {
                            "type": "string",
                            "description": "KSeF invoice number (exact match)"
                        },
                        "invoiceNumber": {
                            "type": "string",
                            "description": "Invoice number from issuer (exact match)"
                        },
                        "sellerNip": {
                            "type": "string",
                            "description": "Seller NIP (exact match)"
                        },
                        "pageSize": {
                            "type": "integer",
                            "description": "Number of results per page",
                            "minimum": 10,
                            "maximum": 100,
                            "default": 10
                        }
                    },
                    "required": ["subjectType", "dateRange"]
                }),
            ),
            ToolDefinition::new(
                "create_invoice_export",
                "Create an encrypted export of invoices",
                json!({
                    "type": "object",
                    "properties": {
                        "encryption": {
                            "type": "object",
                            "description": "Encryption info for export result",
                            "properties": {
                                "encryptedSymmetricKey": {
                                    "type": "string",
                                    "description": "Base64-encoded encrypted symmetric key"
                                },
                                "initializationVector": {
                                    "type": "string",
                                    "description": "Base64-encoded initialization vector"
                                }
                            },
                            "required": ["encryptedSymmetricKey", "initializationVector"]
                        },
                        "filters": {
                            "type": "object",
                            "description": "Invoice query filters",
                            "properties": {
                                "subjectType": {
                                    "type": "string",
                                    "description": "Subject type",
                                    "enum": ["Subject1", "Subject2", "Subject3", "SubjectAuthorized"]
                                },
                                "dateRange": {
                                    "type": "object",
                                    "properties": {
                                        "dateType": {
                                            "type": "string",
                                            "description": "Date type"
                                        },
                                        "from": {
                                            "type": "string",
                                            "description": "Start date"
                                        },
                                        "to": {
                                            "type": "string",
                                            "description": "End date"
                                        }
                                    },
                                    "required": ["dateType", "from"]
                                }
                            },
                            "required": ["subjectType", "dateRange"]
                        }
                    },
                    "required": ["encryption", "filters"]
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
                        "formCode": {
                            "type": "object",
                            "description": "Invoice schema for this session",
                            "properties": {
                                "systemCode": {
                                    "type": "string",
                                    "description": "System code (e.g., 'FA (2)', 'FA (3)', 'PEF (3)')"
                                },
                                "schemaVersion": {
                                    "type": "string",
                                    "description": "Schema version (e.g., '1-0E', '2-1')"
                                },
                                "value": {
                                    "type": "string",
                                    "description": "Form value (e.g., 'FA', 'PEF')"
                                }
                            },
                            "required": ["systemCode", "schemaVersion", "value"]
                        },
                        "encryption": {
                            "type": "object",
                            "description": "Symmetric encryption key info encrypted with MF public key",
                            "properties": {
                                "encryptedSymmetricKey": {
                                    "type": "string",
                                    "description": "Base64-encoded encrypted symmetric key"
                                },
                                "initializationVector": {
                                    "type": "string",
                                    "description": "Base64-encoded initialization vector"
                                }
                            },
                            "required": ["encryptedSymmetricKey", "initializationVector"]
                        }
                    },
                    "required": ["formCode", "encryption"]
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
                "Submit an encrypted invoice to a session",
                json!({
                    "type": "object",
                    "properties": {
                        "sessionReferenceNumber": {
                            "type": "string",
                            "description": "Reference number of the session"
                        },
                        "invoiceHash": {
                            "type": "string",
                            "description": "Base64-encoded SHA256 hash of original invoice"
                        },
                        "invoiceSize": {
                            "type": "integer",
                            "description": "Size of original invoice in bytes"
                        },
                        "encryptedInvoiceHash": {
                            "type": "string",
                            "description": "Base64-encoded SHA256 hash of encrypted invoice"
                        },
                        "encryptedInvoiceSize": {
                            "type": "integer",
                            "description": "Size of encrypted invoice in bytes"
                        },
                        "encryptedInvoiceContent": {
                            "type": "string",
                            "description": "Base64-encoded encrypted invoice (AES-256-CBC with PKCS#7)"
                        },
                        "offlineMode": {
                            "type": "boolean",
                            "description": "Offline invoicing mode",
                            "default": false
                        },
                        "hashOfCorrectedInvoice": {
                            "type": "string",
                            "description": "Base64-encoded SHA256 hash of corrected invoice (for technical corrections)"
                        }
                    },
                    "required": ["sessionReferenceNumber", "invoiceHash", "invoiceSize", "encryptedInvoiceHash", "encryptedInvoiceSize", "encryptedInvoiceContent"]
                }),
            ),
            ToolDefinition::new(
                "authenticate",
                "Authenticate with KSeF API using NIP and KSeF token (public key is fetched automatically)",
                json!({
                    "type": "object",
                    "properties": {
                        "nip": {
                            "type": "string",
                            "description": "Polish tax identification number (NIP) - 10 digits",
                            "pattern": "^[0-9]{10}$"
                        },
                        "ksefToken": {
                            "type": "string",
                            "description": "KSeF authorization token generated from KSeF portal"
                        }
                    },
                    "required": ["nip", "ksefToken"]
                }),
            ),
            ToolDefinition::new(
                "get_authentication_status",
                "Get current authentication status",
                json!({"type": "object", "properties": {}}),
            ),
            ToolDefinition::new(
                "logout",
                "Clear authentication session",
                json!({"type": "object", "properties": {}}),
            ),
            ToolDefinition::new(
                "refresh_token",
                "Refresh the access token using refresh token",
                json!({"type": "object", "properties": {}}),
            ),
            ToolDefinition::new(
                "get_sessions",
                "Get list of all sessions (both online and batch)",
                json!({
                    "type": "object",
                    "properties": {
                        "pageSize": {
                            "type": "integer",
                            "description": "Number of results per page (10-1000)",
                            "minimum": 10,
                            "maximum": 1000,
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
                "get_session_status",
                "Get status and details of a specific session",
                json!({
                    "type": "object",
                    "properties": {
                        "referenceNumber": {
                            "type": "string",
                            "description": "Reference number of the session"
                        }
                    },
                    "required": ["referenceNumber"]
                }),
            ),
            ToolDefinition::new(
                "get_session_invoices",
                "Get list of invoices in a session with their statuses",
                json!({
                    "type": "object",
                    "properties": {
                        "referenceNumber": {
                            "type": "string",
                            "description": "Reference number of the session"
                        },
                        "continuationToken": {
                            "type": "string",
                            "description": "Token for getting next page of results"
                        }
                    },
                    "required": ["referenceNumber"]
                }),
            ),
            ToolDefinition::new(
                "get_invoice_upo_by_ksef",
                "Get UPO (confirmation) for an invoice by its KSeF number",
                json!({
                    "type": "object",
                    "properties": {
                        "sessionReferenceNumber": {
                            "type": "string",
                            "description": "Reference number of the session"
                        },
                        "ksefNumber": {
                            "type": "string",
                            "description": "KSeF number of the invoice"
                        }
                    },
                    "required": ["sessionReferenceNumber", "ksefNumber"]
                }),
            ),
            ToolDefinition::new(
                "get_invoice_upo_by_reference",
                "Get UPO (confirmation) for an invoice by its reference number",
                json!({
                    "type": "object",
                    "properties": {
                        "sessionReferenceNumber": {
                            "type": "string",
                            "description": "Reference number of the session"
                        },
                        "invoiceReferenceNumber": {
                            "type": "string",
                            "description": "Reference number of the invoice"
                        }
                    },
                    "required": ["sessionReferenceNumber", "invoiceReferenceNumber"]
                }),
            ),
            ToolDefinition::new(
                "get_session_upo",
                "Get collective UPO for a session",
                json!({
                    "type": "object",
                    "properties": {
                        "sessionReferenceNumber": {
                            "type": "string",
                            "description": "Reference number of the session"
                        },
                        "upoReferenceNumber": {
                            "type": "string",
                            "description": "Reference number of the UPO"
                        }
                    },
                    "required": ["sessionReferenceNumber", "upoReferenceNumber"]
                }),
            ),
            ToolDefinition::new(
                "create_batch_session",
                "Create a new batch session for bulk invoice processing",
                json!({
                    "type": "object",
                    "properties": {
                        "formCode": {
                            "type": "object",
                            "description": "Invoice schema for this batch",
                            "properties": {
                                "systemCode": {
                                    "type": "string",
                                    "description": "System code (e.g., 'FA (2)', 'FA (3)')"
                                },
                                "schemaVersion": {
                                    "type": "string",
                                    "description": "Schema version (e.g., '1-0E')"
                                },
                                "value": {
                                    "type": "string",
                                    "description": "Form value (e.g., 'FA')"
                                }
                            },
                            "required": ["systemCode", "schemaVersion", "value"]
                        },
                        "batchFile": {
                            "type": "object",
                            "description": "Batch file information (max 5GB, max 50 parts)",
                            "properties": {
                                "fileSize": {
                                    "type": "integer",
                                    "description": "Total file size in bytes"
                                },
                                "fileHash": {
                                    "type": "string",
                                    "description": "Base64-encoded SHA256 hash of entire file"
                                },
                                "fileParts": {
                                    "type": "array",
                                    "description": "File parts (max 100MB per part before encryption)",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "ordinalNumber": {
                                                "type": "integer",
                                                "description": "Sequential part number"
                                            },
                                            "fileSize": {
                                                "type": "integer",
                                                "description": "Encrypted part size in bytes"
                                            },
                                            "fileHash": {
                                                "type": "string",
                                                "description": "Base64 SHA256 hash of encrypted part"
                                            }
                                        },
                                        "required": ["ordinalNumber", "fileSize", "fileHash"]
                                    }
                                }
                            },
                            "required": ["fileSize", "fileHash", "fileParts"]
                        },
                        "encryption": {
                            "type": "object",
                            "description": "Symmetric encryption key encrypted with MF public key",
                            "properties": {
                                "encryptedSymmetricKey": {
                                    "type": "string",
                                    "description": "Base64-encoded encrypted symmetric key"
                                },
                                "initializationVector": {
                                    "type": "string",
                                    "description": "Base64-encoded initialization vector"
                                }
                            },
                            "required": ["encryptedSymmetricKey", "initializationVector"]
                        },
                        "offlineMode": {
                            "type": "boolean",
                            "description": "Offline invoicing mode",
                            "default": false
                        }
                    },
                    "required": ["formCode", "batchFile", "encryption"]
                }),
            ),
            ToolDefinition::new(
                "close_batch_session",
                "Close a batch session and start processing",
                json!({
                    "type": "object",
                    "properties": {
                        "referenceNumber": {
                            "type": "string",
                            "description": "Reference number of the batch session to close"
                        }
                    },
                    "required": ["referenceNumber"]
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

                // Extract invoice data (remove sessionReferenceNumber from args)
                let mut invoice_data = args.clone();
                if let Some(obj) = invoice_data.as_object_mut() {
                    obj.remove("sessionReferenceNumber");
                }

                let result = self.ksef_client.submit_invoice(session_ref, &invoice_data).await?;
                Ok(format!("Invoice submitted:\n{}", result))
            }
            "authenticate" => {
                let nip = args
                    .get("nip")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing nip"))?;

                let ksef_token = args
                    .get("ksefToken")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing ksefToken"))?;

                let result = self.ksef_client.authenticate(nip, ksef_token).await?;
                Ok(result)
            }
            "get_authentication_status" => {
                let result = self.ksef_client.get_auth_status()?;
                Ok(result)
            }
            "logout" => {
                let result = self.ksef_client.logout()?;
                Ok(result)
            }
            "refresh_token" => {
                let result = self.ksef_client.refresh_access_token().await?;
                Ok(result)
            }
            "get_sessions" => {
                let page_size = args.get("pageSize").and_then(|v| v.as_i64()).unwrap_or(10);
                let continuation_token = args.get("continuationToken").and_then(|v| v.as_str());

                let result = self.ksef_client.get_sessions(page_size, continuation_token).await?;
                Ok(format!("Sessions list:\n{}", result))
            }
            "get_session_status" => {
                let reference_number = args
                    .get("referenceNumber")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing referenceNumber"))?;

                let result = self.ksef_client.get_session_status(reference_number).await?;
                Ok(format!("Session status:\n{}", result))
            }
            "get_session_invoices" => {
                let reference_number = args
                    .get("referenceNumber")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing referenceNumber"))?;
                let continuation_token = args.get("continuationToken").and_then(|v| v.as_str());

                let result = self.ksef_client.get_session_invoices(reference_number, continuation_token).await?;
                Ok(format!("Session invoices:\n{}", result))
            }
            "get_invoice_upo_by_ksef" => {
                let session_ref = args
                    .get("sessionReferenceNumber")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing sessionReferenceNumber"))?;
                let ksef_number = args
                    .get("ksefNumber")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing ksefNumber"))?;

                let result = self.ksef_client.get_invoice_upo_by_ksef(session_ref, ksef_number).await?;
                Ok(format!("Invoice UPO:\n{}", result))
            }
            "get_invoice_upo_by_reference" => {
                let session_ref = args
                    .get("sessionReferenceNumber")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing sessionReferenceNumber"))?;
                let invoice_ref = args
                    .get("invoiceReferenceNumber")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing invoiceReferenceNumber"))?;

                let result = self.ksef_client.get_invoice_upo_by_reference(session_ref, invoice_ref).await?;
                Ok(format!("Invoice UPO:\n{}", result))
            }
            "get_session_upo" => {
                let session_ref = args
                    .get("sessionReferenceNumber")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing sessionReferenceNumber"))?;
                let upo_ref = args
                    .get("upoReferenceNumber")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing upoReferenceNumber"))?;

                let result = self.ksef_client.get_session_upo(session_ref, upo_ref).await?;
                Ok(format!("Session UPO:\n{}", result))
            }
            "create_batch_session" => {
                let result = self.ksef_client.create_batch_session(args).await?;
                Ok(format!("Batch session created:\n{}", result))
            }
            "close_batch_session" => {
                let reference_number = args
                    .get("referenceNumber")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing referenceNumber"))?;

                let result = self.ksef_client.close_batch_session(reference_number).await?;
                Ok(format!("Batch session closed:\n{}", result))
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

        // Only send response if it's not a notification
        if let Some(resp) = response {
            let response_json = serde_json::to_string(&resp)?;
            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;
        }
    }

    Ok(())
}
