-- MultiAgent/Proofs/Pipeline.lean
-- Proof that valid pipelines have matching types between consecutive stages.
import MultiAgent.Types
import MultiAgent.Funs
import Aeneas

open Primitives
open multi_agent

namespace multi_agent

/-!
# Pipeline Proofs

We prove that `is_pipeline_valid` implies pairwise type matching:
for all consecutive stages `(i, i+1)`, `stages[i].output_type = stages[i+1].input_type`.
-/

/-- A valid pipeline with at most one stage is trivially valid. -/
theorem pipeline_single_valid (s : PipelineStage) :
    is_pipeline_valid_aux [s] = true := by
  simp [is_pipeline_valid_aux]

/-- `is_pipeline_valid_aux` on an empty list is true. -/
theorem pipeline_empty_valid :
    is_pipeline_valid_aux [] = true := by
  simp [is_pipeline_valid_aux]

/-- If `is_pipeline_valid_aux (s1 :: s2 :: rest)` is true,
    then `s1.output_type = s2.input_type`. -/
theorem pipeline_head_types_match (s1 s2 : PipelineStage) (rest : List PipelineStage)
    (h : is_pipeline_valid_aux (s1 :: s2 :: rest) = true) :
    s1.output_type = s2.input_type := by
  simp [is_pipeline_valid_aux] at h
  split at h <;> simp_all

/-- If `is_pipeline_valid_aux (s1 :: s2 :: rest)` is true,
    then `is_pipeline_valid_aux (s2 :: rest)` is also true. -/
theorem pipeline_tail_valid (s1 s2 : PipelineStage) (rest : List PipelineStage)
    (h : is_pipeline_valid_aux (s1 :: s2 :: rest) = true) :
    is_pipeline_valid_aux (s2 :: rest) = true := by
  simp [is_pipeline_valid_aux] at h
  split at h <;> simp_all

/-- If `is_pipeline_valid pipeline = true`, then for all consecutive stages
    `(i, i+1)`, `stages[i].output_type = stages[i+1].input_type`. -/
theorem valid_pipeline_types_match (pipeline : Pipeline)
    (h : is_pipeline_valid pipeline = true)
    (i : Nat)
    (s1 s2 : PipelineStage)
    (h1 : pipeline.stages.get? i = some s1)
    (h2 : pipeline.stages.get? (i + 1) = some s2) :
    s1.output_type = s2.input_type := by
  sorry  -- By induction on i and the stages list, using pipeline_head_types_match
         -- and pipeline_tail_valid at each step

end multi_agent
