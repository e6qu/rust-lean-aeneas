/// Accumulates streaming response chunks into a complete response.
///
/// Chunks are stored individually and also concatenated into `accumulated`
/// for quick access to the full response so far.
#[derive(Clone, Debug)]
pub struct StreamAccumulator {
    pub chunks: Vec<Vec<u8>>,
    pub accumulated: Vec<u8>,
}

impl StreamAccumulator {
    /// Create an empty accumulator.
    pub fn new() -> Self {
        StreamAccumulator {
            chunks: Vec::new(),
            accumulated: Vec::new(),
        }
    }

    /// Add a chunk to the accumulator.
    ///
    /// Appends the chunk to the list and extends the accumulated buffer.
    pub fn add_chunk(&mut self, chunk: &[u8]) {
        let mut chunk_vec: Vec<u8> = Vec::new();
        let mut i: usize = 0;
        while i < chunk.len() {
            chunk_vec.push(chunk[i]);
            i += 1;
        }
        self.chunks.push(chunk_vec);

        // Extend accumulated
        let mut j: usize = 0;
        while j < chunk.len() {
            self.accumulated.push(chunk[j]);
            j += 1;
        }
    }

    /// Check whether the accumulated data has reached the expected length.
    pub fn is_complete(&self, expected_len: usize) -> bool {
        self.accumulated.len() >= expected_len
    }

    /// Return a reference to the accumulated data.
    pub fn get_accumulated(&self) -> &[u8] {
        &self.accumulated
    }
}

/// Split a byte slice into chunks of the given size.
///
/// The last chunk may be smaller than `chunk_size`.
/// Uses explicit indexing for Aeneas compatibility.
pub fn chunk_response(full: &[u8], chunk_size: usize) -> Vec<Vec<u8>> {
    let mut result: Vec<Vec<u8>> = Vec::new();
    if chunk_size == 0 {
        return result;
    }

    let mut offset: usize = 0;
    while offset < full.len() {
        let mut end = offset + chunk_size;
        if end > full.len() {
            end = full.len();
        }

        let mut chunk: Vec<u8> = Vec::new();
        let mut i: usize = offset;
        while i < end {
            chunk.push(full[i]);
            i += 1;
        }

        result.push(chunk);
        offset = end;
    }

    result
}
