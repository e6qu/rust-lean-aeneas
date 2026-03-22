use crate::message_types::ToolCallInfo;

/// Why the model stopped generating.
#[derive(Clone, PartialEq, Debug)]
pub enum FinishReason {
    Stop,
    Length,
    ToolUse,
}

/// Token usage statistics.
#[derive(Clone, PartialEq, Debug)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

/// A single piece of response content.
#[derive(Clone, PartialEq, Debug)]
pub enum ResponseContent {
    Text(Vec<u8>),
    ToolUse(ToolCallInfo),
}

/// A parsed LLM response.
#[derive(Clone, PartialEq, Debug)]
pub struct Response {
    pub content: Vec<ResponseContent>,
    pub finish_reason: FinishReason,
    pub usage: Usage,
}

/// Errors from parsing a response.
#[derive(Clone, PartialEq, Debug)]
pub enum ResponseParseError {
    InvalidFormat,
    MissingField,
    InvalidFinishReason,
}

/// Read a big-endian u32 from a byte slice at the given offset.
/// Returns the value and the new offset, or an error.
fn read_u32(data: &[u8], offset: usize) -> Result<(u32, usize), ResponseParseError> {
    if offset + 4 > data.len() {
        return Err(ResponseParseError::InvalidFormat);
    }
    let val = ((data[offset] as u32) << 24)
        | ((data[offset + 1] as u32) << 16)
        | ((data[offset + 2] as u32) << 8)
        | (data[offset + 3] as u32);
    Ok((val, offset + 4))
}

/// Read a length-prefixed byte vector from data at offset.
/// Format: [len:u32 BE][bytes:len]
fn read_bytes(data: &[u8], offset: usize) -> Result<(Vec<u8>, usize), ResponseParseError> {
    let (len, off) = read_u32(data, offset)?;
    let len = len as usize;
    if off + len > data.len() {
        return Err(ResponseParseError::InvalidFormat);
    }
    let mut result: Vec<u8> = Vec::new();
    let mut i: usize = 0;
    while i < len {
        result.push(data[off + i]);
        i += 1;
    }
    Ok((result, off + len))
}

/// Parse a response from a simplified length-prefixed binary format.
///
/// Format:
///   [finish_reason: u8]
///   [prompt_tokens: u32 BE]
///   [completion_tokens: u32 BE]
///   [content_count: u32 BE]
///   For each content item:
///     [content_type: u8] -- 0 = Text, 1 = ToolUse
///     If Text: [text: length-prefixed bytes]
///     If ToolUse: [id: lp bytes][function_name: lp bytes][arguments: lp bytes]
///
/// This is NOT JSON -- it is a fixed binary layout chosen for verifiability.
pub fn parse_response(data: &[u8]) -> Result<Response, ResponseParseError> {
    if data.is_empty() {
        return Err(ResponseParseError::InvalidFormat);
    }

    // Parse finish reason
    let finish_reason = match data[0] {
        0 => FinishReason::Stop,
        1 => FinishReason::Length,
        2 => FinishReason::ToolUse,
        _ => return Err(ResponseParseError::InvalidFinishReason),
    };

    let mut offset: usize = 1;

    // Parse usage
    let (prompt_tokens, off) = read_u32(data, offset)?;
    offset = off;
    let (completion_tokens, off) = read_u32(data, offset)?;
    offset = off;

    // Parse content items
    let (content_count, off) = read_u32(data, offset)?;
    offset = off;

    let mut content: Vec<ResponseContent> = Vec::new();
    let mut i: u32 = 0;
    while i < content_count {
        if offset >= data.len() {
            return Err(ResponseParseError::MissingField);
        }
        let content_type = data[offset];
        offset += 1;

        match content_type {
            0 => {
                // Text
                let (text, off) = read_bytes(data, offset)?;
                offset = off;
                content.push(ResponseContent::Text(text));
            }
            1 => {
                // ToolUse
                let (id, off) = read_bytes(data, offset)?;
                offset = off;
                let (function_name, off) = read_bytes(data, offset)?;
                offset = off;
                let (arguments, off) = read_bytes(data, offset)?;
                offset = off;
                content.push(ResponseContent::ToolUse(ToolCallInfo {
                    id,
                    function_name,
                    arguments,
                }));
            }
            _ => return Err(ResponseParseError::InvalidFormat),
        }

        i += 1;
    }

    Ok(Response {
        content,
        finish_reason,
        usage: Usage {
            prompt_tokens,
            completion_tokens,
        },
    })
}
