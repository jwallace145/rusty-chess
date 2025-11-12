pub struct SearchHistory {
    positions: Vec<u64>,
}

impl Default for SearchHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchHistory {
    pub fn new() -> Self {
        Self {
            positions: Vec::with_capacity(100),
        }
    }

    pub fn push(&mut self, hash: u64) {
        self.positions.push(hash);
    }

    pub fn pop(&mut self) {
        self.positions.pop();
    }

    pub fn count(&self, hash: u64) -> usize {
        self.positions.iter().filter(|&&h| h == hash).count()
    }

    pub fn is_repetition(&self, hash: u64) -> bool {
        self.count(hash) >= 3
    }
}
