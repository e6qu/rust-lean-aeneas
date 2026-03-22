-- MessageProtocol/Utils.lean
-- Hand-written utility lemmas for byte-list reasoning.
import MessageProtocol.Types
import MessageProtocol.Funs
import Aeneas

open Primitives
open message_protocol

namespace message_protocol

/-! ## Byte-list lemmas -/

/-- write_u32_be always produces exactly 4 bytes. -/
theorem write_u32_be_length (val : U32) :
    (write_u32_be val).length = 4 := by
  simp [write_u32_be]

/-- write_u64_be always produces exactly 8 bytes. -/
theorem write_u64_be_length (val : U64) :
    (write_u64_be val).length = 8 := by
  simp [write_u64_be]

/-- Reading back what was written for U32 yields the original value.
    This is the key primitive roundtrip lemma. -/
theorem read_u32_be_write_u32_be (val : U32) :
    read_u32_be (write_u32_be val) 0 = .ok (val, 4) := by
  simp [read_u32_be, write_u32_be]
  sorry  -- Requires bitwise arithmetic reasoning

/-- Reading back what was written for U64 yields the original value. -/
theorem read_u64_be_write_u64_be (val : U64) :
    read_u64_be (write_u64_be val) 0 = .ok (val, 8) := by
  simp [read_u64_be, write_u64_be]
  sorry  -- Requires bitwise arithmetic reasoning

/-- Appending then taking gives the prefix. -/
theorem list_take_append (xs ys : List α) :
    (xs ++ ys).take xs.length = xs := by
  simp [List.take_append]

/-- Appending then dropping gives the suffix. -/
theorem list_drop_append (xs ys : List α) :
    (xs ++ ys).drop xs.length = ys := by
  simp [List.drop_append]

/-- Length of append. -/
theorem list_append_length (xs ys : List α) :
    (xs ++ ys).length = xs.length + ys.length := by
  simp

/-- Reading u32 from a concatenation where the first 4+ bytes encode the value. -/
theorem read_u32_be_append (val : U32) (rest : List U8) :
    read_u32_be (write_u32_be val ++ rest) 0 = .ok (val, 4) := by
  sorry  -- Follows from read_u32_be_write_u32_be + list reasoning

/-- Reading u64 from a concatenation where the first 8+ bytes encode the value. -/
theorem read_u64_be_append (val : U64) (rest : List U8) :
    read_u64_be (write_u64_be val ++ rest) 0 = .ok (val, 8) := by
  sorry  -- Follows from read_u64_be_write_u64_be + list reasoning

/-- serialize_bytes produces length equal to 4 + data.length. -/
theorem serialize_bytes_length (data : List U8) :
    (serialize_bytes data).length = 4 + data.length := by
  simp [serialize_bytes, write_u32_be_length]

/-- The build_payload result has a computable length. -/
theorem build_payload_length_text (data : List U8) :
    (build_payload (.Text data)).length = 4 + data.length := by
  simp [build_payload, serialize_bytes_length]

theorem build_payload_length_heartbeat (ts : U64) :
    (build_payload (.Heartbeat ts)).length = 8 := by
  simp [build_payload, write_u64_be_length]

end message_protocol
