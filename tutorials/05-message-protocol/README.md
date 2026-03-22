[← Previous: Tutorial 04](../04-state-machines/README.md) | [Index](../README.md) | [Next: Tutorial 06 →](../06-buffer-management/README.md)

# Tutorial 05: Message Protocol

Implement TLV (Tag-Length-Value) binary serialization and deserialization for a
message protocol, then formally verify roundtrip correctness, framing
correctness for streaming reads, and error detection for malformed input.

## Table of Contents

1. [Introduction](#1-introduction)
2. [Prerequisites](#2-prerequisites)
3. [The TLV Format](#3-the-tlv-format)
4. [Message Types](#4-message-types)
5. [Serialization](#5-serialization)
6. [Deserialization](#6-deserialization)
7. [Running Aeneas](#7-running-aeneas)
8. [Lean: Reasoning About Byte Lists](#8-lean-reasoning-about-byte-lists)
9. [Proving Roundtrip Correctness](#9-proving-roundtrip-correctness)
10. [Length Prefix Correctness](#10-length-prefix-correctness)
11. [Message Framing](#11-message-framing)
12. [Proving Framing Correctness](#12-proving-framing-correctness)
13. [Proving Error Detection](#13-proving-error-detection)
14. [Exercises](#14-exercises)
15. [Looking Ahead](#15-looking-ahead)

---

## 1. Introduction

Binary protocols are everywhere: network packets, file formats, IPC channels.
A single off-by-one error in serialization or deserialization can corrupt data,
crash a program, or open a security hole. In this tutorial we build a complete
TLV (Tag-Length-Value) message protocol in Rust and prove in Lean that:

- **Roundtrip correctness**: deserializing a serialized message always recovers
  the original.
- **Length prefix correctness**: the encoded length field always matches the
  actual payload size.
- **Framing correctness**: when multiple messages are concatenated (as in a TCP
  stream), we can extract each one without loss or reordering.
- **Error detection**: malformed input (bad tags, truncated data) is reliably
  rejected.

The Rust code is intentionally written in an Aeneas-friendly style: no traits,
no iterators, no closures. Every loop is a `while` with an explicit index.
Byte buffers are `Vec<u8>` rather than `String`, avoiding UTF-8 invariants.

### Why verify serialization?

Serialization bugs are notoriously hard to catch with testing alone. Consider:

- A field written in little-endian but read in big-endian works on some
  platforms but silently corrupts data on others.
- A length prefix that is off by one only manifests when the payload happens to
  end at a buffer boundary.
- Framing errors that lose or duplicate messages only appear under specific
  timing conditions in a streaming context.

Formal verification gives us a *proof* that these classes of bugs cannot occur,
regardless of the message content, size, or chunking.

---

## 2. Prerequisites

Before starting this tutorial, you should be comfortable with:

- **Tutorial 01**: Setting up a Rust + Lean project, running Aeneas.
- **Tutorial 02**: Basic Lean proofs, `simp`, `omega`.
- **Tutorial 03**: Inductive types and pattern matching in Lean.
- **Tutorial 04**: State machines (the `FrameAccumulator` is a state machine
  that accumulates partial reads from a byte stream).

You will need:

- Rust (edition 2024)
- Aeneas (for translating Rust to Lean)
- Lean 4 with the Aeneas backend library

---

## 3. The TLV Format

TLV (Tag-Length-Value) is one of the simplest self-describing binary formats.
Every message on the wire has exactly three parts:

```
+-------+-------------------+---------------------+
| Tag   | Length            | Value (Payload)     |
| 1 byte| 4 bytes (BE u32) | `length` bytes      |
+-------+-------------------+---------------------+
```

- **Tag** (1 byte): Identifies the message variant.
  - `0x00` = Text
  - `0x01` = Command
  - `0x02` = Error
  - `0x03` = Heartbeat

- **Length** (4 bytes, big-endian `u32`): The number of bytes in the payload
  that follows. This does *not* include the tag or the length field itself.

- **Value** (variable): The payload, whose internal structure depends on the
  tag.

### Why big-endian?

Network protocols traditionally use big-endian ("network byte order"). While
Rust and most modern CPUs are little-endian, we write explicit shift-and-mask
code so the encoding is platform-independent. This also makes the Lean
translation straightforward: each byte is an arithmetic expression on the
original value.

### Payload sub-formats

Each message variant has its own payload encoding:

**Text** (`tag = 0`):
```
[length_prefix: u32 BE][raw bytes]
```
The payload is a single length-prefixed byte sequence.

**Command** (`tag = 1`):
```
[cmd_type: u8][arg_count: u32 BE][arg1_len: u32 BE][arg1 bytes]...[argN_len: u32 BE][argN bytes]
```
One byte for the command type, then a list of length-prefixed byte vectors.

**Error** (`tag = 2`):
```
[error_code: u8][detail_len: u32 BE][detail bytes]
```
One byte for the error code, then a length-prefixed detail message.

**Heartbeat** (`tag = 3`):
```
[timestamp: u64 BE]
```
A single 64-bit timestamp in big-endian.

---

## 4. Message Types

We define the message types in `rust/src/message.rs`:

```rust
#[derive(Clone, PartialEq, Debug)]
pub enum CmdType { Ping, Quit, Help, Run }

#[derive(Clone, PartialEq, Debug)]
pub enum ErrorCode { InvalidInput, NotFound, Internal }

#[derive(Clone, PartialEq, Debug)]
pub enum Message {
    Text(Vec<u8>),
    Command(CmdType, Vec<Vec<u8>>),
    Error(ErrorCode, Vec<u8>),
    Heartbeat(u64),
}

#[derive(Clone, PartialEq, Debug)]
pub enum ParseError { NotEnoughData, InvalidTag, InvalidLength, Malformed }
```

### Design decisions

**`Vec<u8>` instead of `String`**: Strings carry a UTF-8 invariant that would
need to be maintained (and verified) across serialization boundaries. By using
raw byte vectors, we keep the Aeneas translation simple: `Vec<u8>` maps to
`List U8` in Lean, with no encoding invariants.

**Flat enums for `CmdType` and `ErrorCode`**: Each variant maps to a single
tag byte. This keeps the wire format compact and the Lean proofs by case
analysis tractable. In a real system you might use `Custom(u8)` variants;
we omit those here for simplicity.

**`ParseError` as a separate enum**: Rather than using `Result<_, String>`,
we enumerate the specific failure modes. This lets us state precise theorems
like "tags > 3 always produce `InvalidTag`".

---

## 5. Serialization

The serialization code lives in `rust/src/serialize.rs`. The key design
principle is **layered encoding**: primitive helpers build up to the
message-level `serialize` function.

### Primitive: `write_u32_be`

```rust
fn write_u32_be(val: u32, out: &mut Vec<u8>) {
    out.push(((val >> 24) & 0xFF) as u8);
    out.push(((val >> 16) & 0xFF) as u8);
    out.push(((val >> 8) & 0xFF) as u8);
    out.push((val & 0xFF) as u8);
}
```

This encodes a `u32` as 4 big-endian bytes by shifting and masking. Each byte
is extracted from most significant to least significant. In Lean, this becomes
pure arithmetic on `U8` values, which `omega` can reason about.

### Length-prefixed bytes: `serialize_bytes`

```rust
fn serialize_bytes(data: &[u8], out: &mut Vec<u8>) {
    write_u32_be(data.len() as u32, out);
    let mut i: usize = 0;
    while i < data.len() {
        out.push(data[i]);
        i += 1;
    }
}
```

This is the building block for all variable-length fields. The length comes
first (as a `u32`), then the raw bytes. In Lean, this translates to
`write_u32_be len ++ data`.

### Top-level: `serialize`

```rust
pub fn serialize(msg: &Message) -> Vec<u8> {
    let mut out = Vec::new();
    let tag = message_tag(msg);
    let payload = build_payload(msg);
    out.push(tag);
    write_u32_be(payload.len() as u32, &mut out);
    // append payload bytes
    ...
    out
}
```

The structure is always `[tag][length][payload]`. The `build_payload` helper
constructs the variant-specific payload, and `serialize` wraps it in the TLV
envelope.

### Why explicit `while` loops?

Aeneas translates Rust to Lean by converting loops into recursive functions
with termination proofs. While `for` loops over iterators are idiomatic Rust,
Aeneas handles `while` loops with explicit index variables more reliably.
The pattern is always:

```rust
let mut i: usize = 0;
while i < data.len() {
    // body using data[i]
    i += 1;
}
```

This translates to a `Nat.rec` or explicit decreasing recursion in Lean.

---

## 6. Deserialization

The deserialization code lives in `rust/src/deserialize.rs`. It mirrors the
serialization structure but returns `Result<(T, usize), ParseError>` where the
`usize` is the number of bytes consumed.

### Offset tracking

Every read function takes an `offset` parameter and returns the new offset:

```rust
fn read_u32_be(data: &[u8], offset: usize) -> Result<(u32, usize), ParseError> {
    if offset + 4 > data.len() {
        return Err(ParseError::NotEnoughData);
    }
    let val = (data[offset] as u32) << 24
        | (data[offset + 1] as u32) << 16
        | (data[offset + 2] as u32) << 8
        | (data[offset + 3] as u32);
    Ok((val, offset + 4))
}
```

This pattern makes the Lean translation clean: each function produces a pair
`(value, new_offset)`, and the caller threads the offset through. In Lean,
this becomes monadic `do` notation with `let (val, off) <- read_u32_be data offset`.

### Error handling

Every function checks that enough bytes are available before reading. The
check `offset + N > data.len()` is the first thing each function does. If
the data is too short, we return `NotEnoughData` immediately.

The top-level `deserialize` function:

1. Checks the data is non-empty.
2. Reads the tag byte at position 0.
3. Reads the length field at offset 1.
4. Checks that `5 + length <= data.len()` (the full TLV envelope fits).
5. Dispatches on the tag to parse the variant-specific payload.
6. Returns `(message, total_bytes_consumed)`.

Unknown tags produce `InvalidTag`. A command or error with an empty payload
(where we need at least one sub-type byte) produces `Malformed`.

---

## 7. Running Aeneas

To translate the Rust code to Lean:

```bash
cd rust/
cargo build
aeneas -backend lean src/lib.rs
```

Aeneas produces `Types.lean` (inductive types) and `Funs.lean` (function
bodies). In this tutorial the Lean files are simulated Aeneas output,
pre-written to match what Aeneas would generate.

Key things to check in the generated output:

- `Vec<u8>` becomes `List U8`.
- `Result<T, E>` becomes Aeneas's `Result T`.
- `while` loops become recursive functions with `decreasing_by` obligations.
- Shift-and-mask operations become Lean `>>>` and `|||` on `U8`/`U32`/`U64`.

---

## 8. Lean: Reasoning About Byte Lists

The Lean proofs reason about `List U8` -- the model of byte buffers. Several
patterns recur throughout the proofs:

### List append associativity

Serialized messages are built by concatenation:
```
serialize msg = [tag] ++ length_bytes ++ payload
```

We frequently need `List.append_assoc` to reassociate nested appends.

### `omega` for arithmetic

The `omega` tactic handles linear arithmetic goals involving `Nat`, `U32.val`,
and list lengths. Typical goals:

- `4 + payload.length <= data.length` (bounds check)
- `offset + 4 = offset + 4` (offset arithmetic)
- `(serialize msg).length >= 5` (minimum message size)

### Take and drop

When deserializing from a concatenation `prefix ++ suffix`, we use:

- `List.take n (xs ++ ys)` when `n = xs.length` gives `xs`.
- `List.drop n (xs ++ ys)` when `n = xs.length` gives `ys`.

These lemmas, proved in `Utils.lean`, are the workhorses of the roundtrip proof.

### Shift-mask roundtrip

The deepest lemma is that `read_u32_be (write_u32_be val) 0 = ok (val, 4)`.
This requires showing that decomposing a `U32` into 4 bytes and recomposing
gives back the original value. The proof involves bitwise arithmetic:

```
((val >>> 24) % 256) <<< 24
  ||| ((val >>> 16) % 256) <<< 16
  ||| ((val >>> 8) % 256) <<< 8
  ||| (val % 256)
  = val
```

This is essentially the statement that a number equals the sum of its base-256
digits weighted by powers of 256.

---

## 9. Proving Roundtrip Correctness

The main theorem lives in `RoundtripProof.lean`:

```lean
theorem roundtrip (msg : Message) :
    deserialize (serialize msg) = ok (msg, serialize_length msg)
```

### Proof strategy

1. **Case split** on `msg` (Text, Command, Error, Heartbeat).
2. For each variant, **unfold** `serialize` to get the concrete byte list.
3. **Unfold** `deserialize` and show:
   - The tag byte matches the expected variant.
   - `read_u32_be` on the length field returns the correct payload length.
   - The variant-specific payload parser recovers the original fields.
4. Each step uses the primitive roundtrip lemmas from `Utils.lean`.

### Example: Text roundtrip

For `Message.Text data`:

```
serialize (Text data) = [0x00] ++ write_u32_be (4 + data.length) ++ write_u32_be data.length ++ data
```

Deserializing:
1. Tag is `0x00` -> Text variant.
2. Length field reads `4 + data.length` -> payload is `write_u32_be data.length ++ data`.
3. `read_bytes` reads the length prefix (`data.length`), then takes that many bytes -> `data`.
4. Result: `(Text data, 5 + 4 + data.length)` = `(Text data, serialize_length (Text data))`.

Each step relies on `read_u32_be_append` to read past concatenated data.

---

## 10. Length Prefix Correctness

```lean
theorem length_prefix_correct (msg : Message) :
    read_u32_be (serialize msg) 1 = ok (payload_length msg, 5)
```

This says: if we read 4 bytes starting at offset 1 (skipping the tag byte)
of any serialized message, we get back the payload length.

The proof is direct: unfold `serialize`, observe that bytes 1-4 are
`write_u32_be (payload.length)`, and apply `read_u32_be_write_u32_be`.

We also prove helper facts:

- `serialize_total_length`: the total length is always `5 + payload_length`.
- `heartbeat_payload_length`: Heartbeat payloads are exactly 8 bytes.
- `text_payload_length`: Text payloads are `4 + data.length` bytes.

---

## 11. Message Framing

In a streaming context (TCP, Unix pipes), messages arrive as arbitrary chunks
of bytes. The `FrameAccumulator` in `rust/src/framing.rs` handles this:

```rust
pub struct FrameAccumulator {
    buffer: Vec<u8>,
}

impl FrameAccumulator {
    pub fn new() -> Self { ... }
    pub fn feed(&mut self, data: &[u8]) -> Vec<Result<Message, ParseError>> { ... }
}
```

### How it works

1. `feed` appends incoming bytes to an internal buffer.
2. It then loops, attempting to `deserialize` from the buffer:
   - **Success**: The decoded message is pushed to the result list, and the
     consumed bytes are drained from the buffer.
   - **NotEnoughData**: The loop breaks, waiting for more data.
   - **Other error**: The error is recorded, one byte is skipped, and the
     loop continues (error recovery by resynchronization).

### Why drain-and-retry?

The accumulator does not try to parse the length field separately from the
payload. Instead, it always calls the full `deserialize` and lets the
`NotEnoughData` error signal when more bytes are needed. This keeps the
logic simple and matches the formal model: `deserialize` is a total function
that either succeeds or fails.

### Test: byte-at-a-time feeding

The test `framing_partial_reads` feeds a serialized message one byte at a time.
The accumulator returns an empty list for each incomplete byte, then returns
the full message when the last byte arrives. This exercises the `NotEnoughData`
path on every iteration.

---

## 12. Proving Framing Correctness

The framing theorem in `FramingProof.lean`:

```lean
theorem framing_no_loss (msgs : List Message) :
    frame_extract_all (msgs.flatMap serialize) = ok msgs
```

This says: if we concatenate the serializations of a list of messages, then
extracting messages from the result recovers the original list in order.

### Proof by induction

- **Base case** (`msgs = []`): The concatenation is empty, and
  `frame_extract_all []` returns `ok []`.

- **Inductive step** (`msgs = msg :: rest`): The concatenation is
  `serialize msg ++ rest.flatMap serialize`. By `roundtrip`, deserializing
  the first `serialize_length msg` bytes gives `(msg, serialize_length msg)`.
  Dropping those bytes leaves `rest.flatMap serialize`. By the induction
  hypothesis, `frame_extract_all` on the remainder gives `ok rest`.
  Therefore the full result is `ok (msg :: rest)`.

The key insight is that `roundtrip` guarantees `deserialize` consumes exactly
the bytes produced by `serialize`, so there is no "leakage" between messages.

---

## 13. Proving Error Detection

The error detection proofs in `ErrorDetectionProof.lean` show that the
deserializer correctly rejects malformed input:

### Invalid tag

```lean
theorem invalid_tag_rejected (data : List U8)
    (h_len : data.length >= 5)
    (h_tag : (data.get 0).val > 3) :
    exists e, deserialize data = fail e
```

The proof: after `deserialize` reads the tag and length, it matches on the
tag value. Tags 0-3 are handled; the wildcard case returns `fail`. Since
`h_tag` says the tag is > 3, the wildcard case applies.

### Truncated data

```lean
theorem truncated_data_rejected (data : List U8)
    (h : data.length > 0) (h_short : data.length < 5) :
    exists e, deserialize data = fail e
```

The proof: `deserialize` calls `read_u32_be data 1`, which requires
`1 + 4 <= data.length`, i.e., `data.length >= 5`. Since `h_short` says
`data.length < 5`, the read fails with `NotEnoughData`.

### Insufficient payload

```lean
theorem insufficient_payload_rejected (tag : U8) (claimed_len : U32)
    (payload : List U8)
    (h_tag : tag.val <= 3)
    (h_short : payload.length < claimed_len.val) :
    exists e, deserialize ([tag] ++ write_u32_be claimed_len ++ payload) = fail e
```

This handles the case where the length field claims more payload bytes than
actually exist. The check `total <= data.length` in `deserialize` catches this.

---

## 14. Exercises

### Exercise 1: Add a new message variant

Add a `Status` variant to `Message`:

```rust
Status(u8, Vec<u8>)  // (status_code, description)
```

with tag `0x04`. Update `serialize`, `deserialize`, and all Lean proofs.
The roundtrip proof should require one new case in the case split.

### Exercise 2: Prove roundtrip for your new variant

After adding `Status`, prove:

```lean
theorem roundtrip_status (code : U8) (desc : List U8) :
    deserialize (serialize (.Status code desc)) = ok (.Status code desc, ...)
```

### Exercise 3: Maximum message size

Add a check that the payload length does not exceed `2^24` (16 MiB). Prove
that `serialize` never produces a message larger than `5 + 2^24` bytes.

### Exercise 4: Checksums

Add a CRC32 checksum after the payload:

```
[tag][length][payload][crc32: 4 bytes]
```

Update `deserialize` to verify the checksum. Prove that the roundtrip still
holds (the checksum of the serialized payload always matches).

### Exercise 5: Property-based testing

Use `proptest` or `quickcheck` to generate random `Message` values and verify
the Rust roundtrip. Compare the confidence from 10,000 random tests vs. the
Lean proof that covers *all* possible messages.

---

## 15. Looking Ahead

The message protocol from this tutorial is used directly in:

- **Tutorial 10**: Multi-agent communication, where agents send `Command` and
  `Text` messages over a shared channel. The `roundtrip` theorem guarantees
  that messages are never corrupted in transit.

- **Tutorial 11**: Distributed consensus, where `Heartbeat` messages implement
  failure detection. The `framing_no_loss` theorem ensures that message
  boundaries are preserved even when TCP segments arrive in arbitrary chunks.

The proof techniques introduced here -- byte-list reasoning, shift-mask
roundtrips, induction on message lists -- recur throughout the remaining
tutorials whenever we need to reason about serialized data.

---

## Summary

| What we built | Where |
|---------------|-------|
| Message types | `rust/src/message.rs` |
| TLV serialization | `rust/src/serialize.rs` |
| TLV deserialization | `rust/src/deserialize.rs` |
| Stream framing | `rust/src/framing.rs` |
| Rust tests | `rust/tests/roundtrip_tests.rs` |
| Lean types (simulated Aeneas) | `lean/MessageProtocol/Types.lean` |
| Lean functions (simulated Aeneas) | `lean/MessageProtocol/Funs.lean` |
| Specification helpers | `lean/MessageProtocol/Spec.lean` |
| Byte-list lemmas | `lean/MessageProtocol/Utils.lean` |
| Roundtrip proof | `lean/MessageProtocol/RoundtripProof.lean` |
| Framing proof | `lean/MessageProtocol/FramingProof.lean` |
| Length prefix proof | `lean/MessageProtocol/LengthProof.lean` |
| Error detection proof | `lean/MessageProtocol/ErrorDetectionProof.lean` |

**Key theorems**:
- `roundtrip`: `deserialize (serialize msg) = ok (msg, serialize_length msg)`
- `framing_no_loss`: `frame_extract_all (msgs.flatMap serialize) = ok msgs`
- `length_prefix_correct`: the length field encodes the true payload size
- `invalid_tag_rejected`: unknown tags are rejected
- `truncated_data_rejected`: short inputs are rejected

---

[← Previous: Tutorial 04](../04-state-machines/README.md) | [Index](../README.md) | [Next: Tutorial 06 →](../06-buffer-management/README.md)
