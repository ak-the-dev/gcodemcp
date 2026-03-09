mod data;
mod gcode;
mod mcp;
mod prompts;
mod resources;
mod tools;

use mcp::server::McpServer;

#[tokio::main]
async fn main() {
    let mut server = McpServer::new(
        "gcode-mcp".to_string(),
        "1.0.0".to_string(),
        "MCP server for 3D printer G-code creation, analysis, and optimization".to_string(),
    );

    tools::register_all(&mut server);
    resources::register_all(&mut server);
    prompts::register_all(&mut server);

    if let Err(e) = server.run_stdio().await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
