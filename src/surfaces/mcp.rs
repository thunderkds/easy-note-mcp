//! The MCP server over stdio. No tools yet (a later surface task adds them).
//! The tokio runtime is built here, inside `serve`, so the CLI path never starts it.

use rmcp::model::{Implementation, ServerCapabilities, ServerConfig};
use rmcp::transport::stdio;
use rmcp::{ServerHandler, ServiceExt};

#[derive(Debug, Clone, Default)]
struct EasyNote;

impl ServerHandler for EasyNote {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::default())
            .with_server_info(Implementation::new("easy-note", env!("CARGO_PKG_VERSION")))
    }
}

/// Serve MCP on stdin/stdout until the client closes stdin.
pub fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let running = EasyNote.serve(stdio()).await?;
        running.waiting().await?;
        Ok(())
    })
}
