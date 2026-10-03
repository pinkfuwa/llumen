//! Parse OpenRouter annotation objects.

use protocol::UrlCitation;

/// Extract URL citations from annotations with `type: "url_citation"` returned
/// by OpenRouter web-search server tool.
pub fn extract_url_citations(annotations: &serde_json::Value) -> Vec<UrlCitation> {
    let Some(items) = annotations.as_array() else {
        return Vec::new();
    };

    let mut citations: Vec<UrlCitation> = Vec::new();
    for item in items {
        let Some(obj) = item.as_object() else {
            continue;
        };
        if obj.get("type").and_then(|v| v.as_str()) != Some("url_citation") {
            continue;
        }
        let Some(payload) = obj.get("url_citation") else {
            continue;
        };
        if let Some(citation) = parse_url_citation(payload) {
            if citations
                .iter()
                .any(|existing| existing.url == citation.url)
            {
                continue;
            }
            citations.push(citation);
        }
    }
    citations
}

fn parse_url_citation(value: &serde_json::Value) -> Option<UrlCitation> {
    let obj = value.as_object()?;
    let url = obj.get("url")?.as_str()?.to_string();
    let title = obj
        .get("title")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let content = obj
        .get("content")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let start_index = obj
        .get("start_index")
        .and_then(|v| v.as_i64())
        .and_then(|value| i32::try_from(value).ok());
    let end_index = obj
        .get("end_index")
        .and_then(|v| v.as_i64())
        .and_then(|value| i32::try_from(value).ok());
    let favicon = obj
        .get("favicon")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some(UrlCitation {
        url,
        title,
        content,
        start_index,
        end_index,
        favicon,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_metadata_and_preserves_first_citation_order() {
        let annotations = json!([
            {"type": "url_citation", "url_citation": {
                "url": "https://example.com/first", "title": "First", "content": "Excerpt",
                "start_index": 0, "end_index": 12, "favicon": "https://example.com/icon.png"
            }},
            {"type": "url_citation", "url_citation": {"url": "https://example.com/second"}},
            {"type": "url_citation", "url_citation": {"url": "https://example.com/first", "title": "Duplicate"}}
        ]);
        let citations = extract_url_citations(&annotations);
        assert_eq!(citations.len(), 2);
        let first = &citations[0];
        assert_eq!(first.url, "https://example.com/first");
        assert_eq!(first.title.as_deref(), Some("First"));
        assert_eq!(first.content.as_deref(), Some("Excerpt"));
        assert_eq!(first.start_index, Some(0));
        assert_eq!(first.end_index, Some(12));
        assert_eq!(
            first.favicon.as_deref(),
            Some("https://example.com/icon.png")
        );
        assert_eq!(citations[1].url, "https://example.com/second");
        assert_eq!(citations[1].title, None);
    }

    #[test]
    fn skips_malformed_entries_without_losing_valid_citations() {
        let annotations = json!([
            null, 42, {}, {"type": "other", "url_citation": {"url": "ignored"}},
            {"type": "url_citation"}, {"type": "url_citation", "url_citation": null},
            {"type": "url_citation", "url_citation": {}},
            {"type": "url_citation", "url_citation": {"url": 42}},
            {"type": "url_citation", "url_citation": {"url": "https://example.com"}}
        ]);
        let citations = extract_url_citations(&annotations);
        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].url, "https://example.com");
    }

    #[test]
    fn rejects_non_array_annotations() {
        for annotations in [json!(null), json!({}), json!("text"), json!(42)] {
            assert!(extract_url_citations(&annotations).is_empty());
        }
    }

    #[test]
    fn ignores_invalid_optional_metadata() {
        let citations = extract_url_citations(&json!([{
            "type": "url_citation", "url_citation": {
                "url": "https://example.com", "title": 42, "content": [],
                "favicon": false, "start_index": "0", "end_index": 1.5
            }
        }]));
        assert_eq!(citations.len(), 1);
        let citation = &citations[0];
        assert_eq!(citation.title, None);
        assert_eq!(citation.content, None);
        assert_eq!(citation.favicon, None);
        assert_eq!(citation.start_index, None);
        assert_eq!(citation.end_index, None);
    }

    #[test]
    fn ignores_indices_outside_protocol_integer_range() {
        let citations = extract_url_citations(&json!([{
            "type": "url_citation", "url_citation": {
                "url": "https://example.com",
                "start_index": i64::from(i32::MIN) - 1,
                "end_index": i64::from(i32::MAX) + 1
            }
        }]));
        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].start_index, None);
        assert_eq!(citations[0].end_index, None);
    }
}
