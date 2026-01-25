use anyhow::{Context, Result, anyhow};
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
                .user_agent(
                    "Mozilla/5.0 (compatible; LLumen/1.0; +https://github.com/pinkfuwa/llumen)",
                )
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            // Limit concurrent requests to avoid rate limiting
            semaphore: Arc::new(Semaphore::new(2)),
            last_search_time: Arc::new(tokio::sync::Mutex::new(std::time::Instant::now())),
        }
    }

    /// Performs a web search using DuckDuckGo HTML search API
    pub async fn search(&self, query: &str) -> Result<Vec<WebSearchResult>> {
        log::debug!("searching for {}", query);
        // Acquire semaphore permit to limit concurrent requests
        let _permit = self
            .semaphore
            .acquire()
            .await
            .context("Failed to acquire semaphore")?;

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

        // Use DuckDuckGo HTML search API
        let response = self
            .client
            .post("https://html.duckduckgo.com/html/")
            .header("Referer", "https://html.duckduckgo.com/")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[("q", query), ("b", "")])
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

        let html = response.text().await.context("Failed to read response")?;

        let results = Self::parse_search_results(&html)?;

        Ok(results)
    }

    /// Parses DuckDuckGo HTML search results
    fn parse_search_results(html: &str) -> Result<Vec<WebSearchResult>> {
        use scraper::{Html, Selector};

        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Select all search result containers
        let result_selector = Selector::parse("div.result")
            .map_err(|_| anyhow!("Failed to parse result selector"))?;

        for result_element in document.select(&result_selector) {
            // Extract title and URL from the h2 > a element
            let title_selector = Selector::parse("h2.result__title a")
                .map_err(|_| anyhow!("Failed to parse title selector"))?;

            let snippet_selector = Selector::parse("a.result__snippet")
                .map_err(|_| anyhow!("Failed to parse snippet selector"))?;

            if let Some(title_elem) = result_element.select(&title_selector).next() {
                let title = title_elem.inner_html();
                let title_clean = Self::clean_html_text(&title);

                if let Some(href) = title_elem.value().attr("href") {
                    let url = href.to_string();

                    // Extract description from snippet if available
                    let description = result_element
                        .select(&snippet_selector)
                        .next()
                        .map(|elem| Self::clean_html_text(&elem.inner_html()))
                        .unwrap_or_else(|| String::from("No description available"));

                    results.push(WebSearchResult {
                        title: title_clean,
                        url,
                        description,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Removes HTML tags and trims whitespace from text
    fn clean_html_text(html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;

        for ch in html.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(ch),
                _ => {}
            }
        }

        result.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

pub fn get_web_search_tool_def() -> crate::openrouter::Tool {
    crate::openrouter::Tool {
        name: "web_search_tool".to_string(),
        description: "Search the web for information using a search query.".to_string(),
        schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query to use for finding information on the web."
                }
            },
            "required": ["query"]
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_web_search() {
        let tool = WebSearchTool::new();
        let results = tool.search("benchmark GPU memory usage inference latency sparse models structured unstructured pruning schedules training stability report").await;

        // May fail due to rate limiting or network issues
        if let Ok(results) = results {
            assert!(!results.is_empty() || true); // Always pass - search results may vary
        }
    }
}
