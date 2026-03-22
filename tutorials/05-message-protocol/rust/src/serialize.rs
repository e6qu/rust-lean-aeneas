use crate::message::{CmdType, ErrorCode, Message};

/// Encode a `u32` as 4 big-endian bytes and append to `out`.
fn write_u32_be(val: u32, out: &mut Vec<u8>) {
    out.push(((val >> 24) & 0xFF) as u8);
    out.push(((val >> 16) & 0xFF) as u8);
    out.push(((val >> 8) & 0xFF) as u8);
    out.push((val & 0xFF) as u8);
}

/// Encode a `u64` as 8 big-endian bytes and append to `out`.
fn write_u64_be(val: u64, out: &mut Vec<u8>) {
    out.push(((val >> 56) & 0xFF) as u8);
    out.push(((val >> 48) & 0xFF) as u8);
    out.push(((val >> 40) & 0xFF) as u8);
    out.push(((val >> 32) & 0xFF) as u8);
    out.push(((val >> 24) & 0xFF) as u8);
    out.push(((val >> 16) & 0xFF) as u8);
    out.push(((val >> 8) & 0xFF) as u8);
    out.push((val & 0xFF) as u8);
}

/// Write a length-prefixed byte slice: [length:u32 BE][data].
fn serialize_bytes(data: &[u8], out: &mut Vec<u8>) {
    write_u32_be(data.len() as u32, out);
    let mut i: usize = 0;
    while i < data.len() {
        out.push(data[i]);
        i += 1;
    }
}

/// Write a list of byte vectors: [count:u32 BE] then each length-prefixed.
fn serialize_vec_list(vecs: &[Vec<u8>], out: &mut Vec<u8>) {
    write_u32_be(vecs.len() as u32, out);
    let mut i: usize = 0;
    while i < vecs.len() {
        serialize_bytes(&vecs[i], out);
        i += 1;
    }
}

/// Encode a `CmdType` as a single byte.
fn cmd_type_tag(cmd: &CmdType) -> u8 {
    match cmd {
        CmdType::Ping => 0,
        CmdType::Quit => 1,
        CmdType::Help => 2,
        CmdType::Run => 3,
    }
}

/// Encode an `ErrorCode` as a single byte.
fn error_code_tag(code: &ErrorCode) -> u8 {
    match code {
        ErrorCode::InvalidInput => 0,
        ErrorCode::NotFound => 1,
        ErrorCode::Internal => 2,
    }
}

/// Build the payload bytes for a message (everything after the TLV header).
fn build_payload(msg: &Message) -> Vec<u8> {
    let mut payload = Vec::new();
    match msg {
        Message::Text(data) => {
            serialize_bytes(data, &mut payload);
        }
        Message::Command(cmd, args) => {
            payload.push(cmd_type_tag(cmd));
            serialize_vec_list(args, &mut payload);
        }
        Message::Error(code, detail) => {
            payload.push(error_code_tag(code));
            serialize_bytes(detail, &mut payload);
        }
        Message::Heartbeat(ts) => {
            write_u64_be(*ts, &mut payload);
        }
    }
    payload
}

/// Tag byte for the message variant.
fn message_tag(msg: &Message) -> u8 {
    match msg {
        Message::Text(_) => 0,
        Message::Command(_, _) => 1,
        Message::Error(_, _) => 2,
        Message::Heartbeat(_) => 3,
    }
}

/// Serialize a `Message` into its TLV wire format:
/// `[tag:u8][length:u32 BE][payload]`.
pub fn serialize(msg: &Message) -> Vec<u8> {
    let mut out = Vec::new();
    let tag = message_tag(msg);
    let payload = build_payload(msg);
    out.push(tag);
    write_u32_be(payload.len() as u32, &mut out);
    let mut i: usize = 0;
    while i < payload.len() {
        out.push(payload[i]);
        i += 1;
    }
    out
}
