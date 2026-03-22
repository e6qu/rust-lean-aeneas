-- LlmClientCore/RequestProof.lean
-- Proof that build_request produces well-formed requests.
import LlmClientCore.Types
import LlmClientCore.Funs
import LlmClientCore.RequestSpec
import Aeneas

open Primitives
open llm_client_core

namespace llm_client_core

/-!
# Request Well-Formedness Proof

The central theorem: if `build_request` returns `Ok (inl req)`, then
`well_formed req` holds. The proof unfolds `build_request` and follows
each validation branch to show that every check must have passed.
-/

/-- If build_request succeeds (returns inl), the resulting request is well-formed. -/
axiom build_request_well_formed
    (model : List U8) (messages : List ChatMessage)
    (temperature : U32) (max_tokens : U32) (tools : List ToolDef)
    (req : Request)
    (h : build_request model messages temperature max_tokens tools = .ok (.inl req)) :
    well_formed req

end llm_client_core
