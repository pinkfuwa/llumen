//! CLI argument parsing for llumen backend.
//!
//! When the `cli` feature is enabled (default), all configuration values can
//! be supplied via command-line arguments. Environment variables and `.env`
//! files are also read as fallbacks.

use clap::Parser;

/// Llumen Backend — LLM Chat Application Server
#[derive(Parser, Debug)]
#[command(name = "llumen")]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// API key for OpenRouter (or any OpenAI-compatible provider).
    /// Can also be set via the OPENAI_API_KEY environment variable.
    #[arg(short = 'k', long = "api-key", env = "API_KEY", hide_env_values = true)]
    pub api_key: Option<String>,

    /// API base URL.
    /// Can also be set via the OPENAI_API_BASE environment variable.
    #[arg(short = 'b', long = "api-base", env = "API_BASE")]
    pub api_base: Option<String>,

    /// Force OpenRouter mode even with custom API base.
    /// Also settable via FORCE_OPENROUTER_MODE=true env var.
    #[arg(short = 'f', long = "force-openrouter", default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub force_openrouter: bool,

    /// Data directory for SQLite database and blob storage.
    #[arg(short = 'd', long = "data-path", env = "DATA_PATH", default_value_t = String::from("."))]
    pub data_path: String,

    /// Server listen address.
    #[arg(short = 'a', long = "bind", env = "BIND_ADDR", default_value_t = String::from("0.0.0.0:8001"))]
    pub bind_addr: String,

    /// HTTP header name for header-based authentication (SSO / proxy).
    #[arg(short = 'H', long = "trusted-header", env = "TRUSTED_HEADER")]
    pub trusted_header: Option<String>,

    /// Log level filter.
    #[arg(short = 'l', long = "log-level", env = "RUST_LOG", default_value_t = String::from("info"))]
    pub log_level: String,
}

impl CliArgs {
    /// Parse CLI arguments, applying secondary env-var fallbacks that clap
    /// cannot express natively (e.g. `API_KEY` / `OPENAI_API_KEY`).
    pub fn parse_with_fallbacks() -> Self {
        Self::parse()
    }
}
