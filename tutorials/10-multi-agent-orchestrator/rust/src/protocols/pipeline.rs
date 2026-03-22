use crate::agent_trait::AgentId;

/// A single stage in a processing pipeline.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PipelineStage {
    pub agent_id: AgentId,
    pub input_type: u32,
    pub output_type: u32,
}

/// A multi-stage processing pipeline.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Pipeline {
    pub stages: Vec<PipelineStage>,
}

/// Create a new empty pipeline.
pub fn pipeline_new() -> Pipeline {
    Pipeline { stages: Vec::new() }
}

/// Add a stage to the pipeline.
pub fn pipeline_add_stage(
    pipeline: &mut Pipeline,
    agent_id: AgentId,
    input_type: u32,
    output_type: u32,
) {
    pipeline.stages.push(PipelineStage {
        agent_id,
        input_type,
        output_type,
    });
}

/// Validate that the pipeline is well-formed: each stage's output_type
/// matches the next stage's input_type.
pub fn is_pipeline_valid(pipeline: &Pipeline) -> bool {
    if pipeline.stages.len() <= 1 {
        return true;
    }
    for i in 0..pipeline.stages.len() - 1 {
        if pipeline.stages[i].output_type != pipeline.stages[i + 1].input_type {
            return false;
        }
    }
    true
}

/// Return the agent responsible for the given stage index, if it exists.
pub fn pipeline_agent_at(pipeline: &Pipeline, stage_index: u32) -> Option<AgentId> {
    let idx = stage_index as usize;
    if idx < pipeline.stages.len() {
        Some(pipeline.stages[idx].agent_id)
    } else {
        None
    }
}

/// Return the number of stages in the pipeline.
pub fn pipeline_len(pipeline: &Pipeline) -> u32 {
    pipeline.stages.len() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_pipeline() {
        let mut pipeline = pipeline_new();
        pipeline_add_stage(&mut pipeline, 1, 0, 1);
        pipeline_add_stage(&mut pipeline, 2, 1, 2);
        pipeline_add_stage(&mut pipeline, 3, 2, 3);
        assert!(is_pipeline_valid(&pipeline));
    }

    #[test]
    fn test_invalid_pipeline() {
        let mut pipeline = pipeline_new();
        pipeline_add_stage(&mut pipeline, 1, 0, 1);
        pipeline_add_stage(&mut pipeline, 2, 5, 2); // mismatch: expects 1, got 5
        assert!(!is_pipeline_valid(&pipeline));
    }

    #[test]
    fn test_empty_pipeline_valid() {
        let pipeline = pipeline_new();
        assert!(is_pipeline_valid(&pipeline));
    }

    #[test]
    fn test_single_stage_valid() {
        let mut pipeline = pipeline_new();
        pipeline_add_stage(&mut pipeline, 1, 0, 1);
        assert!(is_pipeline_valid(&pipeline));
    }

    #[test]
    fn test_pipeline_agent_at() {
        let mut pipeline = pipeline_new();
        pipeline_add_stage(&mut pipeline, 10, 0, 1);
        pipeline_add_stage(&mut pipeline, 20, 1, 2);
        assert_eq!(pipeline_agent_at(&pipeline, 0), Some(10));
        assert_eq!(pipeline_agent_at(&pipeline, 1), Some(20));
        assert_eq!(pipeline_agent_at(&pipeline, 2), None);
    }
}
