use super::process::chat::ChatPipeline;

mod normal;

pub type NormalPipeline = ChatPipeline<normal::NormalPipelineInner>;
