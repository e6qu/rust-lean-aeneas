# Tutorial 05: Message Protocol

## Goal

Implement TLV (Tag-Length-Value) binary serialization and deserialization for a message protocol; prove roundtrip correctness, framing correctness for streaming reads, and error detection for malformed input.

## File Structure

```
05-message-protocol/
├── README.md
├── PLAN.md
├── rust/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── message.rs          # Message, CmdType, ErrorCode enums
│   │   ├── serialize.rs        # serialize, write_u32_be, write_u64_be
│   │   ├── deserialize.rs      # deserialize, read_u32_be, read_u64_be
│   │   └── framing.rs          # FrameAccumulator
│   └── tests/roundtrip_tests.rs
└── lean/
    ├── lakefile.lean
    ├── lean-toolchain
    ├── MessageProtocol/Types.lean      # generated
    ├── MessageProtocol/Funs.lean       # generated
    ├── MessageProtocol/Spec.lean       # pure spec functions
    ├── MessageProtocol/Utils.lean      # byte-list lemmas
    ├── MessageProtocol/RoundtripProof.lean
    ├── MessageProtocol/FramingProof.lean
    ├── MessageProtocol/LengthProof.lean
    └── MessageProtocol/ErrorDetectionProof.lean
```

## Rust Code Outline (~400 lines)

### Key Types

| Type | Definition | Description |
|------|-----------|-------------|
| `Message` | `enum { Text { payload: Vec<u8> }, Command { cmd: CmdType, args: Vec<u8> }, Error { code: ErrorCode, detail: Vec<u8> }, Heartbeat }` | Application-level message variants |
| `CmdType` | `enum { Ping, Status, Shutdown, Custom(u8) }` | Command sub-types |
| `ErrorCode` | `enum { InvalidInput, Timeout, Internal, Custom(u8) }` | Error code sub-types |
| `FrameAccumulator` | `struct { buffer: Vec<u8>, state: FrameState }` | Accumulates partial reads from a stream |
| `FrameState` | `enum { WaitingForHeader, WaitingForPayload { tag: u8, length: u32 } }` | Framing state machine |

### Wire Format (TLV)

```
[tag: u8] [length: u32 big-endian] [payload: length bytes]
```

- Tag values: `0x01` = Text, `0x02` = Command, `0x03` = Error, `0x04` = Heartbeat.
- Payload encoding varies by message type (command tag byte prepended for Command, error code byte prepended for Error, empty for Heartbeat).

### Key Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `serialize` | `(&Message) -> Vec<u8>` | Encode message to TLV byte sequence |
| `deserialize` | `(&[u8]) -> Result<(Message, usize), DeserError>` | Decode message from byte slice; returns bytes consumed |
| `write_u32_be` | `(u32) -> [u8; 4]` | Encode u32 as 4 big-endian bytes |
| `read_u32_be` | `(&[u8]) -> Result<u32, DeserError>` | Decode u32 from 4 big-endian bytes |
| `write_u64_be` | `(u64) -> [u8; 8]` | Encode u64 as 8 big-endian bytes |
| `read_u64_be` | `(&[u8]) -> Result<u64, DeserError>` | Decode u64 from 8 big-endian bytes |
| `FrameAccumulator::feed` | `(&mut self, &[u8]) -> Vec<Result<Message, DeserError>>` | Feed bytes into accumulator; returns fully framed messages |

Design note: All functions operate on `Vec<u8>` / `&[u8]` rather than `String` or `std::io::Read`, keeping the interface Aeneas-friendly.

## Generated Lean (approximate)

- **Types.lean**: Inductive types for `Message`, `CmdType`, `ErrorCode`, `FrameState`, plus structure for `FrameAccumulator`.
- **Funs.lean**: Monadic translations. `serialize` will produce a `Result (Vec U8)` and `deserialize` will produce a `Result (Message × Usize)`.

Key pattern: byte manipulation functions like `write_u32_be` become arithmetic on `U8` values with shifting and masking, which Lean can reason about via `omega` and bitwise lemmas.

## Theorems to Prove (~500 lines)

| Theorem | Statement | Proof Strategy |
|---------|-----------|----------------|
| `roundtrip` | `∀ msg, deserialize (serialize msg) = ok (msg, serialize_length msg)` | Case analysis on `Message` variant; for each variant, unfold `serialize` then `deserialize`; show tag matches, length is correct, payload decodes to original. Uses byte-list lemmas from `Utils.lean` |
| `length_prefix_correct` | `serialize msg = tag :: len_bytes ++ payload → read_u32_be len_bytes = ok payload.length` | Unfold `serialize`; show `write_u32_be` and `read_u32_be` are inverses; `omega` for arithmetic |
| `framing_no_loss` | `∀ msgs chunks, concat chunks = concat (map serialize msgs) → feed_all acc chunks = msgs` | Induction on chunks; show `FrameAccumulator` correctly reassembles split messages. Key lemma: header is always read atomically or buffered correctly |
| `invalid_tag_rejected` | `∀ bs, bs[0] ∉ {0x01, 0x02, 0x03, 0x04} → deserialize bs = err InvalidTag` | Direct computation on the tag-matching branch |

### Utility Lemmas (Utils.lean)

- `write_read_u32_roundtrip`: `read_u32_be (write_u32_be n) = ok n`
- `write_read_u64_roundtrip`: `read_u64_be (write_u64_be n) = ok n`
- `append_slice_eq`: Lemmas about `List.take` / `List.drop` on concatenated byte lists.

## New Lean Concepts Introduced

- **Byte sequence reasoning**: Working with `List U8` as the model for byte buffers; relating list operations (append, take, drop) to serialization structure.
- **`omega` for arithmetic bounds**: Using `omega` to discharge goals about byte widths, lengths, and offsets (e.g., `4 + payload.length ≤ buf.length`).
- **Serialization invariants**: The pattern of proving serialize/deserialize roundtrip by showing each field encodes/decodes correctly, then composing.
- **Two-phase proof structure**: First prove primitive roundtrips (`u32_be`, `u64_be`), then compose into message-level roundtrip.

## Cross-References

- **Prerequisites**: Tutorials 01-03 for basic proof techniques; Tutorial 04's state machine pattern is echoed in `FrameAccumulator` (which is essentially a state machine over `FrameState`).
- **Forward**: Tutorials 10-11 use this wire format for inter-agent communication. The `serialize`/`deserialize` functions and `roundtrip` theorem are imported directly.

## Estimated Lines of Code

| Component | Lines |
|-----------|-------|
| Rust source | ~400 |
| Rust tests | ~80 |
| Generated Lean (Types + Funs) | ~250 |
| Hand-written specs + utils | ~120 |
| Hand-written proofs | ~500 |
| README | ~400 |
| **Total** | **~1750** |
