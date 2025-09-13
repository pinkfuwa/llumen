use crate::prompts::{PromptStore, PromptTemplate};

pub struct ChatStore;

impl PromptStore for ChatStore {
    type Source = &'static str;
    type Extra = ();
    type Pipe = ();

    async fn template(
        &self,
        locale: Option<&str>,
    ) -> anyhow::Result<super::PromptTemplate<Self::Source, Self::Extra, Self::Pipe>> {
        let template = match locale {
            Some("zh-tw") => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../prompts/normal/zh-tw.md"
            )),
            Some("en") | _ => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../prompts/normal/en.md"
            )),
        };

        Ok(PromptTemplate::new(template))
    }
}

pub struct TitleGenStore;

impl PromptStore for TitleGenStore {
    type Source = &'static str;
    type Extra = ();
    type Pipe = ();

    async fn template(
        &self,
        locale: Option<&str>,
    ) -> anyhow::Result<super::PromptTemplate<Self::Source, Self::Extra, Self::Pipe>> {
        let template = match locale {
            Some("zh-tw") => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../prompts/title_gen/zh-tw.md"
            )),
            Some("en") | _ => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../prompts/title_gen/en.md"
            )),
        };

        Ok(PromptTemplate::new(template))
    }
}
