use super::raw;

#[derive(Clone, Default)]
pub struct Model {
    pub id: String,
    pub temperature: Option<f32>,
    pub repeat_penalty: Option<f32>,
    pub top_k: Option<i32>,
    pub top_p: Option<f32>,
    pub online: bool,
    pub response_format: Option<raw::ResponseFormat>,
}

impl Model {
    pub fn get_model_id(&self) -> String {
        let mut id = self.id.clone();
        if self.online {
            id.push_str(":online");
        }
        id
    }

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
    online: bool,
    response_format: Option<raw::ResponseFormat>,
}

impl ModelBuilder {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            temperature: None,
            repeat_penalty: None,
            top_k: None,
            top_p: None,
            online: false,
            response_format: None,
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

    pub fn online(mut self, online: bool) -> Self {
        self.online = online;
        self
    }

    pub fn response_format(mut self, response_format: raw::ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }

    pub fn json_schema(mut self, name: impl Into<String>, schema: serde_json::Value) -> Self {
        self.response_format = Some(raw::ResponseFormat {
            r#type: "json_schema".to_string(),
            json_schema: serde_json::json!({
                "name": name.into(),
                "strict": true,
                "schema": schema
            }),
        });
        self
    }

    pub fn build(self) -> Model {
        Model {
            id: self.id,
            temperature: self.temperature,
            repeat_penalty: self.repeat_penalty,
            top_k: self.top_k,
            top_p: self.top_p,
            online: self.online,
            response_format: self.response_format,
        }
    }
}
