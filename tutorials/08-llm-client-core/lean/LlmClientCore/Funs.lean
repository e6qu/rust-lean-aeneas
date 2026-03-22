-- LlmClientCore/Funs.lean
-- Simulated Aeneas output: function translations for the LLM client core.
import LlmClientCore.Types
import Aeneas

open Primitives
open llm_client_core

namespace llm_client_core

/-! ## Message helpers -/

/-- Extract the logical role of a message. -/
def message_role : ChatMessage → Role
  | .RoleMessage role _ => role
  | .ToolCall _         => .Assistant
  | .ToolResult _       => .User

/-- Extract the content bytes of a message (for token estimation). -/
def message_content : ChatMessage → List U8
  | .RoleMessage _ content => content
  | .ToolCall info         => info.arguments
  | .ToolResult info       => info.content

/-! ## Token estimation -/

def BYTES_PER_TOKEN : U32 := ⟨4⟩
def MESSAGE_OVERHEAD : U32 := ⟨4⟩

/-- Estimate tokens for a single message. -/
def estimate_tokens_single (msg : ChatMessage) : U32 :=
  let content := message_content msg
  let content_len : U32 := ⟨content.length⟩
  ⟨content_len.val / BYTES_PER_TOKEN.val + MESSAGE_OVERHEAD.val⟩

/-- Estimate total tokens for a list of messages. -/
def estimate_tokens_aux : List ChatMessage → Nat → Nat
  | [], acc => acc
  | msg :: rest, acc =>
    estimate_tokens_aux rest (acc + (estimate_tokens_single msg).val)

def estimate_tokens (messages : List ChatMessage) : U32 :=
  ⟨estimate_tokens_aux messages 0⟩

/-! ## Request building -/

/-- Build a validated request. Returns Ok or the appropriate error. -/
def build_request (model : List U8) (messages : List ChatMessage)
    (temperature : U32) (max_tokens : U32) (tools : List ToolDef)
    : Result (Request ⊕ RequestError) :=
  match messages with
  | [] => .ok (.inr .EmptyMessages)
  | msg :: _ =>
    if message_role msg != .System then
      .ok (.inr .NoSystemMessage)
    else if temperature.val > 200 then
      .ok (.inr .TemperatureTooHigh)
    else if max_tokens.val = 0 then
      .ok (.inr .MaxTokensZero)
    else
      .ok (.inl ⟨model, messages, temperature, max_tokens, tools⟩)

/-! ## Conversation management -/

/-- Create a new conversation with a system prompt. -/
def conversation_new (system_msg : List U8) (max_context : U32) : Conversation :=
  { messages := [.RoleMessage .System system_msg], max_context_tokens := max_context }

/-- Append a message to a conversation, checking alternation. -/
def conversation_append (conv : Conversation) (msg : ChatMessage)
    : Result (Conversation ⊕ ConvError) :=
  match conv.messages with
  | [] =>
    if message_role msg = .System then
      .ok (.inl { conv with messages := [msg] })
    else
      .ok (.inr .NotSystemFirst)
  | msgs =>
    let last := msgs.getLast!
    let last_role := message_role last
    let new_role := message_role msg
    let valid := match last_role with
      | .System    => new_role == .User
      | .User      => new_role == .Assistant
      | .Assistant  => new_role == .User
    if valid = true then
      .ok (.inl { conv with messages := conv.messages ++ [msg] })
    else
      .ok (.inr .InvalidAlternation)

/-- Trim conversation by removing messages[1] until tokens fit or len ≤ 1. -/
def trim_to_context_aux : List ChatMessage → U32 → List ChatMessage
  | [], _ => []
  | [sys], _ => [sys]
  | sys :: _ :: rest, max_tokens =>
    let msgs := sys :: rest
    if (estimate_tokens msgs).val > max_tokens.val then
      trim_to_context_aux msgs max_tokens
    else
      sys :: rest
  termination_by msgs => msgs.length
  decreasing_by all_goals sorry

def trim_to_context (conv : Conversation) : Conversation :=
  if (estimate_tokens conv.messages).val > conv.max_context_tokens.val then
    { conv with messages := trim_to_context_aux conv.messages conv.max_context_tokens }
  else
    conv

/-! ## Streaming -/

/-- Create an empty stream accumulator. -/
def stream_accumulator_new : StreamAccumulator :=
  { chunks := [], accumulated := [] }

/-- Add a chunk to the accumulator. -/
def stream_push (acc : StreamAccumulator) (chunk : List U8) : StreamAccumulator :=
  { chunks := acc.chunks ++ [chunk], accumulated := acc.accumulated ++ chunk }

/-- Concatenate all accumulated chunks. -/
def stream_finish (acc : StreamAccumulator) : List U8 :=
  acc.accumulated

/-- Split data into chunks of the given size. -/
def chunk_response_aux : List U8 → Nat → List (List U8) → List (List U8)
  | [], _, acc => acc.reverse
  | data, chunk_size, acc =>
    if chunk_size = 0 then acc.reverse
    else
      let chunk := data.take chunk_size
      let rest := data.drop chunk_size
      chunk_response_aux rest chunk_size (chunk :: acc)
  termination_by data => data.length
  decreasing_by all_goals sorry

def chunk_response (data : List U8) (chunk_size : Nat) : List (List U8) :=
  if chunk_size = 0 then []
  else chunk_response_aux data chunk_size []

end llm_client_core
