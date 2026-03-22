use llm_client_core::streaming::*;

#[test]
fn test_chunk_response_basic() {
    let data = b"Hello, World!";
    let chunks = chunk_response(data, 5);
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], b"Hello");
    assert_eq!(chunks[1], b", Wor");
    assert_eq!(chunks[2], b"ld!");
}

#[test]
fn test_chunk_response_exact_division() {
    let data = b"abcdef";
    let chunks = chunk_response(data, 3);
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0], b"abc");
    assert_eq!(chunks[1], b"def");
}

#[test]
fn test_chunk_response_single_chunk() {
    let data = b"hi";
    let chunks = chunk_response(data, 100);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], b"hi");
}

#[test]
fn test_chunk_response_empty() {
    let data: &[u8] = b"";
    let chunks = chunk_response(data, 5);
    assert_eq!(chunks.len(), 0);
}

#[test]
fn test_chunk_response_zero_size() {
    let data = b"hello";
    let chunks = chunk_response(data, 0);
    assert_eq!(chunks.len(), 0);
}

#[test]
fn test_accumulator_roundtrip() {
    let data = b"Hello, World!";
    let chunks = chunk_response(data, 5);

    let mut acc = StreamAccumulator::new();
    for chunk in &chunks {
        acc.add_chunk(chunk);
    }

    assert_eq!(acc.get_accumulated(), data);
    assert!(acc.is_complete(data.len()));
    assert_eq!(acc.chunks.len(), chunks.len());
}

#[test]
fn test_accumulator_is_complete() {
    let mut acc = StreamAccumulator::new();
    assert!(!acc.is_complete(10));

    acc.add_chunk(b"hello");
    assert!(!acc.is_complete(10));
    assert!(acc.is_complete(5));
    assert!(acc.is_complete(3));

    acc.add_chunk(b"world");
    assert!(acc.is_complete(10));
}

#[test]
fn test_accumulator_empty() {
    let acc = StreamAccumulator::new();
    assert_eq!(acc.get_accumulated(), b"");
    assert!(acc.is_complete(0));
    assert_eq!(acc.chunks.len(), 0);
}

#[test]
fn test_chunk_and_reassemble_large() {
    // Test with a larger payload to verify correctness
    let mut data: Vec<u8> = Vec::new();
    let mut i: u8 = 0;
    while (i as usize) < 255 {
        data.push(i);
        i += 1;
    }

    let chunks = chunk_response(&data, 7);
    let mut acc = StreamAccumulator::new();
    for chunk in &chunks {
        acc.add_chunk(chunk);
    }

    assert_eq!(acc.get_accumulated(), &data[..]);
}
