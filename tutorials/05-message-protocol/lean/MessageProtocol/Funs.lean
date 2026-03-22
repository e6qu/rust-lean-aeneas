-- MessageProtocol/Funs.lean
-- Simulated Aeneas output: function translations for the message protocol.
import MessageProtocol.Types
import Aeneas

open Primitives
open message_protocol

namespace message_protocol

/-! ## Byte encoding helpers -/

/-- Encode a U32 as 4 big-endian bytes. -/
def write_u32_be (val : U32) : List U8 :=
  let b3 : U8 := ⟨(val.val >>> 24) % 256, by omega⟩
  let b2 : U8 := ⟨(val.val >>> 16) % 256, by omega⟩
  let b1 : U8 := ⟨(val.val >>> 8) % 256, by omega⟩
  let b0 : U8 := ⟨val.val % 256, by omega⟩
  [b3, b2, b1, b0]

/-- Encode a U64 as 8 big-endian bytes. -/
def write_u64_be (val : U64) : List U8 :=
  let b7 : U8 := ⟨(val.val >>> 56) % 256, by omega⟩
  let b6 : U8 := ⟨(val.val >>> 48) % 256, by omega⟩
  let b5 : U8 := ⟨(val.val >>> 40) % 256, by omega⟩
  let b4 : U8 := ⟨(val.val >>> 32) % 256, by omega⟩
  let b3 : U8 := ⟨(val.val >>> 24) % 256, by omega⟩
  let b2 : U8 := ⟨(val.val >>> 16) % 256, by omega⟩
  let b1 : U8 := ⟨(val.val >>> 8) % 256, by omega⟩
  let b0 : U8 := ⟨val.val % 256, by omega⟩
  [b7, b6, b5, b4, b3, b2, b1, b0]

/-- Read a big-endian U32 from a byte list. -/
def read_u32_be (data : List U8) (offset : Nat) : Result (U32 × Nat) :=
  if h : offset + 4 ≤ data.length then
    let b3 := data.get ⟨offset, by omega⟩
    let b2 := data.get ⟨offset + 1, by omega⟩
    let b1 := data.get ⟨offset + 2, by omega⟩
    let b0 := data.get ⟨offset + 3, by omega⟩
    let val : U32 := ⟨b3.val <<< 24 ||| b2.val <<< 16 ||| b1.val <<< 8 ||| b0.val, by omega⟩
    .ok (val, offset + 4)
  else
    .fail .panic

/-- Read a big-endian U64 from a byte list. -/
def read_u64_be (data : List U8) (offset : Nat) : Result (U64 × Nat) :=
  if h : offset + 8 ≤ data.length then
    let b7 := data.get ⟨offset, by omega⟩
    let b6 := data.get ⟨offset + 1, by omega⟩
    let b5 := data.get ⟨offset + 2, by omega⟩
    let b4 := data.get ⟨offset + 3, by omega⟩
    let b3 := data.get ⟨offset + 4, by omega⟩
    let b2 := data.get ⟨offset + 5, by omega⟩
    let b1 := data.get ⟨offset + 6, by omega⟩
    let b0 := data.get ⟨offset + 7, by omega⟩
    let val : U64 := ⟨b7.val <<< 56 ||| b6.val <<< 48 ||| b5.val <<< 40 ||| b4.val <<< 32
                      ||| b3.val <<< 24 ||| b2.val <<< 16 ||| b1.val <<< 8 ||| b0.val, by omega⟩
    .ok (val, offset + 8)
  else
    .fail .panic

/-- Write a length-prefixed byte list. -/
def serialize_bytes (data : List U8) : List U8 :=
  write_u32_be ⟨data.length, by sorry⟩ ++ data

/-- Write a list of byte vectors: count then each length-prefixed. -/
def serialize_vec_list : List (List U8) → List U8
  | [] => write_u32_be ⟨0, by omega⟩
  | vecs => write_u32_be ⟨vecs.length, by sorry⟩ ++ vecs.flatMap serialize_bytes

/-- CmdType to tag byte. -/
def cmd_type_tag : CmdType → U8
  | .Ping => ⟨0, by omega⟩
  | .Quit => ⟨1, by omega⟩
  | .Help => ⟨2, by omega⟩
  | .Run  => ⟨3, by omega⟩

