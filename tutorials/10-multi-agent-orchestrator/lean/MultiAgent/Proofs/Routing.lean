-- MultiAgent/Proofs/Routing.lean
-- Proof that Direct addressing resolves to the target agent.
import MultiAgent.Types
import MultiAgent.Funs
import Aeneas

open Primitives
open multi_agent

namespace multi_agent

/-!
# Routing Proofs

We prove that `resolve_recipient` for `Direct(id)` always returns the singleton
list `[id]`, regardless of the router state or the set of all agents.
-/

/-- `resolve_recipient` on `Direct id` always returns `[id]`. -/
theorem resolve_direct_delivers_to_target
    (router : Router) (id : AgentId) (agents : List AgentId) :
    resolve_recipient router (.Direct id) agents = [id] := by
  simp [resolve_recipient]

/-- `resolve_recipient` on `Broadcast` returns all agents. -/
theorem resolve_broadcast_returns_all
    (router : Router) (agents : List AgentId) :
    resolve_recipient router .Broadcast agents = agents := by
  simp [resolve_recipient]

/-- `topic_subscribers` on an empty subscription list returns `[]`. -/
theorem topic_subscribers_empty (topic_id : U32) :
    topic_subscribers [] topic_id = [] := by
  simp [topic_subscribers]

/-- `envelope_targets` with `Direct id` returns true iff the id matches. -/
theorem envelope_targets_direct (env : Envelope) (agent_id : AgentId)
    (h : env.recipient = .Direct agent_id) :
    envelope_targets env agent_id = true := by
  simp [envelope_targets, h]

/-- `envelope_targets` with `Broadcast` always returns true. -/
theorem envelope_targets_broadcast (env : Envelope) (agent_id : AgentId)
    (h : env.recipient = .Broadcast) :
    envelope_targets env agent_id = true := by
  simp [envelope_targets, h]

end multi_agent
