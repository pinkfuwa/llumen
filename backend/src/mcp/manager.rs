use std::collections::HashMap;

use anyhow::Result;
use protocol::ModeKind;
use sea_orm::{DatabaseConnection, EntityTrait};
use tokio::sync::Mutex;

use super::client::McpClient;
use super::config::McpServerConfig;

/// Manages active MCP client connections with lazy initialization.
pub struct McpClientManager {
    db: DatabaseConnection,
    clients: Mutex<HashMap<i32, McpClient>>,
}

impl McpClientManager {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            clients: Mutex::new(HashMap::new()),
        }
    }

    /// Ensure a server is started (lazy init). Returns error if connection
    /// fails.
    async fn ensure_started(&self, server_id: i32) -> Result<()> {
        let mut clients = self.clients.lock().await;
        if clients.contains_key(&server_id) {
            log::debug!("MCP server id={} already running", server_id);
            return Ok(());
        }

        log::info!("Starting MCP server id={}", server_id);

        let server = entity::mcp_server::Entity::find_by_id(server_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("MCP server id={} not found", server_id))?;

        if !server.enabled {
            log::warn!(
                "MCP server '{}' (id={}) is disabled",
                server.name,
                server_id
            );
            anyhow::bail!("MCP server '{}' is disabled", server.name);
        }

        let config = McpServerConfig::parse(&server.config_raw)
            .map_err(|e| anyhow::anyhow!("invalid TOML config for '{}': {}", server.name, e))?;

        let client = McpClient::connect(server_id, &config).await?;
        clients.insert(server_id, client);
        Ok(())
    }

    /// Stop a running server (removes from pool).
    pub async fn stop_server(&self, server_id: i32) {
        let mut clients = self.clients.lock().await;
        if let Some(client) = clients.remove(&server_id) {
            client.shutdown().await;
        }
    }

    /// Call a tool on a specific MCP server.
    /// Returns a ToolOutput (text for LLM + rich content for frontend).
    pub async fn call_tool(
        &self,
        server_id: i32,
        tool_name: &str,
        args: serde_json::Map<String, serde_json::Value>,
    ) -> crate::chat::ToolOutput {
        use crate::chat::ToolOutput;

        if let Err(e) = self.ensure_started(server_id).await {
            return ToolOutput {
                text: format!(
                    "Error: failed to start MCP server (id={}): {}",
                    server_id, e
                ),
                rich: Vec::new(),
            };
        }

        let clients = self.clients.lock().await;
        match clients.get(&server_id) {
            Some(client) => client.call_tool(tool_name, args).await,
            None => ToolOutput {
                text: format!(
                    "Error: MCP server (id={}) not available after start",
                    server_id
                ),
                rich: Vec::new(),
            },
        }
    }

    /// List tools from all enabled servers matching the given mode.
    /// Returns (server_id, tool) pairs.
    pub async fn list_tools_for_mode(&self, mode: ModeKind) -> Vec<(i32, rmcp::model::Tool)> {
        let servers = match entity::mcp_server::Entity::find().all(&self.db).await {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Failed to list MCP servers: {}", e);
                return Vec::new();
            }
        };

        let mut result = Vec::new();

        for server in servers {
            if !server.enabled {
                continue;
            }
            let config = match McpServerConfig::parse(&server.config_raw) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Invalid TOML for MCP server '{}': {}", server.name, e);
                    continue;
                }
            };

            if !config.modes().contains(&mode) {
                continue;
            }

            // Lazy start: ensure connection exists
            if let Err(e) = self.ensure_started(server.id).await {
                log::warn!(
                    "Failed to start MCP server '{}' (id={}): {}",
                    server.name,
                    server.id,
                    e
                );
                continue;
            }

            let clients = self.clients.lock().await;
            if let Some(client) = clients.get(&server.id) {
                for tool in client.tools() {
                    result.push((server.id, tool.clone()));
                }
            }
        }

        result
    }

    /// Check if a server is currently running.
    pub async fn is_running(&self, server_id: i32) -> bool {
        self.clients.lock().await.contains_key(&server_id)
    }

    /// Shutdown all active MCP clients.
    pub async fn shutdown_all(&self) {
        let mut clients = self.clients.lock().await;
        let ids: Vec<i32> = clients.keys().copied().collect();
        for id in ids {
            if let Some(client) = clients.remove(&id) {
                client.shutdown().await;
            }
        }
    }
}
