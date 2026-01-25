use crate::runner;
use anyhow::{Context, Result};

/// Crawl tool for fetching and converting web pages to markdown
pub struct CrawlTool {
    client: reqwest::Client,
}

impl CrawlTool {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(
                    "Mozilla/5.0 (compatible; LLumen/1.0; +https://github.com/pinkfuwa/llumen)",
                )
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Crawls a URL and converts the content to markdown
    pub async fn crawl(&self, url: &str) -> Result<String> {
        // Validate URL
        runner::tools::validate_url(url)
            .await
            .context("Invalid URL")?;

        // Fetch the page
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to fetch URL")?;

        // Check for rate limiting
        if let Some(retry_after) = response.headers().get("Retry-After") {
            let retry_seconds = retry_after
                .to_str()
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(60);

            anyhow::bail!(
                "Rate limited. Please retry after {} seconds.",
                retry_seconds
            );
        }

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("HTTP error: {}", status);
        }

        // Get response bytes
        let bytes = response.bytes().await.context("Failed to read response")?;

        // Limit content to 120,000 characters
        const MAX_CHARS: usize = 120_000;
        let content_bytes = if bytes.len() > MAX_CHARS {
            &bytes[..MAX_CHARS]
        } else {
            &bytes
        };

        // Use infer to detect actual content type from magic bytes
        if let Some(kind) = infer::get(content_bytes) {
            let mime_type = kind.mime_type();

            // Reject image content types
            if mime_type.starts_with("image/") {
                anyhow::bail!(
                    "This URL returns an Image content type. Image parsing is not supported."
                );
            }

            // Reject PDF content types
            if mime_type.contains("pdf") {
                anyhow::bail!("This URL returns a PDF content type. PDF parsing is not supported.");
            }

            // Reject other document types
            if mime_type.contains("application/msword")
                || mime_type.contains("application/vnd.openxmlformats-officedocument")
                || mime_type.contains("application/vnd.ms-excel")
                || mime_type.contains("application/vnd.ms-powerpoint")
                || mime_type.contains("application/zip")
                || mime_type.contains("application/x-tar")
                || mime_type.contains("application/x-rar")
            {
                anyhow::bail!(
                    "This URL returns a Document content type. Document parsing is not supported."
                );
            }

            // Reject other binary formats
            if mime_type.starts_with("video/") || mime_type.starts_with("audio/") {
                anyhow::bail!(
                    "This URL returns a media file content type. Media file parsing is not supported."
                );
            }
        }

        // Convert bytes to string
        let content_str = String::from_utf8_lossy(content_bytes);

        // Check if content looks like HTML
        let trimmed = content_str.trim();
        let is_html = trimmed.starts_with("<!DOCTYPE html")
            || trimmed.starts_with("<!doctype html")
            || trimmed.starts_with("<html")
            || trimmed.starts_with("<HTML")
            || trimmed.contains("<body")
            || trimmed.contains("<div");

        if is_html {
            // Convert HTML to markdown using html2text
            let markdown = html2text::from_read(content_str.as_bytes(), 80)
                .map_err(|e| anyhow::anyhow!("Failed to convert HTML to markdown: {}", e))?;
            Ok(markdown)
        } else {
            // Return plain text as-is
            Ok(content_str.to_string())
        }
    }
}

pub fn get_crawl_tool_def() -> crate::openrouter::Tool {
    crate::openrouter::Tool {
        name: "crawl_tool".to_string(),
        description: "Crawl and extract content from a specific URL.".to_string(),
        schema: serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to crawl and extract content from."
                }
            },
            "required": ["url"]
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crawl_tool_invalid_url() {
        let tool = CrawlTool::new();
        // Test that invalid URL returns an error
        let result = tool.crawl("not-a-valid-url").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_crawl_tool_private_ip() {
        let tool = CrawlTool::new();
        // Test that private IP addresses are rejected
        let result = tool.crawl("http://192.168.1.1/test").await;
        assert!(result.is_err());
        if let Err(e) = result {
            let error_str = e.to_string().to_lowercase();
            assert!(error_str.contains("private") || error_str.contains("invalid"));
        }
    }
}
