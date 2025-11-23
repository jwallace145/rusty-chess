use crate::board::ChessMove;

/// A cached evaluation of a chess position.
///
/// # Fields
/// - `hash`: Zobrist hash of the position (0 indicates empty entry)
/// - `depth`: Search depth at which this position was evaluated
/// - `score`: Evaluated score of the position
/// - `best_move`: Best move found from this position, if any
#[derive(Copy, Clone)]
pub struct TTEntry {
    pub hash: u64,
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
/// - `table`: Fixed-size vector storing cached evaluations indexed by hash
/// - `capacity`: Maximum number of entries in the table
/// - `hits`: Count of successful cache lookups
/// - `misses`: Count of positions not found in cache
///
/// # References
/// - [Wikipedia: Transposition Tables](https://en.wikipedia.org/wiki/Transposition_table)
pub struct TranspositionTable {
    table: Vec<TTEntry>,
    num_entries: usize,
    hits: usize,
    misses: usize,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        // Default to 2.5 GB transposition table
        Self::new_with_size_mb(4096)
    }
}

impl TranspositionTable {
    /// Create a new transposition table with specified size in MB
    pub fn new_with_size_mb(size_mb: usize) -> Self {
        // Calculate number of entries based on desired memory size
        let entry_size = std::mem::size_of::<Option<TTEntry>>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;

        // Round down to nearest power of 2 for efficient modulo
        let num_entries = num_entries.next_power_of_two() / 2;

        Self::new_with_entries(num_entries)
    }

    /// Create a new transposition table with specified number of entries
    pub fn new_with_entries(num_entries: usize) -> Self {
        let entry = TTEntry {
            hash: 0,
            depth: 0,
            score: 0,
            best_move: None,
        };
        Self {
            table: vec![entry; num_entries],
            num_entries,
            hits: 0,
            misses: 0,
        }
    }

    /// Probe the cache for a previously evaluated position
    pub fn probe(&mut self, hash: u64, depth: u8) -> Option<TTEntry> {
        let index = (hash as usize) % self.num_entries;
        let entry = self.table[index];

        if entry.hash == hash && entry.depth >= depth {
            self.hits += 1;
            return Some(entry);
        }
        self.misses += 1;
        None
    }

    /// Store evaluated position in the cache (evict entries with lower depth)
    pub fn store(&mut self, hash: u64, depth: u8, score: i32, best_move: Option<ChessMove>) {
        let index = (hash as usize) % self.num_entries;
        let entry = &self.table[index];

        // Only replace if the slot is empty or we have a deeper search
        if entry.hash == 0 || depth >= entry.depth {
            self.table[index] = TTEntry {
                hash,
                depth,
                score,
                best_move,
            };
        }
    }

    pub fn clear(&mut self) {
        let empty_entry = TTEntry {
            hash: 0,
            depth: 0,
            score: 0,
            best_move: None,
        };
        self.table.fill(empty_entry);
        self.hits = 0;
        self.misses = 0;
    }

    pub fn stats(&self) -> (usize, usize) {
        (self.hits, self.misses)
    }

    /// Get the number of entries currently stored in the table
    pub fn size(&self) -> usize {
        self.table.iter().filter(|e| e.hash != 0).count()
    }

    /// Get the approximate memory usage of the table in a human-readable format
    pub fn memory_usage(&self) -> String {
        let entry_size = std::mem::size_of::<TTEntry>();
        let total_bytes = self.size() * entry_size;

        if total_bytes < 1024 {
            format!("{} B", total_bytes)
        } else if total_bytes < 1024 * 1024 {
            format!("{:.2} KB", total_bytes as f64 / 1024.0)
        } else if total_bytes < 1024 * 1024 * 1024 {
            format!("{:.2} MB", total_bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", total_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Get the memory usage in bytes
    pub fn size_bytes(&self) -> usize {
        let entry_size = std::mem::size_of::<TTEntry>();
        self.size() * entry_size
    }

    /// Get the total number of entries (capacity)
    pub fn num_entries(&self) -> usize {
        self.num_entries
    }

    /// Get the number of cache hits
    pub fn hits(&self) -> usize {
        self.hits
    }

    /// Get the number of cache misses
    pub fn misses(&self) -> usize {
        self.misses
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transposition_table_cache_hit() {
        let mut tt = TranspositionTable::new_with_entries(1024);
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
        assert_eq!(entry.hash, hash);
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
        let mut tt = TranspositionTable::new_with_entries(1024);
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
        let mut tt = TranspositionTable::new_with_entries(1024);
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
        let mut tt = TranspositionTable::new_with_entries(1024);

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

        // Verify size is 0
        assert_eq!(tt.size(), 0);
    }

    #[test]
    fn test_transposition_table_memory_usage() {
        let tt_small = TranspositionTable::new_with_entries(100);
        let memory = tt_small.memory_usage();
        assert!(memory.contains("B") || memory.contains("KB"));
    }

    #[test]
    fn test_transposition_table_size() {
        let mut tt = TranspositionTable::new_with_entries(1024);

        // Initially empty
        assert_eq!(tt.size(), 0);

        // Add one entry
        tt.store(12345, 5, 100, None);
        assert_eq!(tt.size(), 1);

        // Add another entry
        tt.store(67890, 3, 200, None);
        assert_eq!(tt.size(), 2);

        // Replace existing entry (size should stay the same)
        tt.store(12345, 6, 150, None);
        assert_eq!(tt.size(), 2);
    }
}
