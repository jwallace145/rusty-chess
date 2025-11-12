use crate::board::ChessMove;
use std::collections::HashMap;

/// A cached evaluation of a chess position.
///
/// # Fields
/// - `depth`: Search depth at which this position was evaluated
/// - `score`: Evaluated score of the position
/// - `best_move`: Best move found from this position, if any
#[derive(Copy, Clone)]
pub struct TTEntry {
    pub depth: u8,
    pub score: i32,
    pub best_move: Option<ChessMove>,
}

/// A cache for previously evaluated chess positions.
///
/// During search, different move sequences often reach identical positions
/// (transpositions). This table stores each position's evaluation and best
/// move to avoid redundant analysis.
///
/// # Fields
/// - `table`: Maps Zobrist hashes to cached evaluations and best moves
/// - `hits`: Count of successful cache lookups
/// - `misses`: Count of positions not found in cache
///
/// # References
/// - [Wikipedia: Transposition Tables](https://en.wikipedia.org/wiki/Transposition_table)
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

    /// Probe the cache for a previously evaluated position
    pub fn probe(&mut self, hash: u64, depth: u8) -> Option<TTEntry> {
        if let Some(&entry) = self.table.get(&hash)
            && entry.depth >= depth
        {
            self.hits += 1;
            return Some(entry);
        }
        self.misses += 1;
        None
    }

    /// Store evaluated position in the cache (evict entries with lower depth)
    pub fn store(&mut self, hash: u64, depth: u8, score: i32, best_move: Option<ChessMove>) {
        use std::collections::hash_map::Entry;

        match self.table.entry(hash) {
            Entry::Vacant(e) => {
                e.insert(TTEntry {
                    depth,
                    score,
                    best_move,
                });
            }
            Entry::Occupied(mut e) => {
                if depth >= e.get().depth {
                    e.insert(TTEntry {
                        depth,
                        score,
                        best_move,
                    });
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.table.clear();
        self.hits = 0;
        self.misses = 0;
    }

    pub fn stats(&self) -> (usize, usize) {
        (self.hits, self.misses)
    }

    /// Get the number of entries currently stored in the table
    pub fn size(&self) -> usize {
        self.table.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transposition_table_cache_hit() {
        let mut tt = TranspositionTable::new();
        let hash = 12345u64;
        let depth = 5;
        let score = 100;
        let best_move = None;

        // Store an entry
        tt.store(hash, depth, score, best_move);

        // Probe with same depth - should hit
        let result = tt.probe(hash, depth);
        assert!(result.is_some());
        let entry = result.unwrap();
        assert_eq!(entry.depth, depth);
        assert_eq!(entry.score, score);
        assert_eq!(entry.best_move, best_move);

        // Verify stats show 1 hit, 0 misses
        let (hits, misses) = tt.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 0);

        // Probe with lower depth - should also hit
        let result2 = tt.probe(hash, depth - 1);
        assert!(result2.is_some());

        // Stats should now show 2 hits
        let (hits, misses) = tt.stats();
        assert_eq!(hits, 2);
        assert_eq!(misses, 0);
    }

    #[test]
    fn test_transposition_table_cache_miss() {
        let mut tt = TranspositionTable::new();
        let hash = 12345u64;

        // Probe empty table - should miss
        let result = tt.probe(hash, 5);
        assert!(result.is_none());

        // Verify stats show 0 hits, 1 miss
        let (hits, misses) = tt.stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 1);

        // Store an entry at depth 3
        tt.store(hash, 3, 100, None);

        // Probe with higher depth requirement - should miss
        let result = tt.probe(hash, 5);
        assert!(result.is_none());

        // Stats should show 0 hits, 2 misses
        let (hits, misses) = tt.stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 2);
    }

    #[test]
    fn test_transposition_table_depth_replacement() {
        let mut tt = TranspositionTable::new();
        let hash = 12345u64;

        // Store entry at depth 3
        tt.store(hash, 3, 100, None);

        // Store entry at higher depth - should replace
        tt.store(hash, 5, 200, None);

        let result = tt.probe(hash, 5);
        assert!(result.is_some());
        let entry = result.unwrap();
        assert_eq!(entry.depth, 5);
        assert_eq!(entry.score, 200);

        // Store entry at lower depth - should NOT replace
        tt.store(hash, 2, 300, None);

        let result = tt.probe(hash, 2);
        assert!(result.is_some());
        let entry = result.unwrap();
        assert_eq!(entry.depth, 5); // Should still be the deeper entry
        assert_eq!(entry.score, 200); // Should still be the deeper entry's score
    }

    #[test]
    fn test_transposition_table_clear() {
        let mut tt = TranspositionTable::new();

        // Add some entries
        tt.store(12345, 5, 100, None);
        tt.store(67890, 3, 200, None);
        tt.probe(12345, 5);

        // Clear the table
        tt.clear();

        // Verify stats are reset
        let (hits, misses) = tt.stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);

        // Verify entries are gone
        let result = tt.probe(12345, 5);
        assert!(result.is_none());
    }
}
