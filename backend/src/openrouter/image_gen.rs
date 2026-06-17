//! Image generation via OpenRouter or native `/v1/images/generations`.

use super::listing::ModelListing;
use super::message::{File, GeneratedImage, Message};
use super::raw;
use super::Error;
use stream_json::IntoSerializer;

/// Supported image aspect ratios as width:height dimension pairs.
#[derive(Debug, Clone, Copy)]
pub enum AspectRatio {
    R1x1,
    R2x3,
    R3x2,
    R3x4,
    R4x3,
    R4x5,
    R5x4,
    R9x16,
    R16x9,
    R21x9,
}

impl AspectRatio {
    fn as_str(self) -> &'static str {
        match self {
            AspectRatio::R1x1 => "1:1",
            AspectRatio::R2x3 => "2:3",
            AspectRatio::R3x2 => "3:2",
            AspectRatio::R3x4 => "3:4",
            AspectRatio::R4x3 => "4:3",
            AspectRatio::R4x5 => "4:5",
            AspectRatio::R5x4 => "5:4",
            AspectRatio::R9x16 => "9:16",
            AspectRatio::R16x9 => "16:9",
            AspectRatio::R21x9 => "21:9",
        }
    }
    fn to_size(self) -> Option<&'static str> {
        match self {
            AspectRatio::R1x1 => None,
            AspectRatio::R9x16 => Some("1024x1792"),
            AspectRatio::R16x9 => Some("1792x1024"),
            _ => None,
        }
    }
}

/// Result of an image generation request.
pub struct ImageGenOutput {
    pub images: Vec<GeneratedImage>,
    pub text: Option<String>,
    pub price: f64,
    pub token: usize,
}

#[derive(Clone)]
pub enum ImageGenEndpoint {
    /// endpoint used by openrouter
    ChatCompletion(String),
    ImageGeneration(String),
}

impl ImageGenEndpoint {
    pub fn from_base_url(base_url: &str, is_custom_api: bool) -> Self {
        if is_custom_api {
            ImageGenEndpoint::ImageGeneration(format!(
                "{}/v1/images/generations",
                base_url.trim_end_matches('/')
            ))
        } else {
            ImageGenEndpoint::ChatCompletion(format!(
                "{}/v1/chat/completions",
                base_url.trim_end_matches('/')
            ))
        }
    }
}

#[derive(Clone)]
pub(super) struct ImageGenClient {
    api_key: String,
    endpoint: ImageGenEndpoint,
    http_client: reqwest::Client,
}

impl ImageGenClient {
    pub fn new(api_key: String, endpoint: ImageGenEndpoint, http_client: reqwest::Client) -> Self {
        Self {
            api_key,
            endpoint,
            http_client,
        }
    }

    fn completion_body(req: raw::CompletionReq) -> (Option<usize>, reqwest::Body) {
        let size = req.size();
        let body = reqwest::Body::wrap_stream(req.into_stream());
        (size, body)
    }

    fn build_message(prompt: String, reference_images: Vec<File>) -> Message {
        if reference_images.is_empty() {
            Message::User(prompt)
        } else {
            Message::MultipartUser {
                text: prompt,
                files: reference_images,
            }
        }
    }

    pub(super) async fn send_image_gen_request(
        &self,
        model_id: String,
        prompt: String,
        aspect_ratio: AspectRatio,
        url: &str,
    ) -> Result<ImageGenOutput, Error> {
        let req = raw::ImageGenApiReq {
            model: model_id,
            prompt,
            n: Some(1),
            size: aspect_ratio.to_size().map(str::to_string),
            response_format: Some("b64_json".to_string()),
        };

        let body = serde_json::to_vec(&req)?;

        let res = self
            .http_client
            .post(url)
            .bearer_auth(&self.api_key)
            .headers(super::OPENROUTER_HEADERS.clone())
            .header(http::header::CONTENT_TYPE, "application/json")
            .header(http::header::CONTENT_LENGTH, body.len())
            .body(body)
            .send()
            .await?;

        let json: raw::ImageGenApiResponse = res.json().await?;

        if let Some(error) = json.error {
            return Err(Error::from(error));
        }

        let images = json
            .data
            .into_iter()
            .filter_map(|item| item.b64_json)
            .map(|b64| GeneratedImage::from_b64_json(b64, "image/png"))
            .collect::<Result<Vec<_>, _>>()?;

        if images.is_empty() {
            return Err(Error::ImageGenNoImagesInResponse);
        }

        Ok(ImageGenOutput {
            images,
            text: None,
            price: 0.0,
            token: 0,
        })
    }

