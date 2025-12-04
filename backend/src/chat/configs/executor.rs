use std::sync::Arc;

use crate::chat::Context;

pub async fn execute_search_tool(ctx: &Arc<Context>, tool_name: &str, args: &str) -> String {
    use serde::Deserialize;

    match tool_name {
        "web_search_tool" => {
            #[derive(Deserialize)]
            struct WebSearchArgs {
                query: String,
            }
            let args: Option<WebSearchArgs> = serde_json::from_str(args).ok();
            if args.is_none() {
                return "Invalid arguments for web_search_tool".to_string();
            }
            let args = args.unwrap();
            match ctx.web_search_tool.search(&args.query).await {
                Ok(results) => {
                    let mut output = String::new();
                    for (i, result) in results.iter().enumerate().take(10) {
                        output.push_str(&format!(
                            "{}. [{}]({})\n   {}\n\n",
                            i + 1,
                            result.title,
                            result.url,
                            result.description
                        ));
                    }

                    if output.is_empty() {
                        output = "No search results found.".to_string();
                    }

                    output
                }
                Err(e) => {
                    log::warn!("Web search error: {}", e);
                    format!("Error: {}", e)
                }
            }
        }
        "crawl_tool" => {
            #[derive(Deserialize)]
            struct CrawlArgs {
                url: String,
            }
            let args: Option<CrawlArgs> = serde_json::from_str(args).ok();
            if args.is_none() {
                return "Invalid arguments for crawl_tool".to_string();
            }
            let args = args.unwrap();
            match ctx.crawl_tool.crawl(&args.url).await {
                Ok(content) => content,
                Err(e) => {
                    log::warn!("Crawl error for URL '{}': {}", args.url, e);
                    format!("Error: {}", e)
                }
            }
        }
        _ => format!("Unknown tool: {}", tool_name),
    }
}
