[← Previous: Tutorial 07](../07-tui-core/README.md) | [Index](../README.md) | [Next: Tutorial 09 →](../09-agent-reasoning/README.md)

# Tutorial 08: LLM Client Core

Implement pure LLM protocol logic -- request building, conversation management,
token estimation, response parsing, and streaming -- with no HTTP or JSON
dependencies. Then formally verify well-formedness invariants, context window
guarantees, and streaming correctness in Lean.

## Table of Contents

1. [Introduction](#1-introduction)
2. [Prerequisites](#2-prerequisites)
3. [The Functional Core / Imperative Shell Pattern](#3-the-functional-core--imperative-shell-pattern)
4. [Message Types](#4-message-types)
5. [Request Building](#5-request-building)
6. [Running Aeneas](#6-running-aeneas)
7. [Lean Types: The Aeneas Translation](#7-lean-types-the-aeneas-translation)
8. [Proving Request Well-Formedness](#8-proving-request-well-formedness)
9. [Conversation Management](#9-conversation-management)
10. [Proving Alternation Preservation](#10-proving-alternation-preservation)
11. [Proving Trim Respects Context](#11-proving-trim-respects-context)
12. [Token Estimation](#12-token-estimation)
13. [Axioms and Approximation Proofs](#13-axioms-and-approximation-proofs)
14. [Response Parsing](#14-response-parsing)
15. [Streaming](#15-streaming)
16. [Proving Streaming Correctness](#16-proving-streaming-correctness)
17. [The Transport Trait](#17-the-transport-trait)
18. [Exercises](#18-exercises)
19. [Looking Ahead](#19-looking-ahead)

---

## 1. Introduction

Large language model APIs follow a structured protocol: you send a request
containing a model name, a list of messages, parameters like temperature and
max tokens, and optionally tool definitions. The API returns a response with
generated content, a finish reason, and token usage statistics. Between
requests, you manage a conversation -- a growing list of messages that must
fit within a context window.

This protocol logic is entirely pure: it validates inputs, manages data
structures, estimates sizes, and parses binary formats. None of it requires
network I/O, JSON parsing, or any external dependency. By isolating this pure
core, we can:

1. **Test it thoroughly** with standard unit tests.
2. **Translate it to Lean** with Aeneas.
3. **Prove key properties** that testing alone cannot guarantee.

The properties we prove include:

- **Request well-formedness**: if `build_request` succeeds, the result
  satisfies all structural invariants (non-empty messages, system prompt
  first, valid temperature, positive max tokens).
- **Alternation preservation**: appending a correctly-roled message to a
  valid conversation preserves the user/assistant alternation pattern.
- **Context window respect**: after trimming, the estimated token count
  fits within the budget (or only the system message remains).
- **Streaming correctness**: chunking a response and reassembling the
  chunks via the accumulator recovers the original data.
- **Token estimate bound**: the estimate is within a constant factor of
  the actual token count (using axiomatized tokenizer properties).

### Why verify LLM client logic?

LLM applications are increasingly used in safety-critical contexts: medical
advice, legal analysis, financial decisions. Bugs in the protocol layer can
cause:

- **Context window overflow**: silently dropping messages, leading to
  incoherent responses.
- **Malformed requests**: sending invalid parameters that cause API errors
  or unexpected behavior.
- **Conversation corruption**: breaking the user/assistant alternation,
  confusing the model.
- **Streaming data loss**: losing chunks or reordering them, corrupting
  the response.

These bugs are subtle and hard to catch with testing alone. A formal proof
that the core logic is correct gives us confidence that the protocol layer
behaves as specified, regardless of message content or conversation length.

---

## 2. Prerequisites

Before starting this tutorial, you should be comfortable with:

- **Tutorial 01**: Setting up a Rust + Lean project, running Aeneas.
- **Tutorial 02**: Basic Lean proofs, `simp`, `omega`.
- **Tutorial 03**: Inductive types and pattern matching in Lean.
- **Tutorial 04**: State machines and the trait-as-structure pattern.
- **Tutorial 05**: Binary serialization and deserialization, byte-list
  reasoning.

You will need:

- Rust (edition 2024)
- Aeneas (for translating Rust to Lean)
- Lean 4 with the Aeneas backend library

---

## 3. The Functional Core / Imperative Shell Pattern

The architecture of this tutorial follows the **functional core / imperative
shell** pattern, a design principle that is particularly well suited to formal
verification:

- **Functional core**: Pure functions that validate, transform, and query data.
  No side effects, no I/O, no mutable global state. This is the part we
  verify.
- **Imperative shell**: The outer layer that performs HTTP requests, reads
  configuration, handles retries, and manages the event loop. This layer
  calls the functional core but is not itself verified.

In our case:

| Functional Core (verified)        | Imperative Shell (not verified)   |
|-----------------------------------|-----------------------------------|
| `build_request`                   | HTTP client                       |
| `parse_response`                  | JSON serialization                |
| `Conversation::append`            | User input handling               |
| `Conversation::trim_to_context`   | Retry logic                       |
| `estimate_tokens`                 | Rate limiting                     |
| `StreamAccumulator`               | SSE event parsing                 |
| `chunk_response`                  | WebSocket framing                 |

The `LlmTransport` trait sits at the boundary: the functional core defines
the interface, and the imperative shell provides the implementation.

This separation is not just a design convenience -- it is essential for
Aeneas. The translation tool requires pure Rust code: no traits with dynamic
dispatch, no async/await, no closures, no iterator adapters. By putting all
verifiable logic in the core, we ensure it translates cleanly.

---

## 4. Message Types

The foundation of the protocol is the message type system. Every interaction
with an LLM is a sequence of messages, each with a role and content.

### Roles

```rust
#[derive(Clone, PartialEq, Debug)]
pub enum Role {
    System,
    User,
    Assistant,
}
```

Three roles model the standard chat API:
- **System**: The initial instruction that sets the model's behavior.
- **User**: Messages from the human (or tool results).
- **Assistant**: Messages from the model (or tool calls).

### Chat messages

```rust
#[derive(Clone, PartialEq, Debug)]
pub enum ChatMessage {
    RoleMessage(Role, Vec<u8>),
    ToolCall(ToolCallInfo),
    ToolResult(ToolResultInfo),
}
```

We use `Vec<u8>` instead of `String` throughout. This is an Aeneas
requirement: `String` involves UTF-8 invariants that complicate verification.
Byte vectors translate directly to `List U8` in Lean.

The `ToolCall` and `ToolResult` variants support function calling, where the
model can request tool invocations and receive results.

### Role extraction

```rust
pub fn message_role(msg: &ChatMessage) -> Role {
    match msg {
        ChatMessage::RoleMessage(role, _) => role.clone(),
        ChatMessage::ToolCall(_) => Role::Assistant,
        ChatMessage::ToolResult(_) => Role::User,
    }
}
```

This function assigns a logical role to every message variant. Tool calls are
logically from the assistant; tool results are logically from the user (they
feed information back to the model). This mapping is crucial for the
alternation invariant we will prove later.

---

## 5. Request Building

A request bundles everything the API needs:

```rust
pub struct Request {
    pub model: Vec<u8>,
    pub messages: Vec<ChatMessage>,
    pub temperature: u32,
    pub max_tokens: u32,
    pub tools: Vec<ToolDef>,
}
```

Temperature is fixed-point with scale 100: the value 100 means 1.0, and 200
means 2.0. This avoids floating-point arithmetic, which Aeneas cannot handle.

### Validation

The `build_request` function validates all inputs before constructing the
request:

```rust
pub fn build_request(
    model: &[u8],
    messages: &[ChatMessage],
    temperature: u32,
    max_tokens: u32,
    tools: &[ToolDef],
) -> Result<Request, RequestError> {
    if messages.is_empty() {
        return Err(RequestError::EmptyMessages);
    }
    let first_role = message_role(&messages[0]);
    if first_role != Role::System {
        return Err(RequestError::NoSystemMessage);
    }
    if temperature > 200 {
        return Err(RequestError::TemperatureTooHigh);
    }
    if max_tokens == 0 {
        return Err(RequestError::MaxTokensZero);
    }
    // ... construct and return Ok(Request)
}
```

Each validation check corresponds to a conjunct in the `well_formed`
predicate we will define in Lean. The key insight is that **every `Ok` path
through `build_request` implies all checks passed**, which is exactly what
our proof will establish.

### Aeneas-friendly copying

Notice that we copy messages and tools into new vectors using explicit while
loops rather than `.to_vec()` or iterators:

```rust
let mut msg_vec: Vec<ChatMessage> = Vec::new();
let mut i: usize = 0;
while i < messages.len() {
    msg_vec.push(messages[i].clone());
    i += 1;
}
```

This is the Aeneas-compatible pattern: explicit indices, while loops, no
iterator adapters.

---

## 6. Running Aeneas

To translate the Rust code to Lean:

```bash
cd rust
cargo build
aeneas --input src/lib.rs --dest ../lean/LlmClientCore
```

Aeneas will generate `Types.lean` and `Funs.lean` in the output directory.
For this tutorial, we provide simulated Aeneas output that matches what the
tool would produce, so you can work through the proofs even without running
Aeneas directly.

Key translation points:

| Rust                    | Lean                          |
|-------------------------|-------------------------------|
| `Vec<u8>`               | `List U8`                     |
| `u32`                   | `U32`                         |
| `enum Role`             | `inductive Role`              |
| `struct Request`        | `structure Request`           |
| `Result<T, E>`          | `Result (T ⊕ E)`             |
| `while` loop            | Recursive function            |
| `trait LlmTransport`    | `structure LlmTransport`      |

The trait translation deserves special attention. Rust traits with `&mut self`
methods become Lean structures with function fields. The pure core never
*calls* the transport; it only defines the interface. This allows
specifications to describe what a conforming transport must satisfy without
providing a concrete implementation.

---

## 7. Lean Types: The Aeneas Translation

The generated (or simulated) types in `Types.lean` mirror the Rust types
directly:

```lean
inductive Role where
  | System
  | User
  | Assistant
deriving DecidableEq, Repr

inductive ChatMessage where
  | RoleMessage (role : Role) (content : List U8)
  | ToolCall (info : ToolCallInfo)
  | ToolResult (info : ToolResultInfo)
deriving DecidableEq, Repr

structure Request where
  model : List U8
  messages : List ChatMessage
  temperature : U32
  max_tokens : U32
  tools : List ToolDef
deriving Repr
```

The `DecidableEq` derivation is important: it lets us use `=` in `if`
expressions and in proofs that require decidability.

---

## 8. Proving Request Well-Formedness

### The specification

In `RequestSpec.lean`, we define what it means for a request to be well-formed:

```lean
def well_formed (req : Request) : Prop :=
  req.messages ≠ [] ∧
  (match req.messages with
   | [] => False
   | msg :: _ => message_role msg = .System) ∧
  req.temperature.val ≤ 200 ∧
  req.max_tokens.val > 0
```

This is a conjunction of four properties, each corresponding to one of the
validation checks in `build_request`.

### The proof

In `RequestProof.lean`, we prove that `build_request` only returns `Ok` when
all checks pass:

```lean
theorem build_request_well_formed
    (model : List U8) (messages : List ChatMessage)
    (temperature : U32) (max_tokens : U32) (tools : List ToolDef)
    (req : Request)
    (h : build_request model messages temperature max_tokens tools
         = .ok (.inl req)) :
    well_formed req
```

The proof strategy is straightforward: unfold `build_request`, case-split on
messages, and observe that each `if` guard must have been false (otherwise we
would have returned an error, contradicting our hypothesis `h`).

This is a classic **validation implies specification** proof pattern. The
runtime code does the validation; the proof shows that passing validation
implies the specification holds.

---

## 9. Conversation Management

The `Conversation` struct manages a sliding window of messages:

```rust
pub struct Conversation {
    pub messages: Vec<ChatMessage>,
    pub max_context_tokens: u32,
}
```

### Creating a conversation

```rust
pub fn new(system_msg: Vec<u8>, max_context: u32) -> Self {
    let mut messages: Vec<ChatMessage> = Vec::new();
    messages.push(ChatMessage::RoleMessage(Role::System, system_msg));
    Conversation { messages, max_context_tokens: max_context }
}
```

A new conversation always starts with the system prompt. This establishes
the base case for the alternation invariant.

### Appending messages

```rust
pub fn append(&mut self, msg: ChatMessage) -> Result<(), ConvError> {
    // Check alternation: after System, expect User;
    // after User, expect Assistant; after Assistant, expect User.
    let last_role = message_role(&self.messages[self.messages.len() - 1]);
    let new_role = message_role(&msg);
    let valid = match last_role {
        Role::System => new_role == Role::User,
        Role::User => new_role == Role::Assistant,
        Role::Assistant => new_role == Role::User,
    };
    if !valid {
        return Err(ConvError::InvalidAlternation);
    }
    self.messages.push(msg);
    Ok(())
}
```

The alternation check ensures that messages follow the expected pattern:
system, user, assistant, user, assistant, ... with tool calls counting as
assistant and tool results counting as user.

### Trimming to context

```rust
pub fn trim_to_context(&mut self) {
    while self.messages.len() > 1
        && estimate_tokens(&self.messages) > self.max_context_tokens
    {
        self.messages.remove(1);
    }
}
```

The trimming strategy is simple: remove the oldest non-system message
(index 1) until the estimate fits or only the system message remains.

In Lean, this while loop becomes a recursive function with a termination
argument based on the decreasing list length.

---

## 10. Proving Alternation Preservation

### The alternation predicate

```lean
def valid_alternation : List ChatMessage → Prop
  | [] => True
  | [msg] => message_role msg = .System
  | msg₁ :: msg₂ :: rest =>
    (match message_role msg₁ with
     | .System    => message_role msg₂ = .User
     | .User      => message_role msg₂ = .Assistant
     | .Assistant  => message_role msg₂ = .User) ∧
    valid_alternation (msg₂ :: rest)
```

This recursive predicate checks that every adjacent pair of messages
respects the alternation rule. A singleton list must be a system message.

### The preservation theorem

```lean
theorem append_preserves_alternation
    (msgs : List ChatMessage) (msg : ChatMessage)
    (h_alt : valid_alternation msgs)
    (h_last : msgs ≠ [] →
      match message_role (msgs.getLast!) with
      | .System    => message_role msg = .User
      | .User      => message_role msg = .Assistant
      | .Assistant  => message_role msg = .User) :
    valid_alternation (msgs ++ [msg])
```

The proof proceeds by induction on the message list. The key observation is
that appending at the end only affects the last-to-new transition. The
inductive hypothesis handles all earlier pairs, and the `h_last` hypothesis
handles the new pair.

---

## 11. Proving Trim Respects Context

The trim theorem states that after trimming, the estimated token count is
within budget:

```lean
theorem trim_respects_context (conv : Conversation) :
    let trimmed := trim_to_context conv
    (estimate_tokens trimmed.messages).val ≤
      trimmed.max_context_tokens.val ∨
    trimmed.messages.length ≤ 1
```

The proof relies on the loop structure of `trim_to_context_aux`:

1. **Loop invariant**: Each iteration removes one message, strictly
   decreasing the list length.
2. **Exit condition**: The loop exits when the estimate fits or only the
   system message remains.
3. **Termination**: The list length is a well-founded measure that decreases
   on each iteration.

The disjunction in the conclusion handles the edge case where the system
message alone exceeds the budget. In that case, we cannot trim further, but
we guarantee at least the system message is preserved.

---

## 12. Token Estimation

Token estimation is a pure heuristic:

```rust
pub const BYTES_PER_TOKEN: u32 = 4;
pub const MESSAGE_OVERHEAD: u32 = 4;

pub fn estimate_tokens_single(msg: &ChatMessage) -> u32 {
    let content = message_content(msg);
    let content_len = content.len() as u32;
    content_len / BYTES_PER_TOKEN + MESSAGE_OVERHEAD
}
```

The ~4 bytes per token heuristic is standard for English text with models
like GPT-4 and Claude. The overhead accounts for role tags, separators,
and other per-message framing.

This is intentionally simple. A production system would use the model's
actual tokenizer, but that would introduce an external dependency that
cannot be verified. Instead, we prove that our estimate is within a
constant factor of the true count, using axioms about the tokenizer.

---

## 13. Axioms and Approximation Proofs

### The trusted base

We introduce two axioms about the relationship between byte length and token
count:

```lean
axiom actual_tokens : ChatMessage → Nat

axiom actual_tokens_upper_bound (msg : ChatMessage) :
  actual_tokens msg ≤ (message_content msg).length

axiom actual_tokens_lower_bound (msg : ChatMessage) :
  (message_content msg).length ≤ 4 * actual_tokens msg
```

The first axiom says each token corresponds to at least one byte. The second
says each token corresponds to at most 4 bytes. Together, they bound the
tokenizer's behavior.

### When are axioms appropriate?

Axioms should be used sparingly. Each axiom is an **unverified assumption**
that could be false, potentially invalidating all proofs that depend on it.
In our case, the axioms are appropriate because:

1. They describe properties of an **external component** (the tokenizer)
   that cannot be expressed in our formal system.
2. They are **empirically validated**: for English text with standard
   tokenizers, the 1-to-4 bytes-per-token range is well established.
3. They are **clearly documented** so readers know exactly what is assumed.

The alternative -- not proving anything about token estimation -- would be
worse, because we would lose the ability to reason about context window
management entirely.

### The approximation theorem

```lean
theorem estimate_within_factor_2 (msg : ChatMessage) :
    (estimate_tokens_single msg).val ≤
      2 * (actual_tokens msg + MESSAGE_OVERHEAD.val)
```

This says our estimate is at most twice the actual token count plus overhead.
The proof combines the definition of `estimate_tokens_single` with the
axioms:

- From `actual_tokens_lower_bound`: `content_len ≤ 4 * actual_tokens msg`
- Therefore: `content_len / 4 ≤ actual_tokens msg`
- Therefore: `content_len / 4 + overhead ≤ actual_tokens msg + overhead`
- Therefore: `estimate ≤ 2 * (actual_tokens msg + overhead)`

The factor of 2 comes from the overhead being counted once in the estimate
and once in the bound.

---

## 14. Response Parsing

The response parser reads a simplified binary format (not JSON):

```
[finish_reason: u8]
[prompt_tokens: u32 BE]
[completion_tokens: u32 BE]
[content_count: u32 BE]
For each content item:
  [content_type: u8]  -- 0 = Text, 1 = ToolUse
  If Text: [text: length-prefixed bytes]
  If ToolUse: [id: lp bytes][function_name: lp bytes][arguments: lp bytes]
```

We use a binary format rather than JSON for the same reason as Tutorial 05:
binary parsing with explicit offsets translates cleanly to Lean and is easier
to reason about than string-based JSON parsing.

The `parse_response` function validates every field and returns structured
errors:

```rust
pub enum ResponseParseError {
    InvalidFormat,
    MissingField,
    InvalidFinishReason,
}
```

Each error variant identifies a specific failure mode, making it possible
to write precise specifications about error detection.

---

## 15. Streaming

Streaming simulates how LLM APIs deliver responses incrementally. We
provide two components:

### Chunking

```rust
pub fn chunk_response(full: &[u8], chunk_size: usize) -> Vec<Vec<u8>> {
    // Split `full` into chunks of at most `chunk_size` bytes
}
```

This simulates the server splitting a response into chunks for delivery.

### Accumulation

```rust
pub struct StreamAccumulator {
    pub chunks: Vec<Vec<u8>>,
    pub accumulated: Vec<u8>,
}

impl StreamAccumulator {
    pub fn new() -> Self { /* empty */ }
    pub fn add_chunk(&mut self, chunk: &[u8]) { /* append */ }
    pub fn is_complete(&self, expected_len: usize) -> bool { /* check */ }
    pub fn get_accumulated(&self) -> &[u8] { /* return */ }
}
```

The accumulator stores each chunk individually (for debugging/replay) and
also maintains a running concatenation for quick access.

These two operations are **inverses**: chunking splits data, and accumulation
reassembles it. This inverse relationship is exactly what we prove in Lean.

---

## 16. Proving Streaming Correctness

### The roundtrip theorem

```lean
theorem chunks_concat_eq_original (data : List U8) (n : Nat) (hn : n > 0) :
    stream_finish (fold_push (chunk_response data n)
      stream_accumulator_new) = data
```

This states that chunking data and then feeding the chunks through the
accumulator recovers the original data.

The proof uses two key insights:

1. `chunk_response data n` produces a list of chunks whose `join` equals
   `data`. This follows from the `take_drop_eq` lemma: taking `n` elements
   and dropping `n` elements from a list and concatenating gives back the
   original.

2. `fold_push` over a list of chunks accumulates their concatenation. This
   is proved by induction, using the fact that `stream_push` appends each
   chunk to the accumulated buffer.

### The accumulator correctness

```lean
theorem fold_push_accumulated (chunks : List (List U8))
    (acc : StreamAccumulator) :
    (fold_push chunks acc).accumulated =
      acc.accumulated ++ chunks.join
```

This helper theorem shows that the accumulated buffer after processing all
chunks equals the initial buffer plus the concatenation of all chunks. When
the initial buffer is empty (as with `stream_accumulator_new`), this
simplifies to `chunks.join`.

---

## 17. The Transport Trait

The `LlmTransport` trait defines the boundary between the verified core and
the unverified shell:

```rust
pub trait LlmTransport {
    fn send_request(&mut self, req: &Request) -> Result<Response, TransportError>;
}
```

In Lean, this becomes a structure:

```lean
structure LlmTransport where
  send_request : Request → Result (Response ⊕ TransportError)
```

We can write specifications about what a "correct" transport must satisfy:

- If the request is well-formed, the transport should not return
  `InvalidResponse`.
- The response's `usage.prompt_tokens` should be consistent with the
  request's message count.

These specifications are not proved about any concrete implementation (we
do not have one in the pure core). Instead, they serve as **contracts** that
guide the imperative shell implementation.

This pattern -- defining interfaces with specifications in the verified core
and implementing them in the unverified shell -- is a powerful technique for
structuring verified software. It appeared in Tutorial 04 (state machine
transitions as trait methods) and will appear again in Tutorials 09-11.

---

## 18. Exercises

### Exercise 1: Add a `ModelNotSpecified` error

Add validation that the model name is non-empty. Update `build_request` to
return `RequestError::ModelNotSpecified` when `model` is empty. Update the
Lean `well_formed` predicate and extend the proof.

### Exercise 2: Prove response parse roundtrip

Write a `serialize_response` function that produces the binary format, and
prove that `parse_response (serialize_response resp) = Ok resp` for all
valid responses.

### Exercise 3: Tighter token estimate bound

Replace the factor-of-2 bound with a tighter bound. Define axioms that
narrow the bytes-per-token range (e.g., 2 to 6 bytes per token) and prove
the tighter bound.

### Exercise 4: Prove trim preserves alternation

Show that `trim_to_context` preserves the `valid_alternation` property.
Hint: removing `messages[1]` (the first user message after system) may
break alternation. Under what conditions is it preserved?

### Exercise 5: Implement a mock transport

Create a struct that implements `LlmTransport` by returning canned
responses. Write tests that exercise the full flow: build a request, send
it through the mock transport, parse the response, and update the
conversation.

### Exercise 6: Maximum conversation depth

Add a `max_messages` field to `Conversation` that limits the total number
of messages (independent of token count). Update `append` to check this
limit, and prove that the invariant is maintained.

---

## 19. Looking Ahead

This tutorial established the protocol layer for LLM interaction. The types
and functions defined here are consumed by the rest of the tutorial series:

- **Tutorial 09 (Agent Reasoning)**: Uses `ChatMessage` and `Request` to
  build the agent's reasoning engine. The agent decides what messages to
  send and how to interpret responses, building on the well-formedness
  guarantees proved here.

- **Tutorial 10 (Multi-Agent Orchestrator)**: Uses `Conversation` management
  per-agent in a multi-agent system. The context window guarantees from
  `trim_to_context` ensure each agent stays within its token budget.

- **Tutorial 11 (Full Integration)**: Combines the LLM client core with
  actual HTTP transport, JSON serialization, and the agent reasoning engine
  into a complete application. The verified core provides the trustworthy
  foundation.

The key lesson from this tutorial is the **functional core / imperative shell**
pattern: by isolating pure logic from I/O, we make verification tractable
without sacrificing the ability to build real applications. The transport
trait shows how to define clean boundaries between verified and unverified
code.

Another important lesson is the **appropriate use of axioms**. The token
estimation axioms represent a deliberate trade-off: we accept unverified
assumptions about the tokenizer in exchange for the ability to prove
meaningful bounds about context window management. The axioms are clearly
documented, empirically grounded, and minimal -- exactly the properties
that make axioms trustworthy.

---

[← Previous: Tutorial 07](../07-tui-core/README.md) | [Index](../README.md) | [Next: Tutorial 09 →](../09-agent-reasoning/README.md)
