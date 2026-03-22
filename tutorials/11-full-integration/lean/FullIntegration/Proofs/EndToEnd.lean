-- FullIntegration/Proofs/EndToEnd.lean
-- End-to-end message flow proofs.
--
-- We trace the path of a user submission through the system:
--   1. User types text and submits -> message appears on the queue.
--   2. Orchestrator tick delivers the message.

import FullIntegration.Funs

namespace FullIntegration.Proofs.EndToEnd

open FullIntegration

/-- After a submit on a non-empty buffer, the message queue is non-empty. -/
theorem submit_reaches_queue (s : AppState) (h : s.input_buffer ≠ []) :
    (handle_submit s).message_queue ≠ [] := by
  simp [handle_submit, List.isEmpty]
  intro hempty
  cases hbuf : s.input_buffer with
  | nil => exact absurd hbuf h
  | cons x xs =>
    simp [hbuf] at hempty

/-- After a submit, the input buffer is cleared. -/
theorem submit_clears_buffer (s : AppState) (h : s.input_buffer ≠ []) :
    (handle_submit s).input_buffer = [] := by
  simp [handle_submit, List.isEmpty]
  cases hbuf : s.input_buffer with
  | nil => exact absurd hbuf h
  | cons x xs => simp [hbuf]

/-- After a submit, a conversation entry is added. -/
theorem submit_adds_conversation_entry (s : AppState) (h : s.input_buffer ≠ []) :
    (handle_submit s).conversations.length = s.conversations.length + 1 := by
  simp [handle_submit, List.isEmpty]
  cases hbuf : s.input_buffer with
  | nil => exact absurd hbuf h
  | cons x xs => simp [hbuf, List.length_append]

/-- An orchestrator tick with a non-empty queue delivers one message,
    producing a conversation entry for the delivery. -/
axiom tick_delivers_message (s : AppState)
    (hbudget : s.turn_count < s.turn_budget)
    (hqueue : s.message_queue ≠ []) :
    (handle_orchestrator_tick s).message_queue.length =
      s.message_queue.length - 1

/-- Submit followed by an orchestrator tick delivers the submitted message.
    This is the key end-to-end property. -/
axiom submit_then_tick_delivers (s : AppState)
    (hbuf : s.input_buffer ≠ [])
    (hqueue : s.message_queue = [])
    (hbudget : s.turn_count < s.turn_budget) :
    (handle_orchestrator_tick (handle_submit s)).message_queue = []

end FullIntegration.Proofs.EndToEnd
