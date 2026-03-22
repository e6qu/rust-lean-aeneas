use crate::message::{CmdType, ErrorCode, Message, ParseError};

/// Read a big-endian `u32` from `data` at `offset`.
/// Returns `(value, new_offset)`.
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

/// Read a big-endian `u64` from `data` at `offset`.
/// Returns `(value, new_offset)`.
fn read_u64_be(data: &[u8], offset: usize) -> Result<(u64, usize), ParseError> {
    if offset + 8 > data.len() {
        return Err(ParseError::NotEnoughData);
    }
    let val = (data[offset] as u64) << 56
        | (data[offset + 1] as u64) << 48
        | (data[offset + 2] as u64) << 40
        | (data[offset + 3] as u64) << 32
        | (data[offset + 4] as u64) << 24
        | (data[offset + 5] as u64) << 16
        | (data[offset + 6] as u64) << 8
        | (data[offset + 7] as u64);
    Ok((val, offset + 8))
}

/// Read a length-prefixed byte vector from `data` at `offset`.
/// Returns `(bytes, new_offset)`.
fn read_bytes(data: &[u8], offset: usize) -> Result<(Vec<u8>, usize), ParseError> {
    let (len, off) = read_u32_be(data, offset)?;
    let len = len as usize;
    if off + len > data.len() {
        return Err(ParseError::NotEnoughData);
    }
    let mut result = Vec::new();
    let mut i: usize = 0;
    while i < len {
        result.push(data[off + i]);
        i += 1;
    }
    Ok((result, off + len))
}

/// Read a list of length-prefixed byte vectors.
/// Returns `(vec_list, new_offset)`.
fn read_vec_list(data: &[u8], offset: usize) -> Result<(Vec<Vec<u8>>, usize), ParseError> {
    let (count, mut off) = read_u32_be(data, offset)?;
    let count = count as usize;
    let mut result: Vec<Vec<u8>> = Vec::new();
    let mut i: usize = 0;
    while i < count {
        let (bytes, new_off) = read_bytes(data, off)?;
        result.push(bytes);
        off = new_off;
        i += 1;
    }
    Ok((result, off))
}

/// Decode a `CmdType` from a tag byte.
fn cmd_type_from_tag(tag: u8) -> Result<CmdType, ParseError> {
    match tag {
        0 => Ok(CmdType::Ping),
        1 => Ok(CmdType::Quit),
        2 => Ok(CmdType::Help),
        3 => Ok(CmdType::Run),
        _ => Err(ParseError::Malformed),
    }
}

/// Decode an `ErrorCode` from a tag byte.
fn error_code_from_tag(tag: u8) -> Result<ErrorCode, ParseError> {
    match tag {
        0 => Ok(ErrorCode::InvalidInput),
        1 => Ok(ErrorCode::NotFound),
        2 => Ok(ErrorCode::Internal),
        _ => Err(ParseError::Malformed),
    }
}

/// Deserialize a `Message` from TLV wire format.
///
/// Returns `(message, bytes_consumed)` on success. The `bytes_consumed` value
/// includes the tag byte, the 4-byte length, and the payload.
pub fn deserialize(data: &[u8]) -> Result<(Message, usize), ParseError> {
    if data.is_empty() {
        return Err(ParseError::NotEnoughData);
    }

    let tag = data[0];
    let (length, payload_start) = read_u32_be(data, 1)?;
    let length = length as usize;
    let total = payload_start + length;

    if total > data.len() {
        return Err(ParseError::NotEnoughData);
    }

    match tag {
        // Text
        0 => {
            let (payload, _) = read_bytes(data, payload_start)?;
            Ok((Message::Text(payload), total))
        }
        // Command
        1 => {
            if payload_start >= total {
                return Err(ParseError::Malformed);
            }
            let cmd_tag = data[payload_start];
            let cmd = cmd_type_from_tag(cmd_tag)?;
            let (args, _) = read_vec_list(data, payload_start + 1)?;
            Ok((Message::Command(cmd, args), total))
        }
        // Error
        2 => {
            if payload_start >= total {
                return Err(ParseError::Malformed);
            }
            let code_tag = data[payload_start];
            let code = error_code_from_tag(code_tag)?;
            let (detail, _) = read_bytes(data, payload_start + 1)?;
            Ok((Message::Error(code, detail), total))
        }
        // Heartbeat
        3 => {
            let (ts, _) = read_u64_be(data, payload_start)?;
            Ok((Message::Heartbeat(ts), total))
        }
        _ => Err(ParseError::InvalidTag),
    }
}
