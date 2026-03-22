-- MessageProtocol/Types.lean
-- Simulated Aeneas output: inductive types for the message protocol.
import Aeneas

open Primitives

namespace message_protocol

/-- Command sub-types. -/
inductive CmdType where
  | Ping
  | Quit
  | Help
  | Run
deriving DecidableEq, Repr

/-- Error code sub-types. -/
inductive ErrorCode where
  | InvalidInput
  | NotFound
  | Internal
deriving DecidableEq, Repr

/-- Application-level message variants. -/
inductive Message where
  | Text (payload : List U8)
  | Command (cmd : CmdType) (args : List (List U8))
  | Error (code : ErrorCode) (detail : List U8)
  | Heartbeat (ts : U64)
deriving DecidableEq, Repr

/-- Parse errors. -/
inductive ParseError where
  | NotEnoughData
  | InvalidTag
  | InvalidLength
  | Malformed
deriving DecidableEq, Repr

/-- Frame accumulator state. -/
structure FrameAccumulator where
  buffer : List U8

end message_protocol
