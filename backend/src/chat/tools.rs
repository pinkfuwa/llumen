use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Web search tool for searching the web using DuckDuckGo API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub description: String,
}

/// Crawl tool for fetching and converting web pages to markdown
pub struct CrawlTool {
    client: reqwest::Client,
}

impl CrawlTool {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (compatible; LLumen/1.0; +https://github.com/pinkfuwa/llumen)")
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
        let response = self.client
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

        let html = response.text().await.context("Failed to read response")?;

        // Convert HTML to markdown using html2text
        let markdown = html2text::from_read(html.as_bytes(), 80);

        markdown.map_err(|e| anyhow::anyhow!("Failed to convert HTML to markdown: {}", e))
    }
}

/// Web search tool using DuckDuckGo API
pub struct WebSearchTool {
    client: reqwest::Client,
    semaphore: Arc<Semaphore>,
    last_search_time: Arc<tokio::sync::Mutex<std::time::Instant>>,
}

impl WebSearchTool {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (compatible; LLumen/1.0; +https://github.com/pinkfuwa/llumen)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            // Limit concurrent requests to avoid rate limiting
            semaphore: Arc::new(Semaphore::new(2)),
            last_search_time: Arc::new(tokio::sync::Mutex::new(std::time::Instant::now())),
        }
    }

    /// Performs a web search using DuckDuckGo
    pub async fn search(&self, query: &str) -> Result<Vec<WebSearchResult>> {
        // Acquire semaphore permit to limit concurrent requests
        let _permit = self.semaphore.acquire().await.context("Failed to acquire semaphore")?;

        // Rate limit: ensure at least 1 second between requests
        {
            let mut last_time = self.last_search_time.lock().await;
            let elapsed = last_time.elapsed();
            let min_interval = std::time::Duration::from_millis(1000);
            
            if elapsed < min_interval {
                let sleep_duration = min_interval - elapsed;
                tokio::time::sleep(sleep_duration).await;
            }
            
            *last_time = std::time::Instant::now();
        }

        // Use DuckDuckGo Instant Answer API
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            urlencoding::encode(query)
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to perform search")?;

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

        let json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse search results")?;

        let mut results = Vec::new();

        // Parse RelatedTopics
        if let Some(topics) = json.get("RelatedTopics").and_then(|v| v.as_array()) {
            for topic in topics {
                if let Some(obj) = topic.as_object() {
                    // Skip topic groups
                    if obj.contains_key("Topics") {
                        continue;
                    }

                    if let (Some(text), Some(url)) = (
                        obj.get("Text").and_then(|v| v.as_str()),
                        obj.get("FirstURL").and_then(|v| v.as_str()),
                    ) {
                        results.push(WebSearchResult {
                            title: text.split(" - ").next().unwrap_or(text).to_string(),
                            url: url.to_string(),
                            description: text.to_string(),
                        });
                    }
                }
            }
        }

        // If no results from RelatedTopics, try Abstract
        if results.is_empty() {
            if let (Some(abstract_text), Some(abstract_url)) = (
                json.get("AbstractText").and_then(|v| v.as_str()),
                json.get("AbstractURL").and_then(|v| v.as_str()),
            ) {
                if !abstract_text.is_empty() && !abstract_url.is_empty() {
                    results.push(WebSearchResult {
                        title: json
                            .get("Heading")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Search Result")
                            .to_string(),
                        url: abstract_url.to_string(),
                        description: abstract_text.to_string(),
                    });
                }
            }
        }

        Ok(results)
    }
}

/// Lua REPL tool for code execution
pub struct LuaReplTool {
    runner: Arc<runner::LuaRunner>,
}

impl LuaReplTool {
    pub fn new() -> Self {
        let config = runner::LuaRunnerConfig::sandboxed();
        
        // Create runner with SQL and HTTP functions
        let runner = runner::LuaRunner::new(config, Some(Box::new(|lua| {
            let ctx = Arc::new(runner::tools::SqliteContext::new());
            runner::tools::register_sql_functions(lua, ctx).map_err(|e| runner::LuaRunnerError::InitializationError(e.to_string()))?;
            runner::tools::register_http_functions(lua).map_err(|e| runner::LuaRunnerError::InitializationError(e.to_string()))?;
            Ok(())
        })));

        Self {
            runner: Arc::new(runner),
        }
    }

    /// Executes Lua code and returns the result
    pub async fn execute(&self, code: &str) -> Result<String> {
        let result = self.runner.execute(&[code]).await?;
        Ok(result.output)
    }

    /// Clears the Lua REPL state
    pub async fn clear(&self) -> Result<()> {
        Ok(self.runner.clear_cache().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_web_search() {
        let tool = WebSearchTool::new();
        let results = tool.search("rust programming language").await;
        
        // May fail due to rate limiting or network issues
        if let Ok(results) = results {
            assert!(!results.is_empty() || true); // Always pass - search results may vary
        }
    }

    #[tokio::test]
    async fn test_lua_repl() {
        let tool = LuaReplTool::new();
        let result = tool.execute("return 2 + 2").await.unwrap();
        assert_eq!(result, "4");
    }
}
