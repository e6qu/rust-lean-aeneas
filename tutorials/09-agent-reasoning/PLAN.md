# Tutorial 09: Agent Reasoning

## Goal

Implement a verified single-agent reasoning engine with state machine, chain-of-thought, tool registry, retry logic, and guardrails; prove termination and safety.

## File Structure

```
09-agent-reasoning/
├── README.md
├── PLAN.md
├── rust/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── agent_state.rs      # AgentPhase, AgentEvent, AgentAction, agent_transition
│       ├── reasoning.rs        # Step, Decision, is_chain_well_formed, append_step
│       ├── tool_registry.rs    # ToolSpec, ToolParam, find_tool, validate_tool_call
│       ├── decision.rs         # DecisionContext, decide_next_action
│       ├── retry.rs            # RetryState, next_retry, should_retry
│       ├── guardrails.rs       # GuardrailConfig, check_*, all_guards_pass
│       └── agent.rs            # AgentConfig, AgentSnapshot, agent_step, agent_run
└── lean/
    ├── lakefile.lean
    ├── lean-toolchain
    ├── AgentReasoning.lean
    ├── AgentReasoning/
    │   ├── Types.lean
    │   ├── Funs.lean
    │   └── Proofs/
    │       ├── StateMachine.lean
    │       ├── Termination.lean
    │       ├── ToolSafety.lean
    │       ├── ChainWF.lean
    │       ├── Retry.lean
    │       └── Guardrails.lean
```

## Rust Code Outline

### Types

- **`AgentPhase`** — Enum with 7 states: `Idle | Observing | Thinking | Deciding | Acting | WaitingForTool | Done`. Represents the agent lifecycle.
- **`AgentEvent`** — Enum: `UserInput(Vec<u8>) | LlmResponse(Vec<u8>) | ToolResult { call_id: u32, output: Vec<u8>, is_error: bool } | Timeout | Tick`. External stimuli.
- **`AgentAction`** — Enum: `SendToLlm(Vec<u8>) | CallTool { name: Vec<u8>, args: Vec<u8> } | Respond(Vec<u8>) | Wait | Fail(Vec<u8>)`. Outputs from transitions.
- **`Step`** — Enum: `Observe(Vec<u8>) | Think(Vec<u8>) | Decide(Decision) | Act(AgentAction)`. One step in the chain-of-thought.
- **`Decision`** — Enum: `UseTool { tool_name: Vec<u8>, args: Vec<u8> } | RespondToUser(Vec<u8>) | ContinueThinking | GiveUp(Vec<u8>)`.
- **`ToolSpec`** — `name: Vec<u8>, description: Vec<u8>, parameters: Vec<ToolParam>`. Describes a registered tool.
- **`ToolParam`** — `name: Vec<u8>, param_type: u8, required: bool`. `param_type`: 0=string, 1=integer, 2=boolean.
- **`ToolCall`** — `tool_name: Vec<u8>, arguments: Vec<(Vec<u8>, Vec<u8>)>`. A concrete invocation (name-value pairs).
- **`RetryState`** — `attempt: u32, max_attempts: u32, base_delay_ms: u32, last_error: Vec<u8>`. Exponential backoff state.
- **`GuardrailConfig`** — `max_steps: u32, max_tool_calls: u32, max_output_bytes: u32, forbidden_tools: Vec<Vec<u8>>`. Safety limits.
- **`AgentConfig`** — `tools: Vec<ToolSpec>, guardrails: GuardrailConfig, max_retries: u32`.
- **`AgentSnapshot`** — `phase: AgentPhase, chain: Vec<Step>, tool_call_count: u32, step_count: u32, retry_state: RetryState, output_bytes: u32`. Complete agent state at a point in time.

### Functions

- **`agent_transition(phase: AgentPhase, event: AgentEvent) -> Option<(AgentPhase, AgentAction)>`** — Pure state machine transition. Returns `None` for invalid transitions (including from terminal states).
- **`is_chain_well_formed(chain: &[Step]) -> bool`** — Check that the chain follows the pattern: Observe, then alternating Think/Decide/Act sequences.
- **`append_step(chain: Vec<Step>, step: Step) -> Option<Vec<Step>>`** — Append only if the result would still be well-formed.
- **`find_tool(registry: &[ToolSpec], name: &[u8]) -> Option<u32>`** — Lookup tool index by name.
- **`validate_tool_call(registry: &[ToolSpec], call: &ToolCall) -> bool`** — Check tool exists, required params present, types match.
- **`decide_next_action(context: &DecisionContext) -> Decision`** — Deterministic decision logic based on chain history and available tools.
- **`next_retry(state: RetryState) -> RetryState`** — Increment attempt, compute exponential backoff delay.
- **`should_retry(state: &RetryState) -> bool`** — `state.attempt < state.max_attempts`.
- **`check_step_limit(snapshot: &AgentSnapshot, config: &GuardrailConfig) -> bool`** — `snapshot.step_count < config.max_steps`.
- **`check_tool_limit(snapshot: &AgentSnapshot, config: &GuardrailConfig) -> bool`** — `snapshot.tool_call_count < config.max_tool_calls`.
- **`check_output_limit(snapshot: &AgentSnapshot, config: &GuardrailConfig) -> bool`** — `snapshot.output_bytes < config.max_output_bytes`.
- **`check_forbidden_tool(call_name: &[u8], config: &GuardrailConfig) -> bool`** — Tool name not in forbidden list.
- **`all_guards_pass(snapshot: &AgentSnapshot, config: &GuardrailConfig, call_name: &[u8]) -> bool`** — Conjunction of all checks.
- **`agent_step(snapshot: AgentSnapshot, event: AgentEvent, config: &AgentConfig) -> AgentSnapshot`** — One step: transition, check guards, update chain and counters.
- **`agent_run(snapshot: AgentSnapshot, events: &[AgentEvent], config: &AgentConfig) -> AgentSnapshot`** — Bounded loop over events; stops when `phase = Done` or events exhausted.

