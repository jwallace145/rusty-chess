use super::transposition_table::TranspositionTable;
use crate::board::{Board, ChessMove};
use crate::search::{Minimax, SearchHistory, SearchMetrics};

/// Chess move search engine using minimax with alpha-beta pruning.
///
/// Manages a transposition table for caching board evaluations across searches.
/// Call `new_game()` to clear the cache between games.
pub struct ChessEngine {
    tt: TranspositionTable,
}

impl Default for ChessEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ChessEngine {
    pub fn new() -> Self {
        Self {
            tt: TranspositionTable::default(),
        }
    }

    pub fn find_best_move(&mut self, board: &Board, depth: u8) -> Option<ChessMove> {
        let mut history = SearchHistory::new();
        let mut metrics = SearchMetrics::new();

        let result =
            Minimax::find_best_move(board, depth, &mut history, &mut self.tt, &mut metrics);

        self.print_search_stats(&metrics);

        result
    }

    // Clear the transposition table (call when starting a new game)
    pub fn new_game(&mut self) {
        self.tt.clear();
    }

    /// Get transposition table statistics
    pub fn get_tt_stats(&self) -> (usize, usize) {
        self.tt.stats()
    }

    fn print_search_stats(&self, metrics: &SearchMetrics) {
        println!("\n=== Search Statistics ===");

        // Search time
        let time_ms = metrics.search_time.as_secs_f64() * 1000.0;
        println!("Search time: {:.2} ms", time_ms);

        // Nodes explored
        println!("Nodes explored: {}", metrics.nodes_explored);

        // Nodes per second
        if metrics.search_time.as_secs_f64() > 0.0 {
            let nps = metrics.nodes_explored as f64 / metrics.search_time.as_secs_f64();
            println!("Nodes/second: {:.0}", nps);
        }

        // Max depth reached
        println!("Max depth reached: {}", metrics.max_depth_reached);

        // Beta cutoffs
        println!("Beta cutoffs: {}", metrics.beta_cutoffs);
        let cutoff_rate = if metrics.nodes_explored > 0 {
            (metrics.beta_cutoffs as f64 / metrics.nodes_explored as f64) * 100.0
        } else {
            0.0
        };
        println!("Cutoff rate: {:.2}%", cutoff_rate);

        // Transposition table stats
        let (hits, misses) = self.tt.stats();
        let tt_size_entries = self.tt.size();
        let tt_size_bytes = self.tt.memory_usage();
        println!("\n--- Transposition Table ---");
        println!("TT size (entries): {} entries", tt_size_entries);
        println!("TT size (bytes): {}", tt_size_bytes);
        println!("TT hits: {}", hits);
        println!("TT misses: {}", misses);

        if hits + misses > 0 {
            let hit_rate = (hits as f64 / (hits + misses) as f64) * 100.0;
            println!("TT hit rate: {:.2}%", hit_rate);
        }

        println!("========================\n");
    }
}
