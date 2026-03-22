use message_protocol::message::*;
use message_protocol::serialize::serialize;
use message_protocol::deserialize::deserialize;
use message_protocol::framing::FrameAccumulator;

// ---------------------------------------------------------------------------
// Roundtrip tests: serialize then deserialize should recover the original
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_text() {
    let msg = Message::Text(b"hello world".to_vec());
    let bytes = serialize(&msg);
    let (decoded, consumed) = deserialize(&bytes).unwrap();
    assert_eq!(decoded, msg);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn roundtrip_text_empty() {
    let msg = Message::Text(Vec::new());
    let bytes = serialize(&msg);
    let (decoded, consumed) = deserialize(&bytes).unwrap();
    assert_eq!(decoded, msg);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn roundtrip_command_no_args() {
    let msg = Message::Command(CmdType::Ping, Vec::new());
    let bytes = serialize(&msg);
    let (decoded, consumed) = deserialize(&bytes).unwrap();
    assert_eq!(decoded, msg);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn roundtrip_command_with_args() {
    let msg = Message::Command(
        CmdType::Run,
        vec![b"arg1".to_vec(), b"arg2".to_vec(), b"arg3".to_vec()],
    );
    let bytes = serialize(&msg);
    let (decoded, consumed) = deserialize(&bytes).unwrap();
    assert_eq!(decoded, msg);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn roundtrip_error() {
    let msg = Message::Error(ErrorCode::NotFound, b"file missing".to_vec());
    let bytes = serialize(&msg);
    let (decoded, consumed) = deserialize(&bytes).unwrap();
    assert_eq!(decoded, msg);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn roundtrip_heartbeat() {
    let msg = Message::Heartbeat(0xDEADBEEF_CAFEBABE);
    let bytes = serialize(&msg);
    let (decoded, consumed) = deserialize(&bytes).unwrap();
    assert_eq!(decoded, msg);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn roundtrip_heartbeat_zero() {
    let msg = Message::Heartbeat(0);
    let bytes = serialize(&msg);
    let (decoded, consumed) = deserialize(&bytes).unwrap();
    assert_eq!(decoded, msg);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn roundtrip_all_cmd_types() {
    let cmds = [CmdType::Ping, CmdType::Quit, CmdType::Help, CmdType::Run];
    for cmd in &cmds {
        let msg = Message::Command(cmd.clone(), vec![b"test".to_vec()]);
        let bytes = serialize(&msg);
        let (decoded, _) = deserialize(&bytes).unwrap();
        assert_eq!(decoded, msg);
    }
}

#[test]
fn roundtrip_all_error_codes() {
    let codes = [ErrorCode::InvalidInput, ErrorCode::NotFound, ErrorCode::Internal];
    for code in &codes {
        let msg = Message::Error(code.clone(), b"detail".to_vec());
        let bytes = serialize(&msg);
        let (decoded, _) = deserialize(&bytes).unwrap();
        assert_eq!(decoded, msg);
    }
}

// ---------------------------------------------------------------------------
// Error detection tests
// ---------------------------------------------------------------------------

#[test]
fn invalid_tag_rejected() {
    let data = [0xFF, 0x00, 0x00, 0x00, 0x00]; // tag=255, length=0
    let result = deserialize(&data);
    assert_eq!(result, Err(ParseError::InvalidTag));
}

#[test]
fn truncated_data_rejected() {
    let msg = Message::Text(b"hello".to_vec());
    let bytes = serialize(&msg);
    // Feed only half
    let half = &bytes[..bytes.len() / 2];
    let result = deserialize(half);
    assert_eq!(result, Err(ParseError::NotEnoughData));
}

#[test]
fn empty_input_rejected() {
    let result = deserialize(&[]);
    assert_eq!(result, Err(ParseError::NotEnoughData));
}

// ---------------------------------------------------------------------------
// Framing tests
// ---------------------------------------------------------------------------

#[test]
fn framing_single_message() {
    let msg = Message::Text(b"framed".to_vec());
    let bytes = serialize(&msg);
    let mut acc = FrameAccumulator::new();
    let results = acc.feed(&bytes);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], Ok(msg));
}

#[test]
fn framing_multiple_messages_at_once() {
    let msgs = vec![
        Message::Text(b"one".to_vec()),
        Message::Heartbeat(42),
        Message::Error(ErrorCode::Internal, b"oops".to_vec()),
    ];
    let mut all_bytes = Vec::new();
    for m in &msgs {
        let b = serialize(m);
        all_bytes.extend_from_slice(&b);
    }
    let mut acc = FrameAccumulator::new();
    let results = acc.feed(&all_bytes);
    assert_eq!(results.len(), 3);
    for (i, m) in msgs.iter().enumerate() {
        assert_eq!(results[i], Ok(m.clone()));
    }
}

#[test]
fn framing_partial_reads() {
    let msg = Message::Command(CmdType::Help, vec![b"foo".to_vec()]);
    let bytes = serialize(&msg);

    let mut acc = FrameAccumulator::new();

    // Feed one byte at a time
    let mut i = 0;
    let mut all_results = Vec::new();
    while i < bytes.len() {
        let results = acc.feed(&bytes[i..i + 1]);
        all_results.extend(results);
        i += 1;
    }
    assert_eq!(all_results.len(), 1);
    assert_eq!(all_results[0], Ok(msg));
}

#[test]
fn framing_two_messages_split() {
    let msg1 = Message::Text(b"first".to_vec());
    let msg2 = Message::Heartbeat(999);
    let mut bytes = serialize(&msg1);
    bytes.extend_from_slice(&serialize(&msg2));

    let mut acc = FrameAccumulator::new();

    // Feed in two chunks splitting in the middle
    let mid = bytes.len() / 2;
    let r1 = acc.feed(&bytes[..mid]);
    let r2 = acc.feed(&bytes[mid..]);

    let mut all: Vec<_> = r1.into_iter().chain(r2.into_iter()).collect();
    // We should get exactly 2 messages
    assert_eq!(all.len(), 2);
    assert_eq!(all.remove(0), Ok(msg1));
    assert_eq!(all.remove(0), Ok(msg2));
}
