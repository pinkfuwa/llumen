use super::model_cache::ModelCacheManager;
use super::{Capability, MaybeCapability, Model};

pub(super) struct CapabilityResolver<'a> {
    cache: &'a ModelCacheManager,
}

impl<'a> CapabilityResolver<'a> {
    pub fn new(cache: &'a ModelCacheManager) -> Self {
        Self { cache }
    }

    /// Get capability of a model (considers user overrides)
    pub async fn get_capability(&self, model: &Model) -> Capability {
        let model_id = model.id.split(':').next().unwrap();

        let overrides: MaybeCapability = model.capability.clone().into();
        let capability = self.get_openrouter_capability(model_id).await;

        macro_rules! merge {
            ($v:ident) => {
                match overrides.$v {
                    Some(v) => v,
                    None => capability.$v,
                }
            };
        }

        Capability {
            text_output: merge!(text_output),
            image_output: merge!(image_output),
            image_input: merge!(image_input),
            structured_output: merge!(structured_output),
            toolcall: merge!(toolcall),
            ocr: merge!(ocr),
            audio: merge!(audio),
            reasoning: merge!(reasoning),
        }
    }

    /// Get OpenRouter capabilities (no overrides)
    async fn get_openrouter_capability(&self, model_id: &str) -> Capability {
        self.cache
            .get(model_id)
            .await
            .map(|cache| cache.into())
            .unwrap_or_else(|| Capability {
                text_output: true,
                ..Default::default()
            })
    }
}
