use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// JSON-RPC request from MCP client
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

/// JSON-RPC response to MCP client
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Tool definition for MCP
#[derive(Debug, Clone, Serialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Resource definition for MCP
#[derive(Debug, Clone, Serialize)]
pub struct ResourceDef {
    pub uri: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
}

/// Prompt definition for MCP
#[derive(Debug, Clone, Serialize)]
pub struct PromptDef {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// Tool handler function type
pub type ToolHandler = Box<dyn Fn(Value) -> Result<Value, String> + Send + Sync>;

/// Resource handler function type
pub type ResourceHandler = Box<dyn Fn() -> Result<String, String> + Send + Sync>;

/// Prompt handler function type
pub type PromptHandler =
    Box<dyn Fn(HashMap<String, String>) -> Result<Vec<PromptMessage>, String> + Send + Sync>;

#[derive(Debug, Clone, Serialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: PromptContent,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// The MCP Server
pub struct McpServer {
    name: String,
    version: String,
    description: String,
    tools: Vec<ToolDef>,
    tool_handlers: HashMap<String, ToolHandler>,
    resources: Vec<ResourceDef>,
    resource_handlers: HashMap<String, ResourceHandler>,
    prompts: Vec<PromptDef>,
    prompt_handlers: HashMap<String, PromptHandler>,
}

impl McpServer {
    pub fn new(name: String, version: String, description: String) -> Self {
        Self {
            name,
            version,
            description,
            tools: Vec::new(),
            tool_handlers: HashMap::new(),
            resources: Vec::new(),
            resource_handlers: HashMap::new(),
            prompts: Vec::new(),
            prompt_handlers: HashMap::new(),
        }
    }

    pub fn add_tool(&mut self, def: ToolDef, handler: ToolHandler) {
        let name = def.name.clone();
        self.tools.push(def);
        self.tool_handlers.insert(name, handler);
    }

    pub fn add_resource(&mut self, def: ResourceDef, handler: ResourceHandler) {
        let uri = def.uri.clone();
        self.resources.push(def);
        self.resource_handlers.insert(uri, handler);
    }

    pub fn add_prompt(&mut self, def: PromptDef, handler: PromptHandler) {
        let name = def.name.clone();
        self.prompts.push(def);
        self.prompt_handlers.insert(name, handler);
    }

    fn handle_initialize(&self, _params: Option<Value>) -> Value {
        serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": { "listChanged": false },
                "resources": { "subscribe": false, "listChanged": false },
                "prompts": { "listChanged": false }
            },
            "serverInfo": {
                "name": self.name,
                "version": self.version
            },
            "instructions": self.description
        })
    }

    fn handle_tools_list(&self) -> Value {
        serde_json::json!({ "tools": self.tools })
    }

    fn handle_tools_call(&self, params: Option<Value>) -> Value {
        let params = match params {
            Some(p) => p,
            None => return self.error_content("Missing params"),
        };

        let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        match self.tool_handlers.get(name) {
            Some(handler) => match handler(arguments) {
                Ok(result) => {
                    let text = match result {
                        Value::String(s) => s,
                        other => serde_json::to_string_pretty(&other).unwrap_or_default(),
                    };
                    serde_json::json!({
                        "content": [{ "type": "text", "text": text }]
                    })
                }
                Err(e) => serde_json::json!({
                    "content": [{ "type": "text", "text": e }],
                    "isError": true
                }),
            },
            None => serde_json::json!({
                "content": [{ "type": "text", "text": format!("Unknown tool: {}", name) }],
                "isError": true
            }),
        }
    }

    fn handle_resources_list(&self) -> Value {
        serde_json::json!({ "resources": self.resources })
    }

    fn handle_resources_read(&self, params: Option<Value>) -> Value {
        let params = match params {
            Some(p) => p,
            None => return self.error_content("Missing params"),
        };

        let uri = params.get("uri").and_then(|u| u.as_str()).unwrap_or("");

        if let Some(resource_def) = self.find_resource(uri) {
            let handler_key = &resource_def.uri;
            if let Some(handler) = self.resource_handlers.get(handler_key) {
                return match handler() {
                    Ok(text) => serde_json::json!({
                        "contents": [{
                            "uri": uri,
                            "mimeType": resource_def.mime_type,
                            "text": text
                        }]
                    }),
                    Err(e) => self.error_content(&e),
                };
            }
        }

        self.error_content(&format!("Unknown resource: {}", uri))
    }

    fn handle_prompts_list(&self) -> Value {
        serde_json::json!({ "prompts": self.prompts })
    }

    fn handle_prompts_get(&self, params: Option<Value>) -> Value {
        let params = match params {
            Some(p) => p,
            None => return self.error_content("Missing params"),
        };

        let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let arguments: HashMap<String, String> = params
            .get("arguments")
            .and_then(|a| serde_json::from_value(a.clone()).ok())
            .unwrap_or_default();

        match self.prompt_handlers.get(name) {
            Some(handler) => match handler(arguments) {
                Ok(messages) => serde_json::json!({ "messages": messages }),
                Err(e) => self.error_content(&e),
            },
            None => self.error_content(&format!("Unknown prompt: {}", name)),
        }
    }

    fn error_content(&self, msg: &str) -> Value {
        serde_json::json!({
            "content": [{ "type": "text", "text": msg }],
            "isError": true
        })
    }

    fn find_resource(&self, uri: &str) -> Option<&ResourceDef> {
        self.resources.iter().find(|resource| {
            resource.uri == uri
                || (resource.uri.contains('{')
                    && uri.starts_with(resource.uri.split('{').next().unwrap_or(&resource.uri)))
        })
    }

    fn handle_request(&self, req: &JsonRpcRequest) -> Option<JsonRpcResponse> {
        if req.jsonrpc != "2.0" {
            return req.id.clone().map(|id| JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: Some(id),
                result: None,
                error: Some(JsonRpcError {
                    code: -32600,
                    message: format!("Unsupported jsonrpc version: {}", req.jsonrpc),
                    data: None,
                }),
            });
        }

        let result = match req.method.as_str() {
            "initialize" => Some(self.handle_initialize(req.params.clone())),
            "initialized" | "notifications/initialized" => None,
            "tools/list" => Some(self.handle_tools_list()),
            "tools/call" => Some(self.handle_tools_call(req.params.clone())),
            "resources/list" => Some(self.handle_resources_list()),
            "resources/read" => Some(self.handle_resources_read(req.params.clone())),
            "prompts/list" => Some(self.handle_prompts_list()),
            "prompts/get" => Some(self.handle_prompts_get(req.params.clone())),
            "ping" => Some(serde_json::json!({})),
            _ => {
                return req.id.clone().map(|id| JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Some(id),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: format!("Method not found: {}", req.method),
                        data: None,
                    }),
                });
            }
        };

        let id = req.id.clone()?;
        result.map(|value| JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            result: Some(value),
            error: None,
        })
    }

    pub async fn run_stdio(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }

            let req: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    let err_response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: None,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: format!("Parse error: {}", e),
                            data: None,
                        }),
                    };
                    let response_str = serde_json::to_string(&err_response)?;
                    stdout.write_all(response_str.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                    continue;
                }
            };

            if let Some(response) = self.handle_request(&req) {
                let response_str = serde_json::to_string(&response)?;
                stdout.write_all(response_str.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }

        Ok(())
    }
}
