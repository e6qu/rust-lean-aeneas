//! Geometric primitives: Rect, Position, Cell.

/// A rectangle in terminal coordinates.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// A single position in the terminal grid.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

/// A rendered cell: position, character byte, and style byte.
#[derive(Clone, PartialEq, Debug)]
pub struct Cell {
    pub pos: Position,
    pub ch: u8,
    pub style: u8,
}

impl Rect {
    /// Create a new rectangle.
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Rect { x, y, width, height }
    }

    /// Check whether `pos` lies within this rectangle.
    pub fn contains(&self, pos: Position) -> bool {
        pos.x >= self.x
            && pos.x < self.x.wrapping_add(self.width)
            && pos.y >= self.y
            && pos.y < self.y.wrapping_add(self.height)
    }

    /// Total area as u32 to avoid u16 overflow.
    pub fn area(&self) -> u32 {
        (self.width as u32) * (self.height as u32)
    }

    /// Check whether two rectangles overlap.
    pub fn intersects(&self, other: &Rect) -> bool {
        let self_right = self.x.wrapping_add(self.width);
        let self_bottom = self.y.wrapping_add(self.height);
        let other_right = other.x.wrapping_add(other.width);
        let other_bottom = other.y.wrapping_add(other.height);

        self.x < other_right
            && other.x < self_right
            && self.y < other_bottom
            && other.y < self_bottom
    }

    /// Shrink the rectangle inward by `margin` on each side.
    /// Returns a zero-sized rect if margin is too large.
    pub fn inner(&self, margin: u16) -> Rect {
        let double = margin.wrapping_mul(2);
        if self.width <= double || self.height <= double {
            return Rect { x: self.x, y: self.y, width: 0, height: 0 };
        }
        Rect {
            x: self.x.wrapping_add(margin),
            y: self.y.wrapping_add(margin),
            width: self.width.wrapping_sub(double),
            height: self.height.wrapping_sub(double),
        }
    }
}
