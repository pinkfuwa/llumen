use super::pipeline::chat::ChatPipeline;

mod normal;

pub type NormalPipeline = ChatPipeline<normal::NormalPipelineInner>;
