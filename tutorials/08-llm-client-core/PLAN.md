# Tutorial 08: LLM Client Core

## Goal

Implement pure LLM protocol logic (request building, conversation management, streaming) with no HTTP or JSON; prove well-formedness and context window guarantees.

## File Structure

```
08-llm-client-core/
├── README.md
├── PLAN.md
├── rust/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── message_types.rs    # Role, ChatMessage, ToolCallInfo, ToolResultInfo
│   │   ├── request.rs          # Request, build_request, RequestError
│   │   ├── response.rs         # Response, parse_response (simplified format)
│   │   ├── token_estimate.rs   # estimate_tokens_single, estimate_tokens
│   │   ├── conversation.rs     # Conversation: new, append, trim_to_context
│   │   ├── streaming.rs        # StreamAccumulator, chunk_response
│   │   └── transport_trait.rs  # trait LlmTransport
│   └── tests/
│       ├── request_tests.rs
│       ├── response_tests.rs
│       ├── conversation_tests.rs
│       └── streaming_tests.rs
└── lean/
    ├── lakefile.lean
    ├── lean-toolchain
    ├── LlmClientCore/Types.lean
    ├── LlmClientCore/Funs.lean
    ├── LlmClientCore/RequestSpec.lean
    ├── LlmClientCore/RequestProof.lean
    ├── LlmClientCore/ConversationSpec.lean
    ├── LlmClientCore/ConversationProof.lean
    ├── LlmClientCore/StreamingProof.lean
    ├── LlmClientCore/TokenEstimateProof.lean
    └── LlmClientCore/StringUtils.lean
```

## Rust Code Outline

### Types

- **`Role`** — Enum: `System | User | Assistant | Tool`.
- **`ChatMessage`** — Enum: `RoleMessage { role: Role, content: Vec<u8> } | ToolCall { id: u32, name: Vec<u8>, arguments: Vec<u8> } | ToolResult { call_id: u32, content: Vec<u8>, is_error: bool }`. Uses `Vec<u8>` instead of `String` for Aeneas compatibility.
- **`ToolCallInfo`** — `id: u32, name: Vec<u8>, arguments: Vec<u8>`. Describes a tool invocation extracted from an assistant response.
- **`ToolResultInfo`** — `call_id: u32, content: Vec<u8>, is_error: bool`. The result of executing a tool.
- **`Request`** — `model: Vec<u8>, messages: Vec<ChatMessage>, temperature: u32 (fixed-point, scale 100), max_tokens: u32, tools: Vec<ToolSpec>`. Fixed-point temperature avoids floating point.
- **`ToolSpec`** — `name: Vec<u8>, description: Vec<u8>, parameters: Vec<ToolParam>`. Simplified tool schema.
- **`ToolParam`** — `name: Vec<u8>, param_type: ParamType, required: bool`.
- **`ParamType`** — Enum: `PString | PInteger | PBoolean`.
- **`RequestError`** — Enum: `EmptyMessages | NoModel | ContextOverflow | InvalidTemperature`.
- **`Response`** — `message: ChatMessage, finish_reason: FinishReason, usage: Usage`.
- **`FinishReason`** — Enum: `Stop | Length | ToolUse`.
- **`Usage`** — `prompt_tokens: u32, completion_tokens: u32`.
- **`Conversation`** — `messages: Vec<ChatMessage>, max_context_tokens: u32`. Manages the sliding window of messages.
- **`StreamAccumulator`** — `chunks: Vec<Vec<u8>>, done: bool`. Accumulates streaming response chunks.

### Functions

- **`build_request(model: &[u8], conv: &Conversation, temperature: u32, max_tokens: u32, tools: &[ToolSpec]) -> Result<Request, RequestError>`** — Validates inputs and constructs a well-formed request.
- **`validate_request(req: &Request) -> Result<(), RequestError>`** — Check invariants: non-empty messages, valid temperature (0..=200), model present.
- **`parse_response(data: &[u8]) -> Result<Response, u32>`** — Parse a simplified response format (not JSON — a fixed binary layout for verifiability).
- **`estimate_tokens_single(msg: &ChatMessage) -> u32`** — Approximate token count for one message (~4 chars per token heuristic).
- **`estimate_tokens(msgs: &[ChatMessage]) -> u32`** — Sum of per-message estimates plus overhead.
- **`conversation_new(max_context_tokens: u32) -> Conversation`** — Create empty conversation with token budget.
- **`conversation_append(conv: Conversation, msg: ChatMessage) -> Conversation`** — Append a message; does not trim.
- **`trim_to_context(conv: Conversation) -> Conversation`** — Remove oldest non-system messages until estimated tokens fit within `max_context_tokens`.
- **`stream_accumulator_new() -> StreamAccumulator`** — Create empty accumulator.
- **`stream_push(acc: StreamAccumulator, chunk: Vec<u8>) -> StreamAccumulator`** — Append a chunk.
- **`stream_finish(acc: &StreamAccumulator) -> Vec<u8>`** — Concatenate all chunks into a single response body.
- **`chunk_response(data: &[u8], chunk_size: u32) -> Vec<Vec<u8>>`** — Split a response into chunks of given size (simulates streaming).

