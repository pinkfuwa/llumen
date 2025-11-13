use super::agent::chat::ChatPipeline;

mod deep;
mod normal;
mod search;

pub type Normal = ChatPipeline<normal::Inner>;
pub type Search = ChatPipeline<search::Inner>;
pub type Deep = ChatPipeline<deep::Inner>;
