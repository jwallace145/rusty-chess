use crate::board::{Board2, ChessMove};
use crate::opening::OpeningBook;
use crate::search::{Minimax, SearchHistory, SearchMetrics, SearchParams};
use crate::transpositions::TranspositionTable;

/// Chess move search engine using minimax with alpha-beta pruning.
///
/// Manages a transposition table for caching board evaluations across searches.
/// Call `new_game()` to clear the cache between games.
pub struct ChessEngine {
    minimax: Minimax,
    tt: TranspositionTable,
    last_search_metrics: Option<SearchMetrics>,
    opening_book: Option<OpeningBook>,
    use_opening_book: bool,
}

impl Default for ChessEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ChessEngine {
    pub fn new() -> Self {
        Self {
            minimax: Minimax::new(),
            tt: TranspositionTable::default(),
            last_search_metrics: None,
            opening_book: None,
            use_opening_book: false,
        }
    }

    /// Creates a new ChessEngine with an opening book loaded from a file.
    pub fn with_opening_book(book_path: &str) -> std::io::Result<Self> {
        let book = OpeningBook::load(book_path)?;
        Ok(Self {
            minimax: Minimax::new(),
            tt: TranspositionTable::default(),
            last_search_metrics: None,
            opening_book: Some(book),
            use_opening_book: true,
        })
    }

    /// Sets the opening book for this engine. If None, the opening book is disabled.
    pub fn set_opening_book(&mut self, book: Option<OpeningBook>) {
        self.opening_book = book;
        self.use_opening_book = self.opening_book.is_some();
    }

    /// Enables or disables the opening book.
    pub fn set_use_opening_book(&mut self, enabled: bool) {
        self.use_opening_book = enabled && self.opening_book.is_some();
    }

    pub fn find_best_move(&mut self, board: &Board2, depth: u8) -> Option<ChessMove> {
        // Check opening book first if enabled
        if self.use_opening_book
            && let Some(ref book) = self.opening_book
            && let Some(uci_move) = book.probe(board.hash)
        {
            // Try to parse and validate the UCI move
            if let Ok(chess_move) = board.parse_uci(uci_move) {
                return Some(chess_move);
            } else {
                eprintln!("Warning: Opening book returned invalid move: {}", uci_move);
            }
        }

        // Fall back to minimax search
        let mut history = SearchHistory::new();
        let mut metrics = SearchMetrics::new();

        let result =
            self.minimax
                .find_best_move(board, depth, &mut history, &mut self.tt, &mut metrics);

        self.print_search_stats(&metrics);

        // Store metrics for later retrieval
        self.last_search_metrics = Some(metrics);

        result
    }

    /// Find the best move using iterative deepening with time-based search.
    ///
    /// This method uses iterative deepening, starting from depth 1 and incrementally
    /// increasing until either:
    /// - The maximum depth is reached AND minimum search time has elapsed
    /// - The minimum search time has elapsed (even if max depth was reached earlier)
    ///
    /// # Arguments
    /// * `board` - The current board position
    /// * `params` - Search parameters (max depth and minimum search time)
    ///
    /// # Returns
    /// The best move found, or None if no legal moves exist
    pub fn find_best_move_iterative(
        &mut self,
        board: &Board2,
        params: &SearchParams,
    ) -> Option<ChessMove> {
        // Check opening book first if enabled
        if self.use_opening_book
            && let Some(ref book) = self.opening_book
            && let Some(uci_move) = book.probe(board.hash)
        {
            // Try to parse and validate the UCI move
            if let Ok(chess_move) = board.parse_uci(uci_move) {
                return Some(chess_move);
            } else {
                eprintln!("Warning: Opening book returned invalid move: {}", uci_move);
            }
        }

        // Fall back to iterative deepening search
        let mut history = SearchHistory::new();
        let mut metrics = SearchMetrics::new();

        let result = self.minimax.find_best_move_iterative(
            board,
            params,
            &mut history,
            &mut self.tt,
            &mut metrics,
        );

        self.print_search_stats(&metrics);

        // Store metrics for later retrieval
        self.last_search_metrics = Some(metrics);

        result
    }

    // Clear the transposition table (call when starting a new game)
    pub fn new_game(&mut self) {
        self.tt.clear();
    }

    /// Get the last search metrics (from the most recent find_best_move call)
    pub fn get_last_search_metrics(&self) -> Option<SearchMetrics> {
        self.last_search_metrics
    }

    /// Get transposition table statistics (hits, misses)
    pub fn get_tt_stats(&self) -> (usize, usize) {
        self.tt.stats()
    }

    /// Get transposition table size in bytes
    pub fn get_tt_size_bytes(&self) -> usize {
        self.tt.size_bytes()
    }

    /// Get transposition table number of entries (capacity)
    pub fn get_tt_num_entries(&self) -> usize {
        self.tt.num_entries()
    }

    /// Get transposition table hits
    pub fn get_tt_hits(&self) -> usize {
        self.tt.hits()
    }

    /// Get transposition table misses
    pub fn get_tt_misses(&self) -> usize {
        self.tt.misses()
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
