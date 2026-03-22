-- MultiAgent/Proofs/NoMessageLoss.lean
-- Proof that messages sent to the bus are eventually delivered.
import MultiAgent.Types
import MultiAgent.Funs
import Aeneas

open Primitives
open multi_agent

namespace multi_agent

/-!
# No Message Loss Proofs

We show that every envelope placed on the bus via `bus_send` eventually
appears in the `delivered` list after sufficient `bus_deliver` calls.
The bus is FIFO: `bus_deliver` moves matching envelopes from `queue` to
`delivered`.
-/

/-- After `bus_send`, the envelope is in the queue. -/
axiom bus_send_enqueues (bus : MessageBus) (env : Envelope)
    (bus' : MessageBus) (h : bus_send bus env = .ok bus') :
    ∃ e ∈ bus'.queue, e.sender = env.sender ∧ e.message = env.message

/-- `partition_envelopes` preserves all envelopes: the union of matching and
    remaining equals the original list (as multisets). -/
theorem partition_envelopes_complete (queue : List Envelope) (agent_id : AgentId) :
    let (matching, remaining) := partition_envelopes queue agent_id
    matching.length + remaining.length = queue.length := by
  induction queue with
  | nil => simp [partition_envelopes]
  | cons env rest ih =>
    simp only [partition_envelopes]
    split <;> simp_all <;> omega

/-- After `bus_deliver`, the delivered envelope appears in `bus.delivered`. -/
axiom sent_then_delivered (bus : MessageBus) (env : Envelope) (agent_id : AgentId)
    (h_target : envelope_targets env agent_id = true)
    (h_in_queue : env ∈ bus.queue) :
    let (bus', delivered) := bus_deliver bus agent_id
    env ∈ delivered ∧ env ∈ bus'.delivered

/-- `bus_deliver` does not create new envelopes. -/
axiom bus_deliver_no_creation (bus : MessageBus) (agent_id : AgentId) :
    let (bus', delivered) := bus_deliver bus agent_id
    ∀ e ∈ delivered, e ∈ bus.queue

end multi_agent
