use std::collections::HashMap;

use anyhow::{Context, Result};
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParams, Content, RawContent},
    service::{RoleClient, RunningService},
    transport::{ConfigureCommandExt, StreamableHttpClientTransport, TokioChildProcess},
};
use tokio::process::Command;

use super::config::{McpServerConfig, TransportKind};

type McpRunning = RunningService<RoleClient, ()>;

/// Wraps a running rmcp client with metadata.
pub struct McpClient {
    running: McpRunning,
    pub server_name: String,
    pub server_id: i32,
    tools: Vec<rmcp::model::Tool>,
}

impl McpClient {
    /// Connect to an MCP server using the parsed config.
    pub async fn connect(server_id: i32, config: &McpServerConfig) -> Result<Self> {
        log::info!(
            "Connecting to MCP server '{}' (id={}) via {:?}",
            config.name,
            server_id,
            config.transport
        );

        let running = match config.transport {
            TransportKind::Stdio => {
                let stdio_cfg = config
                    .stdio
                    .as_ref()
                    .context("missing [stdio] section in config")?;

                let env: HashMap<String, String> = stdio_cfg.env.clone();
                let args: Vec<String> = stdio_cfg.args.clone();
                let command_str = stdio_cfg.command.clone();

                log::debug!(
                    "Starting stdio MCP server: {} {}",
                    command_str,
                    args.join(" ")
                );

                let transport =
                    TokioChildProcess::new(Command::new(&command_str).configure(move |cmd| {
                        for arg in &args {
                            cmd.arg(arg);
                        }
                        for (k, v) in &env {
                            cmd.env(k, v);
                        }
                    }))?;
                ().serve(transport)
                    .await
                    .context("failed to connect to stdio MCP server")?
            }
            TransportKind::Sse => {
                let sse_cfg = config
                    .sse
                    .as_ref()
                    .context("missing [sse] section in config")?;
                log::debug!("Connecting to SSE MCP server at {}", sse_cfg.url);
                let transport = StreamableHttpClientTransport::from_uri(sse_cfg.url.as_str());
                ().serve(transport)
                    .await
                    .context("failed to connect to SSE MCP server")?
            }
            TransportKind::Tcp => {
                anyhow::bail!("TCP transport not yet supported by rmcp client");
            }
        };

        let tools = match running.list_all_tools().await {
            Ok(tools) => {
                log::info!(
                    "MCP server '{}' connected successfully with {} tools",
                    config.name,
                    tools.len()
                );
                for tool in &tools {
                    log::debug!("  - {}", tool.name);
                }
                tools
            }
            Err(e) => {
                log::error!(
                    "Failed to list tools from MCP server '{}': {}",
                    config.name,
                    e
                );
                return Err(e.into());
            }
        };

        Ok(Self {
            running,
            server_name: config.name.clone(),
            server_id,
            tools,
        })
    }

    pub fn tools(&self) -> &[rmcp::model::Tool] {
        &self.tools
    }

    /// Call a tool by name. Returns structured output (never panics).
    pub async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Map<String, serde_json::Value>,
    ) -> crate::chat::ToolOutput {
        use crate::chat::{McpRichContent, ToolOutput};

        let params = CallToolRequestParams {
            meta: None,
            name: name.to_string().into(),
            arguments: Some(args),
            task: None,
        };
        match self.running.call_tool(params).await {
            Ok(result) => {
                let is_error = result.is_error == Some(true);
                let (text, rich) = format_content(&result.content);
                let text = if is_error {
                    format!("Error from tool '{}': {}", name, text)
                } else {
                    text
                };
                ToolOutput { text, rich }
            }
            Err(e) => ToolOutput {
                text: format!(
                    "Error: MCP server '{}' (id={}) tool '{}' failed: {}",
                    self.server_name, self.server_id, name, e
                ),
                rich: Vec::new(),
            },
        }
    }

    /// Refresh the cached tool list.
    pub async fn refresh_tools(&mut self) {
        self.tools = self.running.list_all_tools().await.unwrap_or_default();
    }

    /// Gracefully shut down the client.
    pub async fn shutdown(mut self) {
        let _ = self.running.close().await;
    }
}

/// Format MCP content blocks into text (for LLM) and rich content (for
/// frontend).
fn format_content(content: &[Content]) -> (String, Vec<crate::chat::McpRichContent>) {
    use crate::chat::McpRichContent;
    let mut text_parts = Vec::new();
    let mut rich = Vec::new();
    for c in content {
        match &c.raw {
            RawContent::Text(t) => text_parts.push(t.text.clone()),
            RawContent::Image(img) => {
                text_parts.push(format!("[image: {}]", img.mime_type));
                rich.push(McpRichContent::Image {
                    data: img.data.clone(),
                    mime_type: img.mime_type.clone(),
                });
            }
            RawContent::Resource(res) => match &res.resource {
                rmcp::model::ResourceContents::TextResourceContents {
                    text,
                    uri,
                    mime_type,
                    ..
                } => {
                    text_parts.push(text.clone());
                    rich.push(McpRichContent::Resource {
                        uri: uri.to_string(),
                        mime_type: mime_type.clone(),
                        text: Some(text.clone()),
                    });
                }
                rmcp::model::ResourceContents::BlobResourceContents { uri, mime_type, .. } => {
                    text_parts.push("[embedded resource: binary]".to_string());
                    rich.push(McpRichContent::Resource {
                        uri: uri.to_string(),
                        mime_type: mime_type.clone(),
                        text: None,
                    });
                }
            },
            RawContent::Audio(audio) => {
                text_parts.push(format!(
                    "[audio: {} ({} bytes)]",
                    audio.mime_type,
                    audio.data.len()
                ));
            }
            RawContent::ResourceLink(link) => {
                text_parts.push(format!("[resource: {}]", link.uri));
                rich.push(McpRichContent::Resource {
                    uri: link.uri.to_string(),
                    mime_type: link.mime_type.clone(),
                    text: None,
                });
            }
        }
    }
    (text_parts.join("\n"), rich)
}
