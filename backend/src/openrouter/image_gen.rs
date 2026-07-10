//! Image generation via OpenRouter `/v1/images` or OpenAI-compatible
//! `/v1/images/generations`.

use super::listing::ModelListing;
use super::message::{File, GeneratedImage};
use super::raw;
use super::Error;
use stream_json::{Base64EmbedURL, IntoSerializer};

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
    /// OpenRouter dedicated image endpoint (`/v1/images`).
    OpenrouterImage(String),
    /// OpenAI-compatible image generation endpoint (`/v1/images/generations`).
    ImageGeneration(String),
}

impl ImageGenEndpoint {
    pub fn from_base_url(base_url: &str, is_custom_api: bool) -> Self {
        let base = base_url.trim_end_matches('/');
        if is_custom_api {
            ImageGenEndpoint::ImageGeneration(format!("{base}/v1/images/generations"))
        } else {
            ImageGenEndpoint::OpenrouterImage(format!("{base}/v1/images"))
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

    fn completion_body(req: raw::OpenrouterImageGenReq) -> (Option<usize>, reqwest::Body) {
        let size = req.size();
        let body = reqwest::Body::wrap_stream(req.into_stream());
        (size, body)
    }

    fn map_reference_image(file: File) -> Result<raw::OpenrouterImageInputRef, Error> {
        let File {
            data,
            mime_type,
            name: _,
        } = file;
        let data_len = data.len();

        let mime = mime_type
            .as_deref()
            .and_then(|m| {
                let lower = m.to_ascii_lowercase();
                if lower.starts_with("image/") {
                    Some(lower)
                } else {
                    None
                }
            })
            .or_else(|| {
                super::raw::detect_image_format(data.as_ref()).map(|f| format!("image/{f}"))
            })
            .unwrap_or_else(|| "image/png".to_string());

        let embed_file = Base64EmbedURL::new(data, data_len, mime).map_err(|_| {
            Error::Incompatible("Failed to encode image reference for image generation")
        })?;

        Ok(raw::OpenrouterImageInputRef {
            image_url: raw::OpenrouterImageInputUrl { url: embed_file },
        })
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

    async fn send_openrouter_image_request(
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

        let input_references = reference_images
            .into_iter()
            .map(Self::map_reference_image)
            .collect::<Result<Vec<_>, Error>>()?;

        let request = raw::OpenrouterImageGenReq {
            model: model_id,
            prompt,
            aspect_ratio: Some(aspect_ratio.as_str().to_string()),
            input_references,
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
        let json: raw::OpenrouterImageGenResponse = res.json().await.map_err(Error::Http)?;

        if let Some(error) = json.error {
            return Err(Error::from(error));
        }

        let (price, token) = json
            .usage
            .map(|usage| {
                let cost = usage
                    .cost_details
                    .and_then(|details| details.upstream_inference_cost)
                    .unwrap_or(usage.cost.unwrap_or(0.0));
                (cost, usage.total_tokens as usize)
            })
            .unwrap_or_default();

        let images = json
            .data
            .into_iter()
            .map(|item| {
                let mime_type = item.media_type.unwrap_or("image/png".to_string());
                GeneratedImage::from_b64_json(item.b64_json, mime_type)
            })
            .collect::<Result<Vec<_>, Error>>()?;

        if images.is_empty() {
            return Err(Error::ImageGenNoImagesInResponse);
        }

        Ok(ImageGenOutput {
            images,
            text: None,
            price,
            token,
        })
    }

    /// Generate an image.
    ///
    /// When using OpenRouter, requests are sent to the dedicated `/v1/images`
    /// endpoint. For custom (OpenAI-compatible) APIs, requests go to
    /// `/v1/images/generations`.
    pub async fn generate(
        &self,
        listing: &ModelListing,
        model_id: String,
        prompt: String,
        reference_images: Vec<File>,
        aspect_ratio: AspectRatio,
    ) -> Result<ImageGenOutput, Error> {
        match &self.endpoint {
            ImageGenEndpoint::OpenrouterImage(url) => {
                self.send_openrouter_image_request(
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