/-- ErrorCode to tag byte. -/
def error_code_tag : ErrorCode → U8
  | .InvalidInput => ⟨0, by omega⟩
  | .NotFound     => ⟨1, by omega⟩
  | .Internal     => ⟨2, by omega⟩

/-- Tag byte for a message variant. -/
def message_tag : Message → U8
  | .Text _        => ⟨0, by omega⟩
  | .Command _ _   => ⟨1, by omega⟩
  | .Error _ _     => ⟨2, by omega⟩
  | .Heartbeat _   => ⟨3, by omega⟩

/-- Build the payload for a message (everything after the TLV header). -/
def build_payload : Message → List U8
  | .Text data         => serialize_bytes data
  | .Command cmd args  => [cmd_type_tag cmd] ++ serialize_vec_list args
  | .Error code detail => [error_code_tag code] ++ serialize_bytes detail
  | .Heartbeat ts      => write_u64_be ts

/-- Serialize a message into TLV wire format: [tag:U8][length:U32 BE][payload]. -/
def serialize (msg : Message) : List U8 :=
  let payload := build_payload msg
  [message_tag msg] ++ write_u32_be ⟨payload.length, by sorry⟩ ++ payload

/-- Read a length-prefixed byte list from data at offset. -/
def read_bytes (data : List U8) (offset : Nat) : Result (List U8 × Nat) :=
  do
    let (len, off) ← read_u32_be data offset
    let len_nat := len.val
    if h : off + len_nat ≤ data.length then
      let bytes := (data.drop off).take len_nat
      .ok (bytes, off + len_nat)
    else
      .fail .panic

/-- Read a list of length-prefixed byte vectors. -/
def read_vec_list_aux (data : List U8) (offset : Nat) : Nat → List (List U8) → Result (List (List U8) × Nat)
  | 0, acc => .ok (acc.reverse, offset)
  | n + 1, acc => do
    let (bytes, off) ← read_bytes data offset
    read_vec_list_aux data off n (bytes :: acc)

def read_vec_list (data : List U8) (offset : Nat) : Result (List (List U8) × Nat) :=
  do
    let (count, off) ← read_u32_be data offset
    read_vec_list_aux data off count.val []

/-- Decode a CmdType from a tag byte. -/
def cmd_type_from_tag (tag : U8) : Result CmdType :=
  match tag.val with
  | 0 => .ok .Ping
  | 1 => .ok .Quit
  | 2 => .ok .Help
  | 3 => .ok .Run
  | _ => .fail .panic

/-- Decode an ErrorCode from a tag byte. -/
def error_code_from_tag (tag : U8) : Result ErrorCode :=
  match tag.val with
  | 0 => .ok .InvalidInput
  | 1 => .ok .NotFound
  | 2 => .ok .Internal
  | _ => .fail .panic

/-- Deserialize a message from TLV wire format.
    Returns (message, bytes_consumed). -/
def deserialize (data : List U8) : Result (Message × Nat) :=
  if h : data.length = 0 then
    .fail .panic
  else
    let tag := data.get ⟨0, by omega⟩
    do
      let (length, payload_start) ← read_u32_be data 1
      let total := payload_start + length.val
      if htotal : total ≤ data.length then
        match tag.val with
        | 0 => do  -- Text
          let (payload, _) ← read_bytes data payload_start
          .ok (.Text payload, total)
        | 1 => do  -- Command
          if hps : payload_start < total then
            let cmd_tag := data.get ⟨payload_start, by omega⟩
            let cmd ← cmd_type_from_tag cmd_tag
            let (args, _) ← read_vec_list data (payload_start + 1)
            .ok (.Command cmd args, total)
          else
            .fail .panic
        | 2 => do  -- Error
          if hps : payload_start < total then
            let code_tag := data.get ⟨payload_start, by omega⟩
            let code ← error_code_from_tag code_tag
            let (detail, _) ← read_bytes data (payload_start + 1)
            .ok (.Error code detail, total)
          else
            .fail .panic
        | 3 => do  -- Heartbeat
          let (ts, _) ← read_u64_be data payload_start
          .ok (.Heartbeat ts, total)
        | _ => .fail .panic  -- InvalidTag
      else
        .fail .panic  -- NotEnoughData

end message_protocol
