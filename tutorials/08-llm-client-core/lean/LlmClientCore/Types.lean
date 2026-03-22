-- LlmClientCore/Types.lean
-- Simulated Aeneas output: inductive types for the LLM client core.
import Aeneas

open Primitives

namespace llm_client_core

/-- Chat roles. -/
inductive Role where
  | System
  | User
  | Assistant
deriving DecidableEq, BEq, Repr, Inhabited

/-- Tool call information from the assistant. -/
structure ToolCallInfo where
  id : List U8
  function_name : List U8
  arguments : List U8
deriving DecidableEq, Repr

/-- Tool result information fed back to the model. -/
structure ToolResultInfo where
  tool_call_id : List U8
  content : List U8
deriving DecidableEq, Repr

/-- A message in a chat conversation. -/
inductive ChatMessage where
  | RoleMessage (role : Role) (content : List U8)
  | ToolCall (info : ToolCallInfo)
  | ToolResult (info : ToolResultInfo)
deriving DecidableEq, Repr

instance : Inhabited ChatMessage where
  default := .RoleMessage default []

/-- Simplified tool definition. -/
structure ToolDef where
  name : List U8
  description : List U8
  parameters_schema : List U8
deriving DecidableEq, Repr

/-- A validated LLM API request. Temperature is fixed-point (scale 100). -/
structure Request where
  model : List U8
  messages : List ChatMessage
  temperature : U32
  max_tokens : U32
  tools : List ToolDef
deriving Repr

/-- Request validation errors. -/
inductive RequestError where
  | EmptyMessages
  | NoSystemMessage
  | TemperatureTooHigh
  | MaxTokensZero
deriving DecidableEq, Repr

/-- Finish reason for a response. -/
inductive FinishReason where
  | Stop
  | Length
  | ToolUse
deriving DecidableEq, Repr

/-- Token usage statistics. -/
structure Usage where
  prompt_tokens : U32
  completion_tokens : U32
deriving Repr

/-- A piece of response content. -/
inductive ResponseContent where
  | Text (content : List U8)
  | ToolUse (info : ToolCallInfo)
deriving Repr

/-- A parsed LLM response. -/
structure Response where
  content : List ResponseContent
  finish_reason : FinishReason
  usage : Usage
deriving Repr

/-- Response parse errors. -/
inductive ResponseParseError where
  | InvalidFormat
  | MissingField
  | InvalidFinishReason
deriving DecidableEq, Repr

/-- A managed conversation with a context window budget. -/
structure Conversation where
  messages : List ChatMessage
  max_context_tokens : U32
deriving Repr

/-- Conversation manipulation errors. -/
inductive ConvError where
  | NotSystemFirst
  | InvalidAlternation
deriving DecidableEq, Repr

/-- Streaming accumulator. -/
structure StreamAccumulator where
  chunks : List (List U8)
  accumulated : List U8
deriving Repr

/-- Transport errors (for the trait specification). -/
inductive TransportError where
  | ConnectionFailed
  | Timeout
  | InvalidResponse
deriving DecidableEq, Repr

/-- Transport trait modeled as a structure with function fields.
    The pure core never provides an implementation. -/
structure LlmTransport where
  send_request : Request → Result (Response ⊕ TransportError)

end llm_client_core
