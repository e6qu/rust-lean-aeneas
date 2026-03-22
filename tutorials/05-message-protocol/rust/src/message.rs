/// Command sub-types for the protocol.
#[derive(Clone, PartialEq, Debug)]
pub enum CmdType {
    Ping,
    Quit,
    Help,
    Run,
}

/// Error codes returned in error messages.
#[derive(Clone, PartialEq, Debug)]
pub enum ErrorCode {
    InvalidInput,
    NotFound,
    Internal,
}

/// Application-level message variants using TLV encoding.
///
/// We use `Vec<u8>` rather than `String` so the representation stays
/// Aeneas-friendly — no UTF-8 invariants to reason about.
#[derive(Clone, PartialEq, Debug)]
pub enum Message {
    Text(Vec<u8>),
    Command(CmdType, Vec<Vec<u8>>),
    Error(ErrorCode, Vec<u8>),
    Heartbeat(u64),
}

/// Errors that can occur when deserializing a byte stream.
#[derive(Clone, PartialEq, Debug)]
pub enum ParseError {
    NotEnoughData,
    InvalidTag,
    InvalidLength,
    Malformed,
}
