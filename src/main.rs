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
        env!("CARGO_PKG_NAME").to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
        env!("CARGO_PKG_DESCRIPTION").to_string(),
    );

    tools::register_all(&mut server);
    resources::register_all(&mut server);
    prompts::register_all(&mut server);

    if let Err(e) = server.run_stdio().await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
