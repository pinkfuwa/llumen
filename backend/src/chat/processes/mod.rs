use super::process::chat::ChatPipeline;

mod normal;
mod search;
pub mod deep;

pub type Normal = ChatPipeline<normal::Inner>;
pub type Search = ChatPipeline<search::Inner>;
pub use deep::DeepPipeline;
