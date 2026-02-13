use protocol::ModeKind;
use serde::Deserialize;
use std::collections::HashMap;

/// Parsed MCP server configuration from TOML.
#[derive(Debug, Clone, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub transport: TransportKind,
    #[serde(default)]
    pub attached_modes: Vec<String>,
    #[serde(default)]
    pub stdio: Option<StdioConfig>,
    #[serde(default)]
    pub tcp: Option<TcpConfig>,
    #[serde(default)]
    pub sse: Option<SseConfig>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransportKind {
    Stdio,
    Tcp,
    Sse,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StdioConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TcpConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SseConfig {
    pub url: String,
}

impl McpServerConfig {
    /// Parse raw TOML string into config (validation only).
    pub fn parse(raw: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(raw)
    }

    /// Returns the modes this server is attached to.
    pub fn modes(&self) -> Vec<ModeKind> {
        self.attached_modes
            .iter()
            .filter_map(|m| match m.as_str() {
                "normal" => Some(ModeKind::Normal),
                "search" => Some(ModeKind::Search),
                "research" => Some(ModeKind::Research),
                _ => None,
            })
            .collect()
    }
}

impl std::fmt::Display for TransportKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportKind::Stdio => write!(f, "stdio"),
            TransportKind::Tcp => write!(f, "tcp"),
            TransportKind::Sse => write!(f, "sse"),
        }
    }
}