### Estimated Lines

~550 lines Rust.

## Generated Lean (Approximate)

Aeneas will produce:

- **`LlmClientCore/Types.lean`**: Inductive types for `Role`, `ChatMessage`, `ToolSpec`, `Request`, `Response`, `Conversation`, `StreamAccumulator`. Fixed-point `temperature` is just `U32`. `Vec<u8>` becomes `List U8`.
- **`LlmClientCore/Funs.lean`**: All functions with `Result` return types. `build_request` returns `Result Request RequestError`. `trim_to_context` is a while-loop that drops from the front of the list. `estimate_tokens` folds over the message list.

Key translation notes:
- `Vec<u8>` becomes `List U8` — string-like data without actual string operations.
- The `LlmTransport` trait becomes a Lean structure with function fields; the pure core never calls it, but specs reference it.
- Fixed-point arithmetic (temperature as `u32` scaled by 100) translates directly to `U32` arithmetic.
- `chunk_response` and `stream_finish` are inverse operations, which is directly provable on lists.

## Theorems to Prove

### `build_request_well_formed`
**Statement:** If `build_request` returns `Ok req`, then `req.messages` is non-empty, `req.model` is non-empty, and `estimate_tokens req.messages <= conv.max_context_tokens`.
**Proof strategy:** Unfold `build_request` and follow each validation branch; each `Ok` path implies all checks passed.

### `append_preserves_alternation`
**Statement:** If a conversation's messages satisfy the alternation property (user/assistant messages alternate after the system message), and the appended message has the correct next role, then the result still alternates.
**Proof strategy:** Induct on the message list; the new message is appended at the end, so only the last element changes. Check role of last existing message against the new one.

### `trim_respects_context`
**Statement:** After `trim_to_context conv`, `estimate_tokens conv.messages <= conv.max_context_tokens`.
**Proof strategy:** The trim loop removes messages while the estimate exceeds the budget. Show the loop invariant: each iteration strictly decreases the token estimate (since each removed message contributes > 0 tokens). Termination plus the loop exit condition give the bound.

### `chunks_concat_eq_original`
**Statement:** `stream_finish (fold stream_push (stream_accumulator_new()) (chunk_response data chunk_size)) = data` (chunking then reassembling is identity).
**Proof strategy:** Show `chunk_response` partitions `data` into contiguous slices, and `stream_finish` concatenates them. Use `List.join_splitAt` style lemma.

### `estimate_within_factor_2`
**Statement:** For any message `msg`, `estimate_tokens_single msg <= 2 * actual_tokens msg` where `actual_tokens` is axiomatized.
**Proof strategy:** This uses an **axiom** that relates the 4-chars-per-token heuristic to actual tokenization. The axiom states `actual_tokens msg <= msg.content.length` and `msg.content.length / 4 <= actual_tokens msg`. The proof combines these with the definition of `estimate_tokens_single`.

### Estimated Lines

~750 lines proofs.

## New Lean Concepts Introduced

- **Modeling external APIs as traits with specs**: The `LlmTransport` trait becomes a Lean structure. Specifications describe what a conforming implementation must satisfy without providing one.
- **Axioms for external properties**: The relationship between byte length and token count cannot be proved purely — it is axiomatized. The tutorial discusses when axioms are appropriate and how to minimize the trusted base.
- **Approximation proofs**: Proving that an estimate is within a constant factor of the true value, using upper and lower bound axioms.

## Cross-References

- **From Tutorial 04 (State Machines):** The trait-as-structure pattern for `LlmTransport` follows the same approach used for state machine transitions in Tutorial 04.
- **From Tutorial 05 (Message Protocol):** Serialization ideas inform the simplified binary response format in `parse_response`.
- **To Tutorial 09 (Agent Reasoning):** The `ChatMessage` and `Request` types are consumed by the agent reasoning engine.
- **To Tutorial 10 (Multi-Agent Orchestrator):** Conversation management is used per-agent in the orchestrator.
- **To Tutorial 11 (Full Integration):** The LLM client core provides the protocol layer for the final application.
