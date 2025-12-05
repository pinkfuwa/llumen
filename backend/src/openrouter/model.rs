use super::{option::Tool, raw};

#[derive(Clone, Default, Copy)]
pub struct Capabilities {
    pub image: bool,
    pub structured_output: bool,
}

#[derive(Clone, Default)]
pub struct Model {
    pub id: String,
    pub temperature: Option<f32>,
    pub repeat_penalty: Option<f32>,
    pub top_k: Option<i32>,
    pub top_p: Option<f32>,
    pub response_format: Option<raw::ResponseFormat>,
    pub capabilities: Capabilities,
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
    response_format: Option<raw::ResponseFormat>,
    tools: Vec<Tool>,
    capabilities: Capabilities,
}

impl ModelBuilder {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            temperature: None,
            repeat_penalty: None,
            top_k: None,
            top_p: None,
            response_format: None,
            tools: Vec::new(),
            capabilities: Capabilities::default(),
        }
    }

    pub fn from_model(model: &Model) -> Self {
        Self {
            id: model.id.clone(),
            temperature: model.temperature,
            repeat_penalty: model.repeat_penalty,
            top_k: model.top_k,
            top_p: model.top_p,
            response_format: model.response_format.clone(),
            tools: Vec::new(),
            capabilities: model.capabilities,
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

    pub fn response_format(mut self, response_format: raw::ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }

    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = tools;
        self
    }

    pub fn capabilities(mut self, capabilities: Capabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn support_image(mut self) -> Self {
        self.capabilities.image = true;
        self
    }

    pub fn support_structured_output(mut self) -> Self {
        self.capabilities.structured_output = true;
        self
    }

    pub fn build(self) -> Model {
        Model {
            id: self.id,
            temperature: self.temperature,
            repeat_penalty: self.repeat_penalty,
            top_k: self.top_k,
            top_p: self.top_p,
            response_format: self.response_format,
            capabilities: self.capabilities,
        }
    }
}
