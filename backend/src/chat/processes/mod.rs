use super::process::chat::ChatPipeline;

mod normal;
mod search;

pub type Normal = ChatPipeline<normal::Inner>;
pub type Search = ChatPipeline<search::Inner>;
