use buffer_management::input_buffer::InputBuffer;

#[test]
fn test_insert_and_get_content() {
    let mut ib = InputBuffer::new(20);
    ib.insert_char(b'h');
    ib.insert_char(b'i');
    assert_eq!(ib.get_content(), b"hi");
}

#[test]
fn test_backspace() {
    let mut ib = InputBuffer::new(20);
    ib.insert_char(b'a');
    ib.insert_char(b'b');
    assert!(ib.backspace());
    assert_eq!(ib.get_content(), b"a");
}

#[test]
fn test_backspace_empty() {
    let mut ib = InputBuffer::new(20);
    assert!(!ib.backspace());
}

#[test]
fn test_delete_word_single_word() {
    let mut ib = InputBuffer::new(40);
    // Type "hello"
    let word = b"hello";
    let mut i: usize = 0;
    while i < word.len() {
        ib.insert_char(word[i]);
        i += 1;
    }
    assert!(ib.delete_word());
    assert_eq!(ib.get_content(), b"");
}

#[test]
fn test_delete_word_with_space() {
    let mut ib = InputBuffer::new(40);
    // Type "hello world"
    let text = b"hello world";
    let mut i: usize = 0;
    while i < text.len() {
        ib.insert_char(text[i]);
        i += 1;
    }
    assert!(ib.delete_word());
    assert_eq!(ib.get_content(), b"hello ");
}

#[test]
fn test_delete_word_trailing_spaces() {
    let mut ib = InputBuffer::new(40);
    // Type "hello   " (trailing spaces)
    let text = b"hello   ";
    let mut i: usize = 0;
    while i < text.len() {
        ib.insert_char(text[i]);
        i += 1;
    }
    assert!(ib.delete_word());
    // Should delete the spaces and then "hello"
    assert_eq!(ib.get_content(), b"");
}

#[test]
fn test_delete_word_empty() {
    let mut ib = InputBuffer::new(20);
    assert!(!ib.delete_word());
}

#[test]
fn test_move_cursor_left_right() {
    let mut ib = InputBuffer::new(20);
    ib.insert_char(b'a');
    ib.insert_char(b'b');
    assert_eq!(ib.cursor_position(), 2);

    assert!(ib.move_cursor_left());
    assert_eq!(ib.cursor_position(), 1);

    assert!(ib.move_cursor_right());
    assert_eq!(ib.cursor_position(), 2);
}

#[test]
fn test_move_to_start() {
    let mut ib = InputBuffer::new(20);
    ib.insert_char(b'a');
    ib.insert_char(b'b');
    ib.insert_char(b'c');
    ib.move_to_start();
    assert_eq!(ib.cursor_position(), 0);
    // Content preserved
    assert_eq!(ib.get_content(), b"abc");
}

#[test]
fn test_move_to_end() {
    let mut ib = InputBuffer::new(20);
    ib.insert_char(b'a');
    ib.insert_char(b'b');
    ib.insert_char(b'c');
    ib.move_to_start();
    ib.move_to_end();
    assert_eq!(ib.cursor_position(), 3);
    assert_eq!(ib.get_content(), b"abc");
}

#[test]
fn test_insert_after_move_to_start() {
    let mut ib = InputBuffer::new(20);
    ib.insert_char(b'b');
    ib.insert_char(b'c');
    ib.move_to_start();
    ib.insert_char(b'a');
    assert_eq!(ib.get_content(), b"abc");
}

#[test]
fn test_delete_word_multiple() {
    let mut ib = InputBuffer::new(60);
    // Type "one two three"
    let text = b"one two three";
    let mut i: usize = 0;
    while i < text.len() {
        ib.insert_char(text[i]);
        i += 1;
    }
    ib.delete_word(); // removes "three"
    assert_eq!(ib.get_content(), b"one two ");
    ib.delete_word(); // removes " two" (space then word)
    // After deleting trailing space + "two", we should have "one"
    // Actually: "one two " -> delete_word skips spaces (the trailing space),
    // then deletes "two" -> "one "
    // Wait, let me re-check: content is "one two ", cursor at end
    // Phase 1: skip spaces backward: delete ' ' -> "one two", cursor at 7
    // Phase 2: delete non-spaces: delete 'o','w','t' -> "one ", cursor at 4
    assert_eq!(ib.get_content(), b"one ");
}
