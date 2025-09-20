use base64::{Engine as _, engine::general_purpose};
use reqwest::header::{ACCEPT, AUTHORIZATION};
use serde_json::Value;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de};

use crate::tools::Tool;
use dotenv::var;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RecentMail;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ReplyMail;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SendMail;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GetMailContent;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReplyMailInput {
    thread_id: String,
    recipient_email: String,
    subject: String,
    body: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RecentMailInput {
    max_results: Option<u32>, // max number of results to return, default to 10, max 20
    q: Option<String>,        // query string to filter results, same as Gmail search box
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SendMailInput {
    to: String,
    subject: String,
    body: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetMailContentInput {
    mail_id: String,
}

/// Refreshes a Google OAuth 2.0 access token.
/// Returns Ok(access_token) if successful, or Err(error_message) otherwise.
async fn refresh_google_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> anyhow::Result<String> {
    let token_url = "https://oauth2.googleapis.com/token";
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let client = reqwest::Client::new();
    let resp = client
        .post(token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e: reqwest::Error| anyhow::anyhow!(e))?;

    tracing::debug!("Token refresh response status: {}", resp.status());
    if resp.status().is_success() {
        let json: Value = resp
            .json()
            .await
            .map_err(|e: reqwest::Error| anyhow::anyhow!(e))?;
        if let Some(access_token) = json.get("access_token").and_then(|v| v.as_str()) {
            Ok(access_token.to_string())
        } else {
            Err(anyhow::anyhow!("No access_token found in response"))
        }
    } else {
        let error_json: Value = resp
            .json()
            .await
            .map_err(|e: reqwest::Error| anyhow::anyhow!(e))?;
        let error_desc = error_json
            .get("error_description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_owned())
            .unwrap_or_else(|| "Unknown error".to_owned());
        Err(anyhow::anyhow!(error_desc))
    }
}

async fn fetch_latest_gmail_messages_as_string(
    access_token: &str,
    max_results: i32,
    q: &str,
) -> anyhow::Result<String> {
    let api_list_url = "https://gmail.googleapis.com/gmail/v1/users/me/messages";
    let client = reqwest::Client::new();
    let mut result = String::new();

    // Step 1: Get message ID list
    let list_response = client
        .get(api_list_url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(ACCEPT, "application/json")
        .query(&[
            ("maxResults", max_results.to_string()),
            ("q", q.to_string()),
        ])
        .send()
        .await?;

    let status = list_response.status();
    if !status.is_success() {
        result.push_str(&format!("Request fail，error: {}\n", status));
        if status == reqwest::StatusCode::UNAUTHORIZED {
            result.push_str(
                "Access Token is invalid or expired. Please obtain a new Access Token.\n",
            );
        }
        return Ok(result);
    }

    let messages_data: Value = list_response.json().await?;
    tracing::debug!("List response: {}", messages_data);
    if let Some(messages) = messages_data.get("messages").and_then(|m| m.as_array()) {
        result.push_str("------------------------------\n");
        let empty_vec = Vec::new();
        for (i, message_info) in messages.iter().enumerate() {
            let thread_id = message_info
                .get("threadId")
                .and_then(|id| id.as_str())
                .unwrap_or("Unknown Thread");
            if let Some(message_id) = message_info.get("id").and_then(|id| id.as_str()) {
                let api_get_url = format!("{}/{}", api_list_url, message_id);
                let message_response = client
                    .get(&api_get_url)
                    .header(AUTHORIZATION, format!("Bearer {}", access_token))
                    .header(ACCEPT, "application/json")
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?;
                if !message_response.status().is_success() {
                    result.push_str(&format!(
                        "Unable to get the mail with the ID: {}，Error: {}\n",
                        message_id,
                        message_response.status()
                    ));
                    continue;
                }
                let message_full: Value = message_response
                    .json()
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?;
                let payload = &message_full["payload"];
                // Get headers
                let headers_data = payload["headers"].as_array().unwrap_or(&empty_vec);
                let subject = headers_data
                    .iter()
                    .find(|h| h["name"] == "Subject")
                    .and_then(|h| h["value"].as_str())
                    .unwrap_or("No Title");
                let sender = headers_data
                    .iter()
                    .find(|h| h["name"] == "From")
                    .and_then(|h| h["value"].as_str())
                    .unwrap_or("Unknown Sender");
                let date = headers_data
                    .iter()
                    .find(|h| h["name"] == "Date")
                    .and_then(|h| h["value"].as_str())
                    .unwrap_or("Unknown Date");
                // Get body
                let body_content = if let Some(parts) =
                    payload.get("parts").and_then(|p| p.as_array())
                {
                    let mut found = None;
                    for part in parts {
                        if part["mimeType"] == "text/plain" {
                            if let Some(data) = part["body"].get("data").and_then(|d| d.as_str()) {
                                found = Some(data);
                                break;
                            }
                        }
                    }
                    found
                } else if let Some(data) = payload
                    .get("body")
                    .and_then(|b| b.get("data"))
                    .and_then(|d| d.as_str())
                {
                    Some(data)
                } else {
                    None
                };

                let body_text = if let Some(data) = body_content {
                    match general_purpose::URL_SAFE.decode(data) {
                        Ok(decoded) => {
                            let full_text = String::from_utf8_lossy(&decoded);
                            full_text.chars().take(100).collect::<String>()
                        }
                        Err(_) => "Unable to parse the content.".to_string(),
                    }
                } else {
                    "Unable to parse the content.".to_string()
                };

                result.push_str(&format!("----- Mail {} -----\n", i + 1));
                result.push_str(&format!(
                    "mail_id: {}, thread_id: {}\n",
                    message_id, thread_id
                ));
                result.push_str(&format!("Sender: {}\n", sender));
                result.push_str(&format!("Date: {}\n", date));
                result.push_str(&format!("Title: {}\n", subject));
                result.push_str("Content (first 100 chars):\n");
                result.push_str(&format!("{}\n", body_text));
                result.push_str("------------------------------\n");
            }
        }
    } else {
        result.push_str("No mails\n");
    }
    Ok(result)
}

impl Tool for RecentMail {
    type Input = RecentMailInput;
    type Output = String;

    const NAME: &str = "recentmail";
    const DESCRIPTION: &str = "get recentmail info.
    the result will include sender, mail_id, thread_id, date, title and the first 100 characters of the content of each mail.
    mail_id can be used to fetch the full content of the mail using other tools.
    thread_id can be used to reply to the mail using other tools.
    max_results is optional, if not provided, it will default to 10.
    q is optional, if not provided, it will be label:inbox. You can use the same query syntax as in Gmail search box.
    You can ask the user which mail they want to know about, and use getmailcontent or replymail tool to get the content or reply to the mail.
    ";
    const PROMPT: &str = "use `recentmail` to get recent mail";

    async fn call(&mut self, input: Self::Input) -> anyhow::Result<Self::Output> {
        let client_id = var("CLIENT_ID").unwrap_or("".to_owned());
        let client_secret = var("CLIENT_SECRET").unwrap_or("".to_owned());
        let refresh_token = var("REFRESH_TOKEN").unwrap_or("".to_owned());
        tracing::debug!(
            "client_id: {}, client_secret: {}, refresh_token: {}",
            client_id,
            client_secret,
            refresh_token
        );
        let access_token =
            refresh_google_access_token(&client_id, &client_secret, &refresh_token).await?;
        tracing::debug!("access_token: {}", access_token);
        let max_results = std::cmp::min(input.max_results.unwrap_or(10), 20) as i32;
        let q = input.q.unwrap_or("label:inbox".to_owned());
        let result = fetch_latest_gmail_messages_as_string(&access_token, max_results, &q).await?;
        Ok(result)
    }
}

impl Tool for ReplyMail {
    type Input = ReplyMailInput;
    type Output = String;

    const NAME: &str = "replymail";
    const DESCRIPTION: &str = "reply to a mail using the mail_id obtained from recentmail tool.
    thread_id is the thread_id of the mail to reply to. this id should be first obtained from recentmail tool.
    recipient_email is the email address of the recipient.
    subject is the subject of the reply mail.
    body is the content of the reply mail.
    ";
    const PROMPT: &str = "use `replymail` to reply a mail";

    async fn call(&mut self, input: Self::Input) -> anyhow::Result<Self::Output> {
        let client_id = var("CLIENT_ID").unwrap_or("".to_owned());
        let client_secret = var("CLIENT_SECRET").unwrap_or("".to_owned());
        let refresh_token = var("REFRESH_TOKEN").unwrap_or("".to_owned());
        let access_token =
            refresh_google_access_token(&client_id, &client_secret, &refresh_token).await?;
        let api_send_url = "https://gmail.googleapis.com/gmail/v1/users/me/messages/send";
        let client = reqwest::Client::new();

        // RFC 2047 encode subject (MIME encoded-word)
        let subject_encoded = format!(
            "=?UTF-8?B?{}?=",
            base64::engine::general_purpose::STANDARD.encode(input.subject.as_bytes())
        );

        let email_content = format!(
            "Subject: {}\r\nContent-Type: text/plain; charset=\"UTF-8\"\r\nTo: {}\r\nIn-Reply-To: {}\r\nReferences: {}\r\n\r\n{}",
            subject_encoded, input.recipient_email, input.thread_id, input.thread_id, input.body
        );
        let encoded_email = general_purpose::STANDARD.encode(email_content);

        let body = serde_json::json!({
            "raw": encoded_email,
            "threadId": input.thread_id,
        });

        let response = client
            .post(api_send_url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(ACCEPT, "application/json")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok("Reply sent successfully.".to_string())
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!(format!(
                "Failed to send reply. Status: {}, Error: {}",
                status, error_text
            )))
        }
    }
}

impl Tool for SendMail {
    type Input = SendMailInput;
    type Output = String;

    const NAME: &str = "sendmail";
    const DESCRIPTION: &str = "send a mail to a recipient.
    to is the recipient's email address.
    subject is the subject of the mail.
    body is the content of the mail.
    ";
    const PROMPT: &str = "use `sendmail` to send a mail";

    async fn call(&mut self, input: Self::Input) -> anyhow::Result<Self::Output> {
        let client_id = var("CLIENT_ID").unwrap_or("".to_owned());
        let client_secret = var("CLIENT_SECRET").unwrap_or("".to_owned());
        let refresh_token = var("REFRESH_TOKEN").unwrap_or("".to_owned());
        let access_token =
            refresh_google_access_token(&client_id, &client_secret, &refresh_token).await?;
        let api_send_url = "https://gmail.googleapis.com/gmail/v1/users/me/messages/send";
        let client = reqwest::Client::new();

        // RFC 2047 encode subject (MIME encoded-word)
        let subject_encoded = format!(
            "=?UTF-8?B?{}?=",
            base64::engine::general_purpose::STANDARD.encode(input.subject.as_bytes())
        );

        // Construct the email content
        let email_content = format!(
            "Subject: {}\r\nContent-Type: text/plain; charset=\"UTF-8\"\r\nTo: {}\r\n\r\n{}",
            subject_encoded, input.to, input.body
        );
        let encoded_email = general_purpose::STANDARD.encode(email_content);

        let body = serde_json::json!({
            "raw": encoded_email,
        });

        let response = client
            .post(api_send_url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(ACCEPT, "application/json")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok("Mail sent successfully.".to_string())
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!(format!(
                "Failed to send mail. Status: {}, Error: {}",
                status, error_text
            )))
        }
    }
}

impl Tool for GetMailContent {
    type Input = GetMailContentInput;
    type Output = String;

    const NAME: &str = "getmailcontent";
    const DESCRIPTION: &str = "get the full content of a mail using the mail_id obtained from recentmail tool.
    mail_id is the mail_id of the mail to get content. This id should be first obtained from recentmail tool.
    ";
    const PROMPT: &str = "use `getmailcontent` to get the full content of a mail";

    async fn call(&mut self, input: Self::Input) -> anyhow::Result<Self::Output> {
        let client_id = var("CLIENT_ID").unwrap_or("".to_owned());
        let client_secret = var("CLIENT_SECRET").unwrap_or("".to_owned());
        let refresh_token = var("REFRESH_TOKEN").unwrap_or("".to_owned());
        let access_token =
            refresh_google_access_token(&client_id, &client_secret, &refresh_token).await?;
        let api_get_url = format!(
            "https://gmail.googleapis.com/gmail/v1/users/me/messages/{}",
            input.mail_id
        );
        let client = reqwest::Client::new();

        let message_response = client
            .get(&api_get_url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(ACCEPT, "application/json")
            .send()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        if !message_response.status().is_success() {
            return Err(anyhow::anyhow!(format!(
                "Unable to get the mail with the ID: {}，Error: {}",
                input.mail_id,
                message_response.status()
            )));
        }
        let message_full: Value = message_response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let payload = &message_full["payload"];
        // Get headers
        let empty_vec = Vec::new();
        let headers_data = payload["headers"].as_array().unwrap_or(&empty_vec);
        let subject = headers_data
            .iter()
            .find(|h| h["name"] == "Subject")
            .and_then(|h| h["value"].as_str())
            .unwrap_or("No Title");
        let sender = headers_data
            .iter()
            .find(|h| h["name"] == "From")
            .and_then(|h| h["value"].as_str())
            .unwrap_or("Unknown Sender");
        let date = headers_data
            .iter()
            .find(|h| h["name"] == "Date")
            .and_then(|h| h["value"].as_str())
            .unwrap_or("Unknown Date");
        // Get body
        let body_content = if let Some(parts) = payload.get("parts").and_then(|p| p.as_array()) {
            let mut found = None;
            for part in parts {
                if part["mimeType"] == "text/plain" {
                    if let Some(data) = part["body"].get("data").and_then(|d| d.as_str()) {
                        found = Some(data);
                        break;
                    }
                }
            }
            found
        } else if let Some(data) = payload
            .get("body")
            .and_then(|b| b.get("data"))
            .and_then(|d| d.as_str())
        {
            Some(data)
        } else {
            None
        };
        let body_text = if let Some(data) = body_content {
            match general_purpose::URL_SAFE.decode(data) {
                Ok(decoded) => String::from_utf8_lossy(&decoded).to_string(),
                Err(_) => "Unable to parse the content.".to_string(),
            }
        } else {
            "Unable to parse the content.".to_string()
        };
        let mut result = String::new();
        result.push_str(&format!("ID: {}\n", input.mail_id));
        result.push_str(&format!("Sender: {}\n", sender));
        result.push_str(&format!("Date: {}\n", date));
        result.push_str(&format!("Title: {}\n", subject));
        result.push_str("Content:\n");
        result.push_str(&format!("{}\n", body_text));
        Ok(result)
    }
}
