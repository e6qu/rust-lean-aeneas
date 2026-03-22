//! Layout engine: split and split_at for dividing rectangles.

use crate::geometry::Rect;

/// Direction for splitting a rectangle.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SplitDir {
    Horizontal,
    Vertical,
}

/// Split `area` into `count` equal parts along `dir`.
/// Integer division; the last piece gets the remainder pixels.
/// Returns an empty vec if count is 0.
pub fn split(area: Rect, dir: SplitDir, count: usize) -> Vec<Rect> {
    if count == 0 {
        return Vec::new();
    }

    let mut result: Vec<Rect> = Vec::new();
    let total = match dir {
        SplitDir::Horizontal => area.width,
        SplitDir::Vertical => area.height,
    };

    let piece_size = total / (count as u16);
    let mut offset: u16 = 0;
    let mut i: usize = 0;

    while i < count {
        let size = if i == count - 1 {
            // Last piece gets the remainder
            total.wrapping_sub(offset)
        } else {
            piece_size
        };

        let r = match dir {
            SplitDir::Horizontal => Rect {
                x: area.x.wrapping_add(offset),
                y: area.y,
                width: size,
                height: area.height,
            },
            SplitDir::Vertical => Rect {
                x: area.x,
                y: area.y.wrapping_add(offset),
                width: area.width,
                height: size,
            },
        };

        result.push(r);
        offset = offset.wrapping_add(piece_size);
        i += 1;
    }

    result
}

/// Split `area` into two pieces at `offset` pixels from the start along `dir`.
/// If offset is beyond the area dimension, the second piece has zero size.
pub fn split_at(area: Rect, dir: SplitDir, offset: u16) -> (Rect, Rect) {
    match dir {
        SplitDir::Horizontal => {
            let clamped = if offset > area.width {
                area.width
            } else {
                offset
            };
            let left = Rect {
                x: area.x,
                y: area.y,
                width: clamped,
                height: area.height,
            };
            let right = Rect {
                x: area.x.wrapping_add(clamped),
                y: area.y,
                width: area.width.wrapping_sub(clamped),
                height: area.height,
            };
            (left, right)
        }
        SplitDir::Vertical => {
            let clamped = if offset > area.height {
                area.height
            } else {
                offset
            };
            let top = Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: clamped,
            };
            let bottom = Rect {
                x: area.x,
                y: area.y.wrapping_add(clamped),
                width: area.width,
                height: area.height.wrapping_sub(clamped),
            };
            (top, bottom)
        }
    }
}