### Estimated Lines

~550 lines Rust.

## Generated Lean (Approximate)

Aeneas will produce:

- **`AgentReasoning/Types.lean`**: Inductive types for all enums. `AgentPhase` is a 7-constructor inductive. `Step` and `Decision` are nested inductives. `AgentSnapshot` is a structure with `List Step` for the chain.
- **`AgentReasoning/Funs.lean`**: `agent_transition` as a large match on `(AgentPhase, AgentEvent)` pairs. `agent_run` as a fuel-bounded recursive function (translated from the Rust while loop). `validate_tool_call` iterates over parameter lists.

Key translation notes:
- The 7-state `AgentPhase` translates to a 7-constructor inductive; Lean's pattern matching covers all 49 phase/event combinations.
- `agent_run` becomes a fuel-bounded recursion (Aeneas translates while loops this way). The fuel is the length of the events list.
- `Vec<(Vec<u8>, Vec<u8>)>` for tool arguments becomes `List (List U8 × List U8)`.
- All `u32` counters become `U32` with overflow checks preserved.

## Theorems to Prove

### `agent_transition_from_terminal_is_none`
**Statement:** `agent_transition Done event = none` for all `event`.
**Proof strategy:** Case split on `event`; in each case, unfold `agent_transition` and observe the `Done` branch returns `none`.

### `agent_run_terminates`
**Statement:** `agent_run` always terminates: given a finite event list of length `n`, the function returns after at most `n` recursive calls.
**Proof strategy:** The function is structurally recursive on the event list (after Aeneas translation). Each recursive call processes one event and recurses on the tail. Lean's structural recursion checker accepts this directly.

### `tool_call_uses_registered_tool`
**Statement:** If `all_guards_pass snapshot config call_name = true` and the agent step produces `CallTool`, then `find_tool config.tools call_name = some i` for some `i`.
**Proof strategy:** Unfold `all_guards_pass` and `check_forbidden_tool`; the guard conjunction includes a check that the tool is in the registry. Follow the implication chain.

### `append_preserves_well_formed`
**Statement:** If `is_chain_well_formed chain = true` and `append_step chain step = some chain'`, then `is_chain_well_formed chain' = true`.
**Proof strategy:** `append_step` returns `some` only when the new step follows the well-formedness rules. Unfold both definitions and show the extended chain satisfies each rule.

### `next_retry_increases_attempt`
**Statement:** `(next_retry state).attempt = state.attempt + 1` (when no overflow).
**Proof strategy:** Unfold `next_retry`; direct by computation.

### `retry_stops_at_max`
**Statement:** If `state.attempt >= state.max_attempts`, then `should_retry state = false`.
**Proof strategy:** Unfold `should_retry`; the condition `attempt < max_attempts` fails, so the result is `false`.

### `all_guards_pass_implies_within_limits`
**Statement:** If `all_guards_pass snapshot config call_name = true`, then `snapshot.step_count < config.max_steps` and `snapshot.tool_call_count < config.max_tool_calls` and `snapshot.output_bytes < config.max_output_bytes`.
**Proof strategy:** Unfold `all_guards_pass` as a conjunction of Boolean checks; each conjunct directly gives one inequality.

### Estimated Lines

~600 lines proofs.

## New Lean Concepts Introduced

- **Reachability analysis**: Proving which agent phases are reachable from a given starting phase by analyzing the transition function. Uses the `AgentPhase` inductive and case analysis on transitions.
- **Termination via decreasing measures**: `agent_run` terminates because the event list strictly decreases. This illustrates how Aeneas-generated fuel-bounded recursion maps to Lean's termination checking.
- **Schema validation proofs**: Proving that `validate_tool_call` correctly checks all required parameters against a tool spec. Involves reasoning about list membership and for-all predicates over parameter lists.

## Cross-References

- **From Tutorial 04 (State Machines):** `agent_transition` follows exactly the state machine pattern from Tutorial 04, now with 7 states and richer events/actions.
- **From Tutorial 08 (LLM Client Core):** The `ChatMessage` and request types from Tutorial 08 inform the `AgentEvent::LlmResponse` and `AgentAction::SendToLlm` payloads.
- **To Tutorial 10 (Multi-Agent Orchestrator):** Each agent in the orchestrator runs an instance of this reasoning engine internally.
- **To Tutorial 11 (Full Integration):** Agent reasoning is a core component of the final application.
