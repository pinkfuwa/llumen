use std::time::Duration;

use super::listing::{VideoModelCaps, VideoModelListing};
use super::message::File;
use super::raw;
use super::Error;
use stream_json::{Base64EmbedURL, IntoSerializer};

#[derive(Clone)]
pub struct VideoGenerationOption {
    pub duration: Option<u32>,
    pub resolution: Option<String>,
    pub aspect_ratio: Option<String>,
    pub size: Option<String>,
    pub generate_audio: Option<bool>,
    pub poll_interval: Duration,
    pub max_poll_attempts: usize,
}

impl Default for VideoGenerationOption {
    fn default() -> Self {
        Self {
            duration: None,
            resolution: None,
            aspect_ratio: None,
            size: None,
            generate_audio: None,
            poll_interval: Duration::from_secs(30),
            max_poll_attempts: 120,
        }
    }
}

pub struct VideoModelCapability {
    pub supported_resolutions: Vec<String>,
    pub supported_aspect_ratios: Vec<String>,
    pub supported_sizes: Vec<String>,
    pub allowed_passthrough_parameters: Vec<String>,
    pub max_reference_images: usize,
    pub max_reference_videos: usize,
    pub supports_generate_audio: bool,
}

impl From<VideoModelCaps> for VideoModelCapability {
    fn from(value: VideoModelCaps) -> Self {
        Self {
            supported_resolutions: value.supported_resolutions,
            supported_aspect_ratios: value.supported_aspect_ratios,
            supported_sizes: value.supported_sizes,
            allowed_passthrough_parameters: value.allowed_passthrough_parameters,
            max_reference_images: value.max_reference_images,
            max_reference_videos: value.max_reference_videos,
            supports_generate_audio: value.supports_generate_audio,
        }
    }
}

pub struct GeneratedVideo {
    pub source_url: String,
    pub filename: String,
    pub content_length: Option<u64>,
    pub mime_type: Option<String>,
    response: reqwest::Response,
}

impl GeneratedVideo {
    pub async fn next_chunk(&mut self) -> Result<Option<bytes::Bytes>, Error> {
        self.response.chunk().await.map_err(Error::Http)
    }
}

pub struct VideoGenOutput {
    pub job_id: String,
    pub videos: Vec<GeneratedVideo>,
    pub price: f64,
}

#[derive(Clone)]
pub(super) struct VideoGenClient {
    api_key: String,
    videos_endpoint: String,
    http_client: reqwest::Client,
}

impl VideoGenClient {
    pub fn new(api_key: String, videos_endpoint: String, http_client: reqwest::Client) -> Self {
        Self {
            api_key,
            videos_endpoint,
            http_client,
        }
    }

    fn generation_body(req: raw::VideoGenerationReq) -> (Option<usize>, reqwest::Body) {
        let size = req.size();
        let body = reqwest::Body::wrap_stream(req.into_stream());
        (size, body)
    }

    async fn parse_status_error(response: reqwest::Response) -> Error {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if let Ok(error) = serde_json::from_str::<raw::ErrorResp>(&body) {
            return Error::Api {
                message: error.error.message,
                code: error.error.code.or(Some(status.as_u16() as i32)),
            };
        }

        Error::Api {
            message: format!("video api status {status}: {body}"),
            code: Some(status.as_u16() as i32),
        }
    }

