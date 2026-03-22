use buffer_management::ring_buffer::RingBuffer;

#[test]
fn test_new_buffer_is_empty() {
    let rb: RingBuffer<u32> = RingBuffer::new(4);
    assert!(rb.is_empty());
    assert!(!rb.is_full());
    assert_eq!(rb.len(), 0);
    assert_eq!(rb.capacity(), 4);
}

#[test]
fn test_push_pop_single() {
    let mut rb: RingBuffer<u32> = RingBuffer::new(4);
    assert!(rb.push(42));
    assert_eq!(rb.len(), 1);
    let (found, val) = rb.pop();
    assert!(found);
    assert_eq!(val, 42);
    assert!(rb.is_empty());
}

#[test]
fn test_push_pop_roundtrip() {
    let mut rb: RingBuffer<u32> = RingBuffer::new(4);
    assert!(rb.push(10));
    assert!(rb.push(20));
    assert!(rb.push(30));
    assert_eq!(rb.len(), 3);

    let (ok1, v1) = rb.pop();
    let (ok2, v2) = rb.pop();
    let (ok3, v3) = rb.pop();
    assert!(ok1);
    assert!(ok2);
    assert!(ok3);
    assert_eq!(v1, 10);
    assert_eq!(v2, 20);
    assert_eq!(v3, 30);
}

#[test]
fn test_fifo_order() {
    let mut rb: RingBuffer<u32> = RingBuffer::new(8);
    let mut i: u32 = 0;
    while i < 5 {
        rb.push(i * 10);
        i += 1;
    }

    let mut j: u32 = 0;
    while j < 5 {
        let (found, val) = rb.pop();
        assert!(found);
        assert_eq!(val, j * 10);
        j += 1;
    }
}

#[test]
fn test_full_buffer_rejects_push() {
    let mut rb: RingBuffer<u32> = RingBuffer::new(2);
    assert!(rb.push(1));
    assert!(rb.push(2));
    assert!(rb.is_full());
    assert!(!rb.push(3)); // should fail
    assert_eq!(rb.len(), 2);
}

#[test]
fn test_empty_buffer_pop_returns_false() {
    let mut rb: RingBuffer<u32> = RingBuffer::new(4);
    let (found, _val) = rb.pop();
    assert!(!found);
}

#[test]
fn test_peek() {
    let mut rb: RingBuffer<u32> = RingBuffer::new(4);
    let (found, _) = rb.peek();
    assert!(!found);

    rb.push(99);
    let (found, val) = rb.peek();
    assert!(found);
    assert_eq!(val, 99);
    // peek should not remove the element
    assert_eq!(rb.len(), 1);
}

#[test]
fn test_wraparound() {
    // Fill and drain to move head/tail past the end, then fill again
    let mut rb: RingBuffer<u32> = RingBuffer::new(3);
    rb.push(1);
    rb.push(2);
    rb.push(3);
    rb.pop();
    rb.pop();
    rb.pop();
    // head and tail are now at index 3 % 3 = 0, but let's push more
    // Actually they wrapped: head=0, tail=0 after full drain of capacity 3
    rb.push(10);
    rb.push(20);
    let (ok1, v1) = rb.pop();
    let (ok2, v2) = rb.pop();
    assert!(ok1);
    assert!(ok2);
    assert_eq!(v1, 10);
    assert_eq!(v2, 20);
}

#[test]
fn test_interleaved_push_pop() {
    let mut rb: RingBuffer<u32> = RingBuffer::new(3);
    rb.push(1);
    rb.push(2);
    let (_, v) = rb.pop();
    assert_eq!(v, 1);
    rb.push(3);
    rb.push(4);
    // Now buffer has [2, 3, 4], wrapped around
    let (_, v1) = rb.pop();
    let (_, v2) = rb.pop();
    let (_, v3) = rb.pop();
    assert_eq!(v1, 2);
    assert_eq!(v2, 3);
    assert_eq!(v3, 4);
}
