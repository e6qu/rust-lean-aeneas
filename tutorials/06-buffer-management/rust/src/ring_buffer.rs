//! A fixed-capacity ring buffer (circular queue).
//!
//! Uses pre-allocated storage with modular arithmetic for index wrapping.
//! Returns `(bool, T)` tuples instead of `Option<T>` to simplify Aeneas translation.

/// A fixed-capacity circular buffer that stores elements in FIFO order.
///
/// Internally, a `Vec<T>` is pre-allocated with `T::default()` values.
/// `head` is the next read position, `tail` is the next write position,
/// and `len` tracks the current number of stored elements.
pub struct RingBuffer<T: Clone + Default> {
    data: Vec<T>,
    capacity: usize,
    head: usize,
    tail: usize,
    len: usize,
}

impl<T: Clone + Default> RingBuffer<T> {
    /// Create a new ring buffer with the given capacity.
    ///
    /// Pre-allocates `capacity` slots filled with `T::default()`.
    /// A capacity of 0 creates a buffer that is permanently full and empty.
    pub fn new(capacity: usize) -> Self {
        // Build the data vector using an explicit while loop (Aeneas-friendly)
        let mut data: Vec<T> = Vec::new();
        let mut i: usize = 0;
        while i < capacity {
            data.push(T::default());
            i += 1;
        }
        RingBuffer {
            data,
            capacity,
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    /// Push an item into the buffer. Returns `true` on success, `false` if full.
    pub fn push(&mut self, item: T) -> bool {
        if self.len == self.capacity {
            return false;
        }
        self.data[self.tail] = item;
        self.tail = (self.tail + 1) % self.capacity;
        self.len += 1;
        true
    }

    /// Pop an item from the front of the buffer.
    ///
    /// Returns `(true, value)` if an element was available, or
    /// `(false, T::default())` if the buffer was empty.
    /// We use a tuple instead of `Option<T>` to simplify Aeneas translation.
    pub fn pop(&mut self) -> (bool, T) {
        if self.len == 0 {
            return (false, T::default());
        }
        let item = self.data[self.head].clone();
        self.data[self.head] = T::default();
        self.head = (self.head + 1) % self.capacity;
        self.len -= 1;
        (true, item)
    }

    /// Peek at the front element without removing it.
    ///
    /// Returns `(true, value)` if an element exists, or
    /// `(false, T::default())` if the buffer is empty.
    pub fn peek(&self) -> (bool, T) {
        if self.len == 0 {
            return (false, T::default());
        }
        (true, self.data[self.head].clone())
    }

    /// Returns `true` if the buffer has no remaining capacity.
    pub fn is_full(&self) -> bool {
        self.len == self.capacity
    }

    /// Returns `true` if the buffer contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of elements currently in the buffer.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the total capacity of the buffer.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
