//! Environment configuration for llumen backend.
//!
//! [`Environment`] collects all startup configuration from environment variables
//! (and CLI args when the `cli` feature is enabled). It is the single source of
//! truth for runtime parameters.

use std::path::PathBuf;

#[cfg(not(feature = "cli"))]
use crate::config::DEFAULT_BIND_ADDR;

/// All configuration values needed to start the server.
///
/// Populated from environment variables (always) and CLI args (when the `cli`
/// feature is enabled).
pub struct Environment {
    pub api_key: String,
    pub api_base: String,
    pub force_openrouter: bool,
    pub data_path: PathBuf,
    pub bind_addr: String,
    pub auth_header: Option<String>,
    pub log_level: String,
}

impl Environment {
    /// Load configuration from OS environment variables (and `.env` file).
    #[cfg(not(feature = "cli"))]
    pub fn load() -> Self {
        let api_key = Self::load_api_key();
        let api_base = dotenvy::var("API_BASE")
            .or_else(|_| dotenvy::var("OPENAI_API_BASE"))
            .unwrap_or_else(|_| "https://openrouter.ai/api".to_string());
        let force_openrouter = dotenvy::var("FORCE_OPENROUTER_MODE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);
        let data_path = PathBuf::from(dotenvy::var("DATA_PATH").unwrap_or_else(|_| ".".to_owned()));
        let bind_addr = dotenvy::var("BIND_ADDR").unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_owned());
        let auth_header = dotenvy::var("TRUSTED_HEADER").ok();
        let log_level = dotenvy::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Self {
            api_key,
            api_base,
            force_openrouter,
            data_path,
            bind_addr,
            auth_header,
            log_level,
        }
    }

    /// Load configuration from CLI args with env var fallback.
    #[cfg(feature = "cli")]
    pub fn load_from(cli: &crate::utils::cli::CliArgs) -> Self {
        let api_key = match (
            cli.api_key.clone(),
            dotenvy::var("OPENAI_API_KEY").ok(),
        ) {
            (Some(key), _) => key,
            (None, Some(key)) => key,
            (None, None) => {
                Self::print_api_key_help();
                std::process::exit(1);
            }
        };

        let api_base = cli
            .api_base
            .clone()
            .or_else(|| dotenvy::var("OPENAI_API_BASE").ok())
            .unwrap_or_else(|| "https://openrouter.ai/api".to_string());

        let force_openrouter = if cli.force_openrouter {
            true
        } else {
            dotenvy::var("FORCE_OPENROUTER_MODE")
                .ok()
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false)
        };

        let data_path = PathBuf::from(cli.data_path.clone());

        let bind_addr = cli.bind_addr.clone();

        let auth_header = cli
            .trusted_header
            .clone()
            .or_else(|| dotenvy::var("TRUSTED_HEADER").ok());

        let log_level = cli.log_level.clone();

        Self {
            api_key,
            api_base,
            force_openrouter,
            data_path,
            bind_addr,
            auth_header,
            log_level,
        }
    }

    fn print_api_key_help() {
        println!("Error: API_KEY environment variable not found.");
        println!("Note: llumen read environment variable as well as .env file.");
        println!("You can get a key from https://openrouter.ai/keys");
        println!("Or use alternative setup:");
        println!(
            "- configuration: https://pinkfuwa.github.io/llumen/user/config/environment"
        );
        println!("- documentation: https://pinkfuwa.github.io/llumen/");

        #[cfg(windows)]
        {
            use std::io::{self, Read};
            println!("Press Enter to exit...");
            io::stdin().read_exact(&mut [0u8]).unwrap();
        }
    }

    #[cfg(not(feature = "cli"))]
    fn load_api_key() -> String {
        match (dotenvy::var("API_KEY"), dotenvy::var("OPENAI_API_KEY")) {
            (Ok(key), _) => key,
            (_, Ok(key)) => key,
            _ => {
                Self::print_api_key_help();
                std::process::exit(1);
            }
        }
    }
}