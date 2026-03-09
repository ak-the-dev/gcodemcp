use serde_json::{json, Value};
use std::error::Error;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::timeout;

const RESPONSE_TIMEOUT: Duration = Duration::from_secs(2);
const NO_RESPONSE_TIMEOUT: Duration = Duration::from_millis(150);

struct TestServer {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: Lines<BufReader<ChildStdout>>,
}

impl TestServer {
    async fn spawn() -> Result<Self, Box<dyn Error>> {
        let mut child = Command::new(env!("CARGO_BIN_EXE_gcode-mcp"))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        let stdin = child.stdin.take().ok_or("missing child stdin")?;
        let stdout = child.stdout.take().ok_or("missing child stdout")?;

        Ok(Self {
            child,
            stdin: Some(stdin),
            stdout: BufReader::new(stdout).lines(),
        })
    }

    async fn send_request(&mut self, request: Value) -> Result<Value, Box<dyn Error>> {
        self.send_value(&request).await?;
        self.read_response().await
    }

    async fn send_notification(&mut self, notification: Value) -> Result<(), Box<dyn Error>> {
        self.send_value(&notification).await
    }

    async fn send_raw(&mut self, line: &str) -> Result<(), Box<dyn Error>> {
        let stdin = self.stdin.as_mut().ok_or("server stdin already closed")?;
        stdin.write_all(line.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        Ok(())
    }

    async fn read_response(&mut self) -> Result<Value, Box<dyn Error>> {
        let next_line = timeout(RESPONSE_TIMEOUT, self.stdout.next_line()).await?;
        let response_line = next_line?.ok_or("server stdout closed unexpectedly")?;
        Ok(serde_json::from_str(&response_line)?)
    }

    async fn assert_no_response(&mut self) -> Result<(), Box<dyn Error>> {
        match timeout(NO_RESPONSE_TIMEOUT, self.stdout.next_line()).await {
            Err(_) => Ok(()),
            Ok(Ok(Some(line))) => {
                Err(format!("unexpected response for notification: {line}").into())
            }
            Ok(Ok(None)) => Err("server stdout closed unexpectedly".into()),
            Ok(Err(error)) => Err(error.into()),
        }
    }

    async fn initialize(&mut self) -> Result<Value, Box<dyn Error>> {
        self.send_request(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .await
    }

    async fn shutdown(mut self) -> Result<(), Box<dyn Error>> {
        self.stdin.take();

        if timeout(Duration::from_secs(1), self.child.wait())
            .await
            .is_err()
        {
            self.child.kill().await?;
            self.child.wait().await?;
        }

        Ok(())
    }

    async fn send_value(&mut self, value: &Value) -> Result<(), Box<dyn Error>> {
        let serialized = serde_json::to_string(value)?;
        self.send_raw(&serialized).await
    }
}

fn response_id(response: &Value) -> &Value {
    response.get("id").expect("response missing id")
}

fn response_result(response: &Value) -> &Value {
    response.get("result").expect("response missing result")
}

fn response_error(response: &Value) -> &Value {
    response.get("error").expect("response missing error")
}

fn response_text_content(response: &Value) -> &str {
    response_result(response)["content"][0]["text"]
        .as_str()
        .expect("response text content missing")
}

#[tokio::test(flavor = "current_thread")]
async fn initialize_returns_protocol_metadata_and_instructions() -> Result<(), Box<dyn Error>> {
    let mut server = TestServer::spawn().await?;

    let response = server.initialize().await?;

    assert_eq!(response_id(&response), &json!(1));
    assert_eq!(
        response_result(&response)["protocolVersion"],
        json!("2024-11-05")
    );
    assert_eq!(
        response_result(&response)["serverInfo"]["name"],
        json!("gcode-mcp")
    );
    assert_eq!(
        response_result(&response)["instructions"],
        json!("Analyze, validate, generate, and post-process 3D printer G-code over MCP")
    );
    assert!(response_result(&response)["capabilities"]
        .as_object()
        .is_some_and(|capabilities| !capabilities.is_empty()));

    server.shutdown().await?;
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn initialized_notification_produces_no_response() -> Result<(), Box<dyn Error>> {
    let mut server = TestServer::spawn().await?;
    server.initialize().await?;

    server
        .send_notification(json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }))
        .await?;
    server.assert_no_response().await?;

    server.shutdown().await?;
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn list_methods_return_registered_tools_resources_and_prompts() -> Result<(), Box<dyn Error>>
{
    let mut server = TestServer::spawn().await?;
    server.initialize().await?;

    let tools = server
        .send_request(json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }))
        .await?;
    let resources = server
        .send_request(json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "resources/list"
        }))
        .await?;
    let prompts = server
        .send_request(json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "prompts/list"
        }))
        .await?;

    let tool_names: Vec<_> = response_result(&tools)["tools"]
        .as_array()
        .expect("tools list missing")
        .iter()
        .filter_map(|tool| tool.get("name").and_then(Value::as_str))
        .collect();
    assert!(tool_names.contains(&"lookup_printer"));
    assert!(tool_names.contains(&"analyze_gcode"));
    assert!(tool_names.contains(&"generate_start_gcode"));

    let resource_items = response_result(&resources)["resources"]
        .as_array()
        .expect("resources list missing");
    assert_eq!(resource_items.len(), 4);
    assert!(resource_items.iter().any(|resource| {
        resource["uri"] == json!("gcode://materials")
            && resource["mimeType"] == json!("application/json")
    }));

    let prompt_items = response_result(&prompts)["prompts"]
        .as_array()
        .expect("prompts list missing");
    assert_eq!(prompt_items.len(), 5);
    assert!(prompt_items.iter().any(|prompt| {
        prompt["name"] == json!("create_gcode") && prompt["arguments"].is_array()
    }));

    server.shutdown().await?;
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn tools_call_smoke_returns_expected_payloads() -> Result<(), Box<dyn Error>> {
    let mut server = TestServer::spawn().await?;
    server.initialize().await?;

    let lookup_printer = server
        .send_request(json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "lookup_printer",
                "arguments": {
                    "printer_id": "ender3"
                }
            }
        }))
        .await?;
    let printer_payload: Value = serde_json::from_str(response_text_content(&lookup_printer))?;
    assert_eq!(printer_payload["id"], json!("ender3"));
    assert_eq!(printer_payload["name"], json!("Creality Ender 3"));

    let calculate_extrusion = server
        .send_request(json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "calculate_extrusion",
                "arguments": {
                    "distance_mm": 20.0,
                    "nozzle_diameter": 0.4,
                    "layer_height": 0.2
                }
            }
        }))
        .await?;
    let extrusion_payload: Value =
        serde_json::from_str(response_text_content(&calculate_extrusion))?;
    assert_eq!(extrusion_payload["distance_mm"], json!(20.0));
    assert_eq!(extrusion_payload["extrusion_width_mm"], json!(0.48));
    assert!(extrusion_payload["e_value"]
        .as_f64()
        .is_some_and(|value| value > 0.0));

    server.shutdown().await?;
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn resources_read_and_prompts_get_return_expected_content() -> Result<(), Box<dyn Error>> {
    let mut server = TestServer::spawn().await?;
    server.initialize().await?;

    let materials = server
        .send_request(json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "resources/read",
            "params": {
                "uri": "gcode://materials"
            }
        }))
        .await?;
    let material_contents = response_result(&materials)["contents"]
        .as_array()
        .expect("resource contents missing");
    assert_eq!(material_contents.len(), 1);
    assert_eq!(material_contents[0]["mimeType"], json!("application/json"));
    let material_payload: Value = serde_json::from_str(
        material_contents[0]["text"]
            .as_str()
            .expect("resource text missing"),
    )?;
    assert!(material_payload
        .as_array()
        .is_some_and(|items| !items.is_empty()));

    let prompt = server
        .send_request(json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": "prompts/get",
            "params": {
                "name": "create_gcode",
                "arguments": {
                    "description": "a calibration cube"
                }
            }
        }))
        .await?;
    let messages = response_result(&prompt)["messages"]
        .as_array()
        .expect("prompt messages missing");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["role"], json!("user"));
    assert!(messages[0]["content"]["text"]
        .as_str()
        .is_some_and(|text| text.contains("a calibration cube")));

    server.shutdown().await?;
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn malformed_json_returns_parse_error() -> Result<(), Box<dyn Error>> {
    let mut server = TestServer::spawn().await?;

    server.send_raw("{\"jsonrpc\":\"2.0\"").await?;
    let response = server.read_response().await?;

    assert_eq!(response_error(&response)["code"], json!(-32700));
    assert!(response.get("id").is_none() || response["id"].is_null());

    server.shutdown().await?;
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn unknown_method_returns_json_rpc_error() -> Result<(), Box<dyn Error>> {
    let mut server = TestServer::spawn().await?;
    server.initialize().await?;

    let response = server
        .send_request(json!({
            "jsonrpc": "2.0",
            "id": 9,
            "method": "does/not/exist"
        }))
        .await?;

    assert_eq!(response_id(&response), &json!(9));
    assert_eq!(response_error(&response)["code"], json!(-32601));
    assert_eq!(
        response_error(&response)["message"],
        json!("Method not found: does/not/exist")
    );

    server.shutdown().await?;
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn unknown_tool_returns_mcp_tool_error_payload() -> Result<(), Box<dyn Error>> {
    let mut server = TestServer::spawn().await?;
    server.initialize().await?;

    let response = server
        .send_request(json!({
            "jsonrpc": "2.0",
            "id": 10,
            "method": "tools/call",
            "params": {
                "name": "not_a_real_tool",
                "arguments": {}
            }
        }))
        .await?;

    assert_eq!(response_id(&response), &json!(10));
    assert_eq!(response_result(&response)["isError"], json!(true));
    assert!(
        response_text_content(&response).contains("Unknown tool: not_a_real_tool"),
        "unexpected tool error payload: {response}"
    );

    server.shutdown().await?;
    Ok(())
}
