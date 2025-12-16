use super::option::Tool;
use protocol::OcrEngine;

#[derive(Clone, Default)]
pub struct Capability {
    pub image_output: bool,
    pub image_input: bool,
    pub structured_output: bool,
    pub toolcall: bool,
    pub ocr: OcrEngine,
    pub audio: bool,
    pub reasoning: bool,
}

#[derive(Clone, Default)]
pub struct MaybeCapability {
    pub image_output: Option<bool>,
    pub image_input: Option<bool>,
    pub structured_output: Option<bool>,
    pub toolcall: Option<bool>,
    pub ocr: Option<OcrEngine>,
    pub audio: Option<bool>,
    pub reasoning: Option<bool>,
}

#[derive(Clone, Default)]
pub struct Model {
    pub id: String,
    pub temperature: Option<f32>,
    pub repeat_penalty: Option<f32>,
    pub top_k: Option<i32>,
    pub top_p: Option<f32>,
    // capabilities override
    pub capability: MaybeCapability,
}

impl Model {
    pub fn builder(id: impl Into<String>) -> ModelBuilder {
        ModelBuilder::new(id)
    }
}

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

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn repeat_penalty(mut self, repeat_penalty: f32) -> Self {
        self.repeat_penalty = Some(repeat_penalty);
        self
    }

    pub fn top_k(mut self, top_k: i32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = tools;
        self
    }

    pub fn capability(mut self, capability: MaybeCapability) -> Self {
        self.capability = capability;
        self
    }

    pub fn image_output(mut self, image_output: bool) -> Self {
        self.capability.image_output = Some(image_output);
        self
    }

    pub fn image_input(mut self, image_input: bool) -> Self {
        self.capability.image_input = Some(image_input);
        self
    }

    pub fn structured_output(mut self, structured_output: bool) -> Self {
        self.capability.structured_output = Some(structured_output);
        self
    }

    pub fn ocr(mut self, ocr: OcrEngine) -> Self {
        self.capability.ocr = Some(ocr);
        self
    }

    pub fn audio(mut self, audio: bool) -> Self {
        self.capability.audio = Some(audio);
        self
    }

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
