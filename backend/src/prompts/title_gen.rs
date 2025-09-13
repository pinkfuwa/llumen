use crate::prompts::{PromptStore, PromptTemplate};

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
