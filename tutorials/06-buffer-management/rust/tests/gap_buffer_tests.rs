use buffer_management::gap_buffer::GapBuffer;

#[test]
fn test_new_gap_buffer() {
    let gb = GapBuffer::new(10);
    assert_eq!(gb.content_len(), 0);
    assert_eq!(gb.cursor_pos(), 0);
    assert_eq!(gb.buffer_len(), 10);
}

#[test]
fn test_insert_single() {
    let mut gb = GapBuffer::new(10);
    gb.insert(b'a');
    assert_eq!(gb.content_len(), 1);
    assert_eq!(gb.cursor_pos(), 1);
    assert_eq!(gb.to_vec(), vec![b'a']);
}

#[test]
fn test_insert_multiple() {
    let mut gb = GapBuffer::new(10);
    gb.insert(b'h');
    gb.insert(b'i');
    assert_eq!(gb.to_vec(), vec![b'h', b'i']);
    assert_eq!(gb.content_len(), 2);
}

#[test]
fn test_insert_at_capacity() {
    let mut gb = GapBuffer::new(3);
    gb.insert(b'a');
    gb.insert(b'b');
    gb.insert(b'c');
    // Buffer is full now, insert should do nothing
    gb.insert(b'd');
    assert_eq!(gb.content_len(), 3);
    assert_eq!(gb.to_vec(), vec![b'a', b'b', b'c']);
}

#[test]
fn test_delete_before() {
    let mut gb = GapBuffer::new(10);
    gb.insert(b'a');
    gb.insert(b'b');
    gb.insert(b'c');
    assert!(gb.delete_before());
    assert_eq!(gb.to_vec(), vec![b'a', b'b']);
    assert_eq!(gb.cursor_pos(), 2);
}

#[test]
fn test_delete_before_at_start() {
    let mut gb = GapBuffer::new(10);
    assert!(!gb.delete_before());
}

#[test]
fn test_delete_after() {
    let mut gb = GapBuffer::new(10);
    gb.insert(b'a');
    gb.insert(b'b');
    gb.insert(b'c');
    // Move cursor to the beginning
    gb.move_left();
    gb.move_left();
    gb.move_left();
    assert!(gb.delete_after());
    assert_eq!(gb.to_vec(), vec![b'b', b'c']);
}

#[test]
fn test_delete_after_at_end() {
    let mut gb = GapBuffer::new(10);
    gb.insert(b'a');
    assert!(!gb.delete_after());
}

#[test]
fn test_move_left() {
    let mut gb = GapBuffer::new(10);
    gb.insert(b'a');
    gb.insert(b'b');
    assert_eq!(gb.cursor_pos(), 2);
    assert!(gb.move_left());
    assert_eq!(gb.cursor_pos(), 1);
    // Content should be preserved
    assert_eq!(gb.to_vec(), vec![b'a', b'b']);
}

#[test]
fn test_move_left_at_start() {
    let mut gb = GapBuffer::new(10);
    assert!(!gb.move_left());
}

#[test]
fn test_move_right() {
    let mut gb = GapBuffer::new(10);
    gb.insert(b'a');
    gb.insert(b'b');
    gb.move_left();
    gb.move_left();
    assert_eq!(gb.cursor_pos(), 0);
    assert!(gb.move_right());
    assert_eq!(gb.cursor_pos(), 1);
    assert_eq!(gb.to_vec(), vec![b'a', b'b']);
}

#[test]
fn test_move_right_at_end() {
    let mut gb = GapBuffer::new(10);
    gb.insert(b'a');
    assert!(!gb.move_right());
}

#[test]
fn test_insert_in_middle() {
    let mut gb = GapBuffer::new(10);
    gb.insert(b'a');
    gb.insert(b'c');
    gb.move_left(); // cursor between 'a' and 'c'
    gb.insert(b'b');
    assert_eq!(gb.to_vec(), vec![b'a', b'b', b'c']);
}

#[test]
fn test_content_preserved_through_moves() {
    let mut gb = GapBuffer::new(20);
    // Insert "hello"
    gb.insert(b'h');
    gb.insert(b'e');
    gb.insert(b'l');
    gb.insert(b'l');
    gb.insert(b'o');

    let original = gb.to_vec();

    // Move all the way left
    gb.move_left();
    gb.move_left();
    gb.move_left();
    gb.move_left();
    gb.move_left();
    assert_eq!(gb.to_vec(), original);

    // Move all the way right
    gb.move_right();
    gb.move_right();
    gb.move_right();
    gb.move_right();
    gb.move_right();
    assert_eq!(gb.to_vec(), original);
}

#[test]
fn test_gap_invariant() {
    let mut gb = GapBuffer::new(10);
    gb.insert(b'x');
    gb.insert(b'y');
    assert!(gb.gap_start() <= gb.gap_end());
    assert!(gb.gap_end() <= gb.buffer_len());

    gb.move_left();
    assert!(gb.gap_start() <= gb.gap_end());
    assert!(gb.gap_end() <= gb.buffer_len());

    gb.delete_before();
    assert!(gb.gap_start() <= gb.gap_end());
    assert!(gb.gap_end() <= gb.buffer_len());
}