    fn detect_image_mime_type(data: &[u8]) -> Option<&'static str> {
        if infer::image::is_png(data) {
            Some("image/png")
        } else if infer::image::is_jpeg(data) {
            Some("image/jpeg")
        } else if infer::image::is_gif(data) {
            Some("image/gif")
        } else if infer::image::is_webp(data) {
            Some("image/webp")
        } else if infer::image::is_bmp(data) {
            Some("image/bmp")
        } else if infer::image::is_ico(data) {
            Some("image/x-icon")
        } else {
            None
        }
    }

    fn detect_video_mime_type(data: &[u8]) -> Option<&'static str> {
        if data.len() >= 12 && data[4..8] == *b"ftyp" {
            let brand = &data[8..12];
            if brand.starts_with(b"qt") {
                Some("video/quicktime")
            } else {
                Some("video/mp4")
            }
        } else if data.starts_with(&[0x00, 0x00, 0x01, 0xBA])
            || data.starts_with(&[0x00, 0x00, 0x01, 0xB3])
            || data.starts_with(&[0x00, 0x00, 0x01, 0xB4])
        {
            Some("video/mpeg")
        } else if data.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
            Some("video/webm")
        } else {
            None
        }
    }

    fn map_reference_file(file: File) -> Result<raw::VideoInputReference, Error> {
        let File { name: _, data } = file;
        let data_len = data.len();

        if let Some(mime_type) = Self::detect_image_mime_type(data.as_ref()) {
            let encoded = Base64EmbedURL::new(data, data_len, mime_type.to_string())
                .map_err(|_| Error::Incompatible("Failed to encode image reference"))?;
            return Ok(raw::VideoInputReference::image_data(encoded));
        }

        if let Some(mime_type) = Self::detect_video_mime_type(data.as_ref()) {
            let encoded = Base64EmbedURL::new(data, data_len, mime_type.to_string())
                .map_err(|_| Error::Incompatible("Failed to encode video reference"))?;
            return Ok(raw::VideoInputReference::video_data(encoded));
        }

        Err(Error::VideoGenInvalidReferenceFile)
    }

    fn split_model_id(model_id: &str) -> &str {
        model_id.split(':').next().unwrap_or(model_id)
    }

    fn extract_video_error(value: &serde_json::Value) -> String {
        if let Some(message) = value.get("message").and_then(serde_json::Value::as_str) {
            return message.to_string();
        }
        if let Some(text) = value.as_str() {
            return text.to_string();
        }
        value.to_string()
    }

    async fn submit_generation_request(
        &self,
        request: raw::VideoGenerationReq,
    ) -> Result<raw::VideoGenerationSubmitResponse, Error> {
        let (content_length, body) = Self::generation_body(request);
        let mut request_builder = self
            .http_client
            .post(&self.videos_endpoint)
            .bearer_auth(&self.api_key)
            .header("HTTP-Referer", super::HTTP_REFERER)
            .header("X-Title", super::X_TITLE)
            .header(http::header::CONTENT_TYPE, "application/json");

        if let Some(len) = content_length {
            request_builder = request_builder.header(http::header::CONTENT_LENGTH, len);
        }

        let response = request_builder
            .body(body)
            .send()
            .await
            .map_err(Error::Http)?;
        if !response.status().is_success() {
            return Err(Self::parse_status_error(response).await);
        }

        response
            .json::<raw::VideoGenerationSubmitResponse>()
            .await
            .map_err(Error::Http)
    }

    async fn poll_until_done(
        &self,
        polling_url: &str,
        poll_interval: Duration,
        max_poll_attempts: usize,
    ) -> Result<raw::VideoGenerationStatusResponse, Error> {
        for attempt in 0..max_poll_attempts {
            if attempt > 0 {
                tokio::time::sleep(poll_interval).await;
            }

            let response = self
                .http_client
                .get(polling_url)
                .bearer_auth(&self.api_key)
                .header("HTTP-Referer", super::HTTP_REFERER)
                .header("X-Title", super::X_TITLE)
                .send()
                .await
                .map_err(Error::Http)?;

            if !response.status().is_success() {
                return Err(Self::parse_status_error(response).await);
            }

            let status_response = response
                .json::<raw::VideoGenerationStatusResponse>()
                .await
                .map_err(Error::Http)?;

            match status_response.status {
                raw::VideoJobStatus::Pending | raw::VideoJobStatus::InProgress => continue,
                raw::VideoJobStatus::Completed => return Ok(status_response),
                raw::VideoJobStatus::Failed => {
                    let error_message = status_response
                        .error
                        .as_ref()
                        .map(Self::extract_video_error)
                        .unwrap_or_else(|| "unknown error".to_string());
                    return Err(Error::VideoGenJobFailed(error_message));
                }
                raw::VideoJobStatus::Unknown => {
                    return Err(Error::MalformedResponse(
                        "Unknown video generation job status",
                    ));
                }
            }
        }

        Err(Error::VideoGenPollingTimeout)
    }

    async fn open_download_streams(
        &self,
        job_id: &str,
        video_count: usize,
    ) -> Result<Vec<GeneratedVideo>, Error> {
        let mut generated_videos = Vec::with_capacity(video_count);

        for index in 0..video_count {
            let url = format!(
                "{}/{job_id}/content?index={index}",
                self.videos_endpoint.trim_end_matches('/')
            );
            let response = self
                .http_client
                .get(&url)
                .bearer_auth(&self.api_key)
                .header("HTTP-Referer", super::HTTP_REFERER)
                .header("X-Title", super::X_TITLE)
                .send()
                .await
                .map_err(Error::Http)?;

            if !response.status().is_success() {
                return Err(Self::parse_status_error(response).await);
            }

            let content_length = response.content_length();
            let mime_type = response
                .headers()
                .get(http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .map(ToString::to_string);

            generated_videos.push(GeneratedVideo {
                source_url: url,
                filename: format!("generated-video-{index}.mp4"),
                content_length,
                mime_type,
                response,
            });
        }

        Ok(generated_videos)
    }

    pub async fn video_generate(
        &self,
        listing: &VideoModelListing,
        model_id: String,
        prompt: String,
        references: Vec<File>,
        option: VideoGenerationOption,
    ) -> Result<VideoGenOutput, Error> {
        let model_id = Self::split_model_id(&model_id).to_string();
        let capability = listing
            .get(&model_id)
            .await
            .ok_or(Error::VideoGenModelNotFound)?;

        let mut image_reference_count = 0;
        let mut video_reference_count = 0;
        let mut input_references = Vec::new();

        for file in references {
            let mapped = Self::map_reference_file(file)?;
            match mapped.r#type {
                raw::VideoInputReferenceType::ImageUrl => image_reference_count += 1,
                raw::VideoInputReferenceType::VideoUrl => video_reference_count += 1,
            }
            input_references.push(mapped);
        }

        if image_reference_count > 0 && !capability.supports_reference_images() {
            return Err(Error::VideoGenReferenceImagesNotSupported);
        }
        if video_reference_count > 0 && !capability.supports_reference_videos() {
            return Err(Error::VideoGenReferenceVideosNotSupported);
        }

        if image_reference_count > capability.max_reference_images {
            return Err(Error::VideoGenReferenceImagesLimitExceeded {
                max: capability.max_reference_images,
            });
        }
        if video_reference_count > capability.max_reference_videos {
            return Err(Error::VideoGenReferenceVideosLimitExceeded {
                max: capability.max_reference_videos,
            });
        }

        if let Some(generate_audio) = option.generate_audio {
            if generate_audio && !capability.supports_generate_audio {
                return Err(Error::VideoGenNotSupported);
            }
        }

        let request = raw::VideoGenerationReq {
            model: model_id,
            prompt,
            duration: option.duration,
            resolution: option.resolution,
            aspect_ratio: option.aspect_ratio,
            size: option.size,
            input_references,
            generate_audio: option.generate_audio,
        };

        let submit_response = self.submit_generation_request(request).await?;
        let job_id = submit_response.id;
        let poll_interval = option.poll_interval;
        let max_poll_attempts = option.max_poll_attempts;

        let status_response = match submit_response.status {
            raw::VideoJobStatus::Completed => {
                self.poll_until_done(&submit_response.polling_url, poll_interval, 1)
                    .await?
            }
            raw::VideoJobStatus::Failed => {
                return Err(Error::VideoGenJobFailed(
                    "job failed immediately".to_string(),
                ));
            }
            raw::VideoJobStatus::Pending
            | raw::VideoJobStatus::InProgress
            | raw::VideoJobStatus::Unknown => {
                self.poll_until_done(
                    &submit_response.polling_url,
                    poll_interval,
                    max_poll_attempts,
                )
                .await?
            }
        };

        if status_response.unsigned_urls.is_empty() {
            return Err(Error::VideoGenNoVideosInResponse);
        }

        let generated_videos = self
            .open_download_streams(&job_id, status_response.unsigned_urls.len())
            .await?;

        let price = status_response.usage.map(|usage| usage.cost).unwrap_or(0.0);

        Ok(VideoGenOutput {
            job_id,
            videos: generated_videos,
            price,
        })
    }
}
