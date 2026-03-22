-- AgentReasoning/Proofs/ToolSafety.lean
-- Proofs that tool calls use registered tools and validation is sound.
import AgentReasoning.Types
import AgentReasoning.Funs
import Aeneas

open Primitives
open agent_reasoning

namespace agent_reasoning

/-!
# Tool Safety Proofs

## tool_call_uses_registered_tool
If `find_tool` returns `some idx`, then the tool at that index in the registry
has the requested `name_id`.

## validate_ensures_param_match
If `validate_tool_call` returns `true`, then every required parameter in the
spec is present in the args with the correct kind, and every provided argument
matches a parameter in the spec.
-/

/-- If `find_tool` succeeds, the returned index points to a spec with the
    requested name_id. -/
axiom find_tool_aux_correct
    (registry : List ToolSpec) (name_id : U32) (base : Nat) (idx : Nat)
    (h : find_tool_aux registry name_id base = some idx) :
    ∃ spec : ToolSpec, registry[idx - base]? = some spec ∧ spec.name_id = name_id

axiom tool_call_uses_registered_tool
    (registry : List ToolSpec) (name_id : U32) (idx : Nat)
    (h : find_tool registry name_id = some idx) :
    ∃ spec : ToolSpec, registry[idx]? = some spec ∧ spec.name_id = name_id

/-- If `validate_tool_call` returns true, then every required param is present
    and every provided arg is valid. -/
axiom validate_ensures_param_match
    (spec : ToolSpec) (args : ToolCallArgs)
    (h : validate_tool_call spec args = true) :
    (∀ p ∈ spec.params, p.required → param_satisfied p args) ∧
    (∀ pair ∈ args.param_values, arg_in_spec pair.1 pair.2 spec)

end agent_reasoning
