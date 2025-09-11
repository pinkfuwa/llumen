use reqwest::Url;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tools::Tool;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Wttr;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WttrInput {
    /// the location to get weather info
    /// e.g. `London`, `Moscow`, `Salt+Lake+City`
    location: String,
}
impl Tool for Wttr {
    type Input = WttrInput;
    type Output = String;

    const NAME: &str = "wttr";
    const DESCRIPTION: &str = "get weather info such as humidity, wind speed, temperature, etc from wttr.in in json format";
    const PROMPT: &str = "use `wttr` to get weather info whem user request";

    async fn call(&mut self, input: Self::Input) -> anyhow::Result<Self::Output> {
        let url: Url = "https://wttr.in/".parse()?;
        let url = url.join(input.location.trim().replace(" ", "+").as_str())?;
        let resp = reqwest::get(url).await?.text().await?;
        Ok(resp)
    }
}
