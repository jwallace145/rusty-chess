use crate::transpositions::table::TranspositionTable;

/// Default Transposition Table
impl Default for TranspositionTable {
    fn default() -> Self {
        // Default to 2.5 GB transposition table
        Self::new_with_size_mb(4096)
    }
}
