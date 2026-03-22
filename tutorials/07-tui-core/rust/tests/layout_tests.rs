//! Tests for the layout engine.

use tui_core::geometry::Rect;
use tui_core::layout::{SplitDir, split, split_at};

#[test]
fn split_horizontal_even() {
    let area = Rect::new(0, 0, 80, 24);
    let parts = split(area, SplitDir::Horizontal, 4);
    assert_eq!(parts.len(), 4);
    assert_eq!(parts[0], Rect::new(0, 0, 20, 24));
    assert_eq!(parts[1], Rect::new(20, 0, 20, 24));
    assert_eq!(parts[2], Rect::new(40, 0, 20, 24));
    assert_eq!(parts[3], Rect::new(60, 0, 20, 24));
}

#[test]
fn split_horizontal_remainder() {
    let area = Rect::new(0, 0, 10, 5);
    let parts = split(area, SplitDir::Horizontal, 3);
    assert_eq!(parts.len(), 3);
    // 10 / 3 = 3 per piece, last gets 10 - 6 = 4
    assert_eq!(parts[0].width, 3);
    assert_eq!(parts[1].width, 3);
    assert_eq!(parts[2].width, 4);
    // Total width should equal original
    let total_width: u16 = parts[0].width + parts[1].width + parts[2].width;
    assert_eq!(total_width, 10);
}

#[test]
fn split_vertical() {
    let area = Rect::new(5, 5, 40, 20);
    let parts = split(area, SplitDir::Vertical, 2);
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], Rect::new(5, 5, 40, 10));
    assert_eq!(parts[1], Rect::new(5, 15, 40, 10));
}

#[test]
fn split_single() {
    let area = Rect::new(0, 0, 80, 24);
    let parts = split(area, SplitDir::Horizontal, 1);
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0], area);
}

#[test]
fn split_zero_count() {
    let area = Rect::new(0, 0, 80, 24);
    let parts = split(area, SplitDir::Horizontal, 0);
    assert!(parts.is_empty());
}

#[test]
fn split_no_overlap() {
    let area = Rect::new(0, 0, 100, 50);
    let parts = split(area, SplitDir::Horizontal, 7);
    let mut i: usize = 0;
    while i < parts.len() {
        let mut j = i + 1;
        while j < parts.len() {
            assert!(
                !parts[i].intersects(&parts[j]),
                "parts[{}] and parts[{}] overlap",
                i,
                j
            );
            j += 1;
        }
        i += 1;
    }
}

#[test]
fn split_covers_width() {
    let area = Rect::new(0, 0, 100, 50);
    let parts = split(area, SplitDir::Horizontal, 7);
    let mut total: u16 = 0;
    let mut k: usize = 0;
    while k < parts.len() {
        total += parts[k].width;
        k += 1;
    }
    assert_eq!(total, area.width);
}

#[test]
fn split_at_horizontal() {
    let area = Rect::new(0, 0, 80, 24);
    let (left, right) = split_at(area, SplitDir::Horizontal, 30);
    assert_eq!(left, Rect::new(0, 0, 30, 24));
    assert_eq!(right, Rect::new(30, 0, 50, 24));
}

#[test]
fn split_at_vertical() {
    let area = Rect::new(10, 10, 40, 20);
    let (top, bottom) = split_at(area, SplitDir::Vertical, 5);
    assert_eq!(top, Rect::new(10, 10, 40, 5));
    assert_eq!(bottom, Rect::new(10, 15, 40, 15));
}

#[test]
fn split_at_beyond_boundary() {
    let area = Rect::new(0, 0, 80, 24);
    let (left, right) = split_at(area, SplitDir::Horizontal, 100);
    assert_eq!(left.width, 80);
    assert_eq!(right.width, 0);
}

#[test]
fn split_at_partition() {
    // left.width + right.width == area.width
    let area = Rect::new(0, 0, 80, 24);
    let (left, right) = split_at(area, SplitDir::Horizontal, 30);
    assert_eq!(left.width + right.width, area.width);
}
