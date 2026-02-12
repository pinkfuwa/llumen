use protocol::UrlCitation;

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
        .map(|v| v as i32);
    let end_index = obj
        .get("end_index")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);
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
