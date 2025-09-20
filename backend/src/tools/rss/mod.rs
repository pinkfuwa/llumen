use reqwest::Url;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tools::Tool;
use tokio::fs;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RssSearch;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RssSearchInput {
    /// the keyword list to search in rss feeds. Keywords should include traditional Chinese and English.
    keywords: Vec<String>,
}
impl Tool for RssSearch {
    type Input = RssSearchInput;
    type Output = String;

    const NAME: &str = "rsssearch";
    const DESCRIPTION: &str = "get rss feed subscribed and filter by keywords, return in xml format";
    const PROMPT: &str = "use `rsssearch` to get rss feed";

    async fn call(&mut self, input: Self::Input) -> anyhow::Result<Self::Output> {
        let mut xml_list = Vec::new();
        let dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../rssfeed"
        );
        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("xml") {
                let content = tokio::fs::read_to_string(&path).await?;
                xml_list.push(content);
            }
        }

        // Use serde_xml_rs only, no regex
        use serde_xml_rs::from_str;
        use serde_xml_rs::to_string;
        #[derive(Debug, Serialize, Deserialize)]
        struct Items {
            #[serde(rename = "item")]
            items: Vec<Item>,
        }
        #[derive(Debug, Serialize, Deserialize)]
        struct Item {
            #[serde(default)]
            keyword: Option<String>,
            // ... other fields ...
        }

        let mut filtered_items_xml = Vec::new();
        for xml in &xml_list {
            let mut start = 0;
            while let Some(item_start) = xml[start..].find("<item>") {
                let item_start = start + item_start;
                if let Some(item_end) = xml[item_start..].find("</item>") {
                    let item_end = item_start + item_end + "</item>".len();
                    let item_block = &xml[item_start..item_end];
                    // Try to extract keywords from the <keyword> tag inside item_block
                    if let Some(kw_start) = item_block.find("<keyword>") {
                        if let Some(kw_end) = item_block[kw_start..].find("</keyword>") {
                            let kw_start = kw_start + "<keyword>".len();
                            let kw_end = kw_start + kw_end - "<keyword>".len();
                            let keywords_str = &item_block[kw_start..kw_end];
                            let xml_keywords: Vec<&str> = keywords_str.split(',').map(|s| s.trim()).collect();
                            let mut match_count = 0;
                            for input_kw in &input.keywords {
                                for xml_kw in &xml_keywords {
                                    if xml_kw.to_lowercase().contains(&input_kw.to_lowercase()) {
                                        match_count += 1;
                                    }
                                }
                            }
                            if match_count > 0 {
                                filtered_items_xml.push(item_block.to_string());
                            }
                        }
                    }
                    start = item_end;
                } else {
                    break;
                }
            }
        }

        // Return filtered items as concatenated XML, or all items if none matched
        let result = if !filtered_items_xml.is_empty() {
            filtered_items_xml.join("\n")
        } else {
            // fallback: concatenate all items from all xmls
            let mut all_items_xml = Vec::new();
            for xml in &xml_list {
                if let Ok(items) = from_str::<Items>(xml) {
                    for item in items.items {
                        if let Ok(xml_str) = to_string(&item) {
                            all_items_xml.push(xml_str);
                        }
                    }
                }
            }
            all_items_xml.join("\n")
        };
        Ok(result)
    }
}
