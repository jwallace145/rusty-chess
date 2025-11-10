use crate::board::ChessMove;
use std::collections::HashMap;

#[derive(Clone)]
pub struct TTEntry {
    pub depth: u8,
    pub score: i32,
    pub best_move: Option<ChessMove>,
}

pub struct TranspositionTable {
    table: HashMap<u64, TTEntry>,
    hits: usize,
    misses: usize,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    pub fn probe(&mut self, hash: u64, depth: u8) -> Option<TTEntry> {
        if let Some(entry) = self.table.get(&hash)
            && entry.depth >= depth
        {
            self.hits += 1;
            return Some(entry.clone());
        }
        self.misses += 1;
        None
    }

    pub fn store(&mut self, hash: u64, depth: u8, score: i32, best_move: Option<ChessMove>) {
        // Always replace strategy - could be improved
        self.table.insert(
            hash,
            TTEntry {
                depth,
                score,
                best_move,
            },
        );
    }

    pub fn clear(&mut self) {
        self.table.clear();
        self.hits = 0;
        self.misses = 0;
    }

    pub fn stats(&self) -> (usize, usize) {
        (self.hits, self.misses)
    }
}