    async fn send_complete_request(
        &self,
        listing: &ModelListing,
        model_id: String,
        prompt: String,
        reference_images: Vec<File>,
        aspect_ratio: AspectRatio,
        url: &str,
    ) -> Result<ImageGenOutput, Error> {
        let model_id = model_id.split(':').next().unwrap_or(&model_id).to_string();

        let capability = listing
            .get(&model_id)
            .await
            .map(super::Capability::from)
            .ok_or(Error::ImageGenModelNotFound)?;

        if capability.text_output {
            return Err(Error::ImageGenNotSupported);
        }

        if !reference_images.is_empty() && !capability.image_input {
            return Err(Error::ImageGenReferenceImagesNotSupported);
        }

        let message = Self::build_message(prompt, reference_images);
        let raw_message = message.to_raw_message(&model_id, &capability);

        let request = raw::CompletionReq {
            model: model_id,
            messages: vec![raw_message],
            stream: false,
            temperature: None,
            repeat_penalty: None,
            top_k: None,
            top_p: None,
            max_tokens: None,
            tools: Vec::new(),
            plugins: Vec::new(),
            usage: Some(raw::UsageReq { include: true }),
            response_format: None,
            reasoning: raw::Reasoning::default(),
            modalities: vec!["image".to_string()],
            session_id: None,
            image_config: Some(serde_json::json!({
                "aspect_ratio": aspect_ratio.as_str(),
            })),
        };

        let (content_length, body) = Self::completion_body(request);
        let mut req_builder = self
            .http_client
            .post(url)
            .bearer_auth(&self.api_key)
            .headers(super::OPENROUTER_HEADERS.clone())
            .header(http::header::CONTENT_TYPE, "application/json");

        if let Some(len) = content_length {
            req_builder = req_builder.header(http::header::CONTENT_LENGTH, len);
        }

        let res = req_builder.body(body).send().await.map_err(Error::Http)?;
        let json = res
            .json::<raw::CompletionResponse>()
            .await
            .map_err(Error::Http)?;

        if let Some(error) = json.error {
            return Err(Error::from(error));
        }

        let (token, price) = json
            .usage
            .map(|usage| {
                (
                    usage.total_tokens,
                    usage
                        .cost_details
                        .and_then(|details| details.upstream_inference_cost)
                        .unwrap_or(usage.cost),
                )
            })
            .unwrap_or_default();

        let choice =
            json.choices
                .unwrap_or_default()
                .into_iter()
                .next()
                .ok_or(Error::MalformedResponse(
                    "No choices in completion response",
                ))?;

        let images = choice
            .message
            .images
            .into_iter()
            .map(GeneratedImage::from_raw_image)
            .collect::<Result<Vec<_>, _>>()?;

        if images.is_empty() {
            return Err(Error::ImageGenNoImagesInResponse);
        }

        Ok(ImageGenOutput {
            images,
            text: choice.message.content,
            price,
            token: token.unwrap_or_default() as usize,
        })
    }

    /// Generate a image
    ///
    /// when openrouter router is used, request route to chat completions with
    /// modality. otherwise, request route to the OpenAI-compatible image
    /// endpoint directly.
    pub async fn generate(
        &self,
        listing: &ModelListing,
        model_id: String,
        prompt: String,
        reference_images: Vec<File>,
        aspect_ratio: AspectRatio,
    ) -> Result<ImageGenOutput, Error> {
        match &self.endpoint {
            ImageGenEndpoint::ChatCompletion(url) => {
                self.send_complete_request(
                    listing,
                    model_id,
                    prompt,
                    reference_images,
                    aspect_ratio,
                    url,
                )
                .await
            }
            ImageGenEndpoint::ImageGeneration(url) => {
                self.send_image_gen_request(model_id, prompt, aspect_ratio, url)
                    .await
            }
        }
    }
}
