//! High-level model descriptor and capability overrides.

use super::option::Tool;
use protocol::{OcrEngine, ReasoningEffort, ReasoningOption};

/// Describes a model's capabilities — what it can do.
#[derive(Clone, Default)]
pub struct Capability {
    pub text_output: bool,
    pub image_output: bool,
    pub image_input: bool,
    pub video_input: bool,
    pub structured_output: bool,
    pub toolcall: bool,
    pub ocr: OcrEngine,
    pub audio: bool,
    pub reasoning: bool,
    pub reasoning_effort: ReasoningEffort,
}

/// Capability overrides where each field is optional.
/// `None` leaves the model's advertised capability unchanged.
#[derive(Clone, Default)]
pub struct MaybeCapability {
    pub text_output: Option<bool>,
    pub image_output: Option<bool>,
    pub image_input: Option<bool>,
    pub video_input: Option<bool>,
    pub structured_output: Option<bool>,
    pub toolcall: Option<bool>,
    pub ocr: Option<OcrEngine>,
    pub audio: Option<bool>,
    pub reasoning: Option<bool>,
    pub reasoning_effort: Option<ReasoningEffort>,
}

impl From<ReasoningOption> for MaybeCapability {
    fn from(value: ReasoningOption) -> Self {
        MaybeCapability {
            reasoning: Some(value.is_enabled()),
            reasoning_effort: Some(value.effort()),
            ..Default::default()
        }
    }
}

/// A resolved model descriptor with an identifier and optional parameter
/// overrides.
#[derive(Clone, Default)]
pub struct Model {
    pub id: String,
    pub temperature: Option<f32>,
    pub repeat_penalty: Option<f32>,
    pub top_k: Option<i32>,
    pub top_p: Option<f32>,
    // capabilities **override**
    pub capability: MaybeCapability,
}

impl Model {
    /// Create a [`ModelBuilder`] with the given model identifier.
    pub fn builder(id: impl Into<String>) -> ModelBuilder {
        ModelBuilder::new(id)
    }
}

/// Builder for constructing a [`Model`] with optional parameter overrides.
pub struct ModelBuilder {
    id: String,
    temperature: Option<f32>,
    repeat_penalty: Option<f32>,
    top_k: Option<i32>,
    top_p: Option<f32>,
    tools: Vec<Tool>,
    capability: MaybeCapability,
}

impl ModelBuilder {
    /// Create a new `ModelBuilder` with the given model identifier and all
    /// fields defaulted.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            temperature: None,
            repeat_penalty: None,
            top_k: None,
            top_p: None,
            tools: Vec::new(),
            capability: MaybeCapability::default(),
        }
    }

    /// Pre-populate the builder from an existing [`Model`].
    pub fn from_model(model: &Model) -> Self {
        Self {
            id: model.id.clone(),
            temperature: model.temperature,
            repeat_penalty: model.repeat_penalty,
            top_k: model.top_k,
            top_p: model.top_p,
            tools: Vec::new(),
            capability: model.capability.clone(),
        }
    }

    /// Set the sampling temperature.
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the repetition penalty.
    pub fn repeat_penalty(mut self, repeat_penalty: f32) -> Self {
        self.repeat_penalty = Some(repeat_penalty);
        self
    }

    /// Set the top-k sampling parameter.
    pub fn top_k(mut self, top_k: i32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Set the top-p (nucleus) sampling parameter.
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Replace the list of tools with a new set.
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = tools;
        self
    }

    /// Override the full capability set at once.
    pub fn capability(mut self, capability: MaybeCapability) -> Self {
        self.capability = capability;
        self
    }

    /// Override the image-output capability.
    pub fn image_output(mut self, image_output: bool) -> Self {
        self.capability.image_output = Some(image_output);
        self
    }

    /// Override the image-input capability.
    pub fn image_input(mut self, image_input: bool) -> Self {
        self.capability.image_input = Some(image_input);
        self
    }

    /// Override the video-input capability.
    pub fn video_input(mut self, video_input: bool) -> Self {
        self.capability.video_input = Some(video_input);
        self
    }

    /// Override the structured-output capability.
    pub fn structured_output(mut self, structured_output: bool) -> Self {
        self.capability.structured_output = Some(structured_output);
        self
    }

    /// Override the OCR engine capability.
    pub fn ocr(mut self, ocr: OcrEngine) -> Self {
        self.capability.ocr = Some(ocr);
        self
    }

    /// Override the audio capability.
    pub fn audio(mut self, audio: bool) -> Self {
        self.capability.audio = Some(audio);
        self
    }

    /// Override the text-output capability.
    pub fn text_output(mut self, text_output: bool) -> Self {
        self.capability.text_output = Some(text_output);
        self
    }

    /// Consume the builder and produce a [`Model`].
    pub fn build(self) -> Model {
        Model {
            id: self.id,
            temperature: self.temperature,
            repeat_penalty: self.repeat_penalty,
            top_k: self.top_k,
            top_p: self.top_p,
            capability: self.capability,
        }
    }
}
