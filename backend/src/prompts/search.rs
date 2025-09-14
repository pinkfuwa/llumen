use crate::prompts::{PromptStore, PromptTemplate};

pub struct SearchStore;

impl PromptStore for SearchStore {
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
                "/../prompts/search/zh-tw.md"
            )),
            Some("en") | _ => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../prompts/search/en.md"
            )),
        };

        Ok(PromptTemplate::new(template))
    }
}
