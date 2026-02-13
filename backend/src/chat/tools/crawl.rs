use crate::utils::url_validation;
use anyhow::{Context, Result};
use tokio::time;

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
        url_validation::validate_url(url)
            .await
            .context("Invalid URL")?;

        // Fetch the page
        let response = loop {
            let response = self
                .client
                .get(url)
                .send()
                .await
                .context("Failed to fetch URL")?;

            // Check for rate limiting
            match response.headers().get("Retry-After") {
                Some(retry_after) => {
                    let retry_seconds = retry_after
                        .to_str()
                        .ok()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(1);
                    time::sleep(time::Duration::from_secs(retry_seconds)).await;
                }
                None => break response,
            };
        };

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("HTTP error: {}", status);
        }

        // Get response bytes
        let blob = response.bytes().await.context("Failed to read response")?;

        if infer::is_image(&blob)
            || infer::is_audio(&blob)
            || infer::archive::is_pdf(&blob)
            || infer::is_document(&blob)
            || infer::is_book(&blob)
        {
            anyhow::bail!("This URL returns an unsupported content type.");
        }

        if blob.len() > 1_000_000 {
            anyhow::bail!("This URL returns a content that is too large.");
        }

        let str_content = str::from_utf8(&blob).context("Failed to convert bytes to string")?;
        let parsed_text = html2text::from_read(str_content.as_bytes(), 1000)
            .unwrap_or_else(|_| str_content.to_string());
        Ok(parsed_text)
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
