use crate::board::{Board2, ChessMove, MoveGenerator2, Piece};
use crate::eval::Evaluator;
use crate::search::SearchHistory;
use crate::transpositions::TranspositionTable;
use std::time::Instant;

/// Parameters for configuring search behavior.
#[derive(Debug, Clone, Copy)]
pub struct SearchParams {
    /// Maximum depth to search (prevents going too deep)
    pub max_depth: u8,
    /// Minimum time to search per move in milliseconds (ensures sufficient evaluation)
    pub min_search_time_ms: u64,
}

impl SearchParams {
    pub fn new(max_depth: u8, min_search_time_ms: u64) -> Self {
        Self {
            max_depth,
            min_search_time_ms,
        }
    }
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            max_depth: 8,
            min_search_time_ms: 1000,
        }
    }
}

/// Time constraints for search operations.
#[derive(Debug, Clone, Copy)]
struct TimeConstraints<'a> {
    start_time: &'a Instant,
    min_time_ms: u64,
}

impl<'a> TimeConstraints<'a> {
    fn new(start_time: &'a Instant, min_time_ms: u64) -> Self {
        Self {
            start_time,
            min_time_ms,
        }
    }

    fn is_time_exceeded(&self) -> bool {
        self.start_time.elapsed().as_millis() >= self.min_time_ms as u128
    }
}

/// Statistics gathered during a minimax search operation.
#[derive(Debug, Default, Clone, Copy)]
pub struct SearchMetrics {
    /// Total number of nodes explored during search
    pub nodes_explored: u64,
    /// Maximum depth reached during search
    pub max_depth_reached: u8,
    /// Number of times alpha-beta pruning occurred
    pub beta_cutoffs: u64,
    /// Time taken for the search
    pub search_time: std::time::Duration,
}

impl SearchMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Chess AI using minimax algorithm with alpha-beta pruning.
///
/// This struct implements the minimax search algorithm to find the best chess move
/// by exploring the game tree to a specified depth. The search alternates between
/// maximizing and minimizing players, evaluating positions using a board evaluation function.
///
/// # Optimizations
/// - **Alpha-beta pruning**: Eliminates branches that cannot affect the final decision
/// - **Transposition table**: Caches previously evaluated positions to avoid redundant work
///
/// # References
/// - [Wikipedia: Minimax](https://en.wikipedia.org/wiki/Minimax)
pub struct Minimax {
    evaluator: Evaluator,
}

impl Default for Minimax {
    fn default() -> Self {
        Self::new()
    }
}

impl Minimax {
    /// Creates a new Minimax instance with a fresh evaluator
    pub fn new() -> Self {
        Self {
            evaluator: Evaluator::new(),
        }
    }

    /// Find the best move using minimax with alpha-beta pruning
    pub fn find_best_move(
        &self,
        board: &Board2,
        depth: u8,
        history: &mut SearchHistory,
        tt: &mut TranspositionTable,
        metrics: &mut SearchMetrics,
    ) -> Option<ChessMove> {
        let start_time = Instant::now();

        // Initialize history with the current position
        history.push(board.hash);

        // Preallocate move buffer for reuse across recursive calls
        let mut move_buffer = Vec::with_capacity(128);
        MoveGenerator2::generate_legal_moves(board, &mut move_buffer);

        if move_buffer.is_empty() {
            history.pop(); // Clean up before returning
            metrics.search_time = start_time.elapsed();
            return None;
        }

        // Order moves for better alpha-beta performance
        Self::order_moves(board, &mut move_buffer, tt);

        // Copy moves to avoid them being overwritten during recursive calls
        let moves: Vec<ChessMove> = move_buffer.to_vec();
        let mut best_move = moves[0];
        let mut best_score = i32::MIN;

        for chess_move in &moves {
            let mut board_copy = *board;
            board_copy.make_move(*chess_move);

            // Push position before recursing
            history.push(board_copy.hash);

            let score = -self.alpha_beta(
                &board_copy,
                depth - 1,
                i32::MIN + 1,
                i32::MAX,
                history,
                tt,
                metrics,
                depth,
                &mut move_buffer,
            );

            // Pop position after returning
            history.pop();

            if score > best_score {
                best_score = score;
                best_move = *chess_move;
            }
        }

        history.pop(); // Clean up the initial position
        metrics.search_time = start_time.elapsed();
        Some(best_move)
    }

    /// Find the best move using iterative deepening with time-based search.
    ///
    /// This method performs iterative deepening, starting from depth 1 and incrementally
    /// increasing until either:
    /// - The maximum depth is reached AND minimum search time has elapsed
    /// - The minimum search time has elapsed (even if max depth was reached earlier)
    ///
    /// This ensures that:
    /// - There's always a valid best move if timeout occurs
    /// - The engine searches for at least the minimum time
    /// - Search doesn't go beyond the maximum depth unless needed to satisfy min time
    pub fn find_best_move_iterative(
        &self,
        board: &Board2,
        params: &SearchParams,
        history: &mut SearchHistory,
        tt: &mut TranspositionTable,
        metrics: &mut SearchMetrics,
    ) -> Option<ChessMove> {
        let start_time = Instant::now();

        // Initialize history with the current position
        history.push(board.hash);

        // Check if there are any legal moves
        let mut move_buffer = Vec::with_capacity(128);
        MoveGenerator2::generate_legal_moves(board, &mut move_buffer);

        if move_buffer.is_empty() {
            history.pop();
            metrics.search_time = start_time.elapsed();
            return None;
        }

        let mut best_move: Option<ChessMove> = None;
        let mut depth = 1;

        // Iterative deepening up to max_depth
        let time_constraints = TimeConstraints::new(&start_time, params.min_search_time_ms);
        while depth <= params.max_depth {
            let (_score, mv) =
                self.search_depth(board, depth, &time_constraints, history, tt, metrics);

            if let Some(mv) = mv {
                best_move = Some(mv);
            }

            // Stop if we've exceeded the minimum time
            if start_time.elapsed().as_millis() >= params.min_search_time_ms as u128 {
                break;
            }

            depth += 1;
        }

        // Ensure minimum search time is satisfied
        // Continue searching at deeper depths if we haven't used enough time
        while start_time.elapsed().as_millis() < params.min_search_time_ms as u128 {
            depth += 1;
            let (_score, mv) =
                self.search_depth(board, depth, &time_constraints, history, tt, metrics);

            if let Some(mv) = mv {
                best_move = Some(mv);
            }

            // If search completed very quickly, avoid infinite loop
            // This can happen in endgames with few pieces
            if mv.is_none() {
                break;
            }
        }

        history.pop();
        metrics.search_time = start_time.elapsed();

        best_move
    }

    /// Performs a depth-limited search with time checking.
    ///
    /// Returns (score, best_move). If time limit is exceeded, returns (0, None).
    fn search_depth(
        &self,
        board: &Board2,
        depth: u8,
        time_constraints: &TimeConstraints,
        history: &mut SearchHistory,
        tt: &mut TranspositionTable,
        metrics: &mut SearchMetrics,
    ) -> (i32, Option<ChessMove>) {
        // Stop if time limit is exceeded
        if time_constraints.is_time_exceeded() {
            return (0, None);
        }

        // Generate moves
        let mut move_buffer = Vec::with_capacity(128);
        MoveGenerator2::generate_legal_moves(board, &mut move_buffer);

        if move_buffer.is_empty() {
            return (0, None);
        }

        // Order moves for better alpha-beta performance
        Self::order_moves(board, &mut move_buffer, tt);

        // Copy moves to avoid them being overwritten during recursive calls
        let moves: Vec<ChessMove> = move_buffer.to_vec();
        let mut best_score = i32::MIN;
        let mut best_move = moves[0];

        for chess_move in &moves {
            let mut board_copy = *board;
            board_copy.make_move(*chess_move);

            // Push position before recursing
            history.push(board_copy.hash);

            let score = -self.alpha_beta_with_time(
                &board_copy,
                depth - 1,
                i32::MIN + 1,
                i32::MAX,
                history,
                tt,
                metrics,
                depth,
                &mut move_buffer,
                time_constraints,
            );

            // Pop position after returning
            history.pop();

            if score > best_score {
                best_score = score;
                best_move = *chess_move;
            }

            // Check time limit during move iteration
            if time_constraints.is_time_exceeded() {
                break;
            }
        }

        (best_score, Some(best_move))
    }

    /// Alpha-beta search with time checking for iterative deepening.
    #[allow(clippy::too_many_arguments)]
    fn alpha_beta_with_time(
        &self,
        board: &Board2,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        history: &mut SearchHistory,
        tt: &mut TranspositionTable,
        metrics: &mut SearchMetrics,
        original_depth: u8,
        move_buffer: &mut Vec<ChessMove>,
        time_constraints: &TimeConstraints,
    ) -> i32 {
        // Check time limit at each node
        if time_constraints.is_time_exceeded() {
            return 0; // Time exceeded, return neutral score
        }

        // Track nodes explored and max depth
        metrics.nodes_explored += 1;
        let current_depth = original_depth - depth;
        if current_depth > metrics.max_depth_reached {
            metrics.max_depth_reached = current_depth;
        }

        // Check for repetition FIRST - this prevents infinite check loops
        if history.is_repetition(board.hash) {
            return 0; // Repetition is a draw
        }

        // Probe transposition table - use board.hash directly!
        if let Some(entry) = tt.probe(board.hash, depth) {
            return entry.score;
        }

        // Generate legal moves into the shared buffer
        MoveGenerator2::generate_legal_moves(board, move_buffer);

        // Check for terminal positions (checkmate or stalemate)
        if move_buffer.is_empty() {
            let score = if board.in_check(board.side_to_move) {
                // Losing position - adjust score by depth to prefer faster checkmates
                -100_000 - (depth as i32)
            } else {
                // Stalemate - return draw score
                0
            };
            // Store terminal position in TT
            tt.store(board.hash, depth, score, None);
            return score;
        }

        // Leaf node - evaluate position
        if depth == 0 {
            let score = self.evaluator.evaluate(board);
            tt.store(board.hash, depth, score, None);
            return score;
        }

        // Order moves for better pruning efficiency
        Self::order_moves(board, move_buffer, tt);

        // Copy moves to avoid them being overwritten during recursive calls
        let moves: Vec<ChessMove> = move_buffer.to_vec();
        let mut best_move = None;

        for chess_move in moves {
            let mut board_copy = *board;
            board_copy.make_move(chess_move);

            // Push position before recursing
            history.push(board_copy.hash);

            let score = -self.alpha_beta_with_time(
                &board_copy,
                depth - 1,
                -beta,
                -alpha,
                history,
                tt,
                metrics,
                original_depth,
                move_buffer,
                time_constraints,
            );

            // Pop position after returning
            history.pop();

            // Beta cutoff - opponent won't allow this position
            if score >= beta {
                metrics.beta_cutoffs += 1;
                tt.store(board.hash, depth, beta, Some(chess_move));
                return beta;
            }

            // Update alpha if we found a better move
            if score > alpha {
                alpha = score;
                best_move = Some(chess_move);
            }
        }

        // Store the result in transposition table
        tt.store(board.hash, depth, alpha, best_move);

        alpha
    }

    #[allow(clippy::too_many_arguments)]
    fn alpha_beta(
        &self,
        board: &Board2,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        history: &mut SearchHistory,
        tt: &mut TranspositionTable,
        metrics: &mut SearchMetrics,
        original_depth: u8,
        move_buffer: &mut Vec<ChessMove>,
    ) -> i32 {
        // Track nodes explored and max depth
        metrics.nodes_explored += 1;
        let current_depth = original_depth - depth;
        if current_depth > metrics.max_depth_reached {
            metrics.max_depth_reached = current_depth;
        }

        // Check for repetition FIRST - this prevents infinite check loops
        if history.is_repetition(board.hash) {
            return 0; // Repetition is a draw
        }

        // Probe transposition table - use board.hash directly!
        if let Some(entry) = tt.probe(board.hash, depth) {
            return entry.score;
        }

        // Generate legal moves into the shared buffer
        MoveGenerator2::generate_legal_moves(board, move_buffer);

        // Check for terminal positions (checkmate or stalemate)
        if move_buffer.is_empty() {
            let score = if board.in_check(board.side_to_move) {
                // Losing position - adjust score by depth to prefer faster checkmates
                -100_000 - (depth as i32)
            } else {
                // Stalemate - return draw score
                0
            };
            // Store terminal position in TT
            tt.store(board.hash, depth, score, None);
            return score;
        }

        // Leaf node - evaluate position
        if depth == 0 {
            let score = self.evaluator.evaluate(board);
            tt.store(board.hash, depth, score, None);
            return score;
        }

        // Order moves for better pruning efficiency
        Self::order_moves(board, move_buffer, tt);

        // Copy moves to avoid them being overwritten during recursive calls
        let moves: Vec<ChessMove> = move_buffer.to_vec();
        let mut best_move = None;

        for chess_move in moves {
            let mut board_copy = *board;
            board_copy.make_move(chess_move);

            // Push position before recursing
            history.push(board_copy.hash);

            let score = -self.alpha_beta(
                &board_copy,
                depth - 1,
                -beta,
                -alpha,
                history,
                tt,
                metrics,
                original_depth,
                move_buffer,
            );

            // Pop position after returning
            history.pop();

            // Beta cutoff - opponent won't allow this position
            if score >= beta {
                metrics.beta_cutoffs += 1;
                tt.store(board.hash, depth, beta, Some(chess_move));
                return beta;
            }

            // Update alpha if we found a better move
            if score > alpha {
                alpha = score;
                best_move = Some(chess_move);
            }
        }

        // Store the result in transposition table
        tt.store(board.hash, depth, alpha, best_move);

        alpha
    }

    /// Order moves to search promising moves first (improves alpha-beta pruning)
    /// Priority: TT best move first, then captures by victim value, then non-captures
    ///
    /// This method operates in-place on the provided buffer for better performance.
    fn order_moves(board: &Board2, moves: &mut [ChessMove], tt: &mut TranspositionTable) {
        // Try to get best move from transposition table
        if let Some(entry) = tt.probe(board.hash, 0)
            && let Some(tt_best_move) = entry.best_move
        {
            // Find the TT best move in our list
            if let Some(pos) = moves.iter().position(|&m| m == tt_best_move) {
                // Move it to the front
                moves.swap(0, pos);
                // Sort the rest by capture value
                moves[1..].sort_by_key(|m| Self::move_priority(board, m));
                return;
            }
        }

        // No TT move found, sort all moves by priority
        moves.sort_by_key(|m| Self::move_priority(board, m));
    }

    fn move_priority(board: &Board2, chess_move: &ChessMove) -> i32 {
        if chess_move.capture {
            let victim_value = if let Some((_, piece)) = board.piece_on(chess_move.to as u8) {
                Self::piece_value(piece)
            } else {
                100 // En passant captures a pawn
            };

            // Get attacker value
            let attacker_value = if let Some((_, piece)) = board.piece_on(chess_move.from as u8) {
                Self::piece_value(piece)
            } else {
                0 // Shouldn't happen
            };

            // MVV-LVA: High victim value, low attacker value = best
            // victim_value * 10 - attacker_value ensures victim is primary factor
            // Example: Pawn (100) takes Queen (900) = 9000 - 100 = 8900
            // Example: Queen (900) takes Queen (900) = 9000 - 900 = 8100
            // Negative for descending sort order
            -(victim_value * 10 - attacker_value)
        } else {
            0 // Non-captures have neutral priority
        }
    }

    fn piece_value(piece: Piece) -> i32 {
        match piece {
            Piece::Pawn => 100,
            Piece::Knight => 320,
            Piece::Bishop => 330,
            Piece::Rook => 500,
            Piece::Queen => 900,
            Piece::King => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::chess_move::ChessMoveType;
    use crate::board::{Color, Piece};

    #[test]
    fn test_finds_checkmate_in_one() {
        let mut board = Board2::new_empty();

        // Set up position: White King on c7, Black King on a8, White Queen on c6
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << pos("c7");
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << pos("a8");
        board.pieces[Color::White as usize][Piece::Queen as usize] = 1u64 << pos("c6");

        board.king_sq[Color::White as usize] = pos("c7") as u8;
        board.king_sq[Color::Black as usize] = pos("a8") as u8;

        board.occ[Color::White as usize] = (1u64 << pos("c7")) | (1u64 << pos("c6"));
        board.occ[Color::Black as usize] = 1u64 << pos("a8");
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        board.side_to_move = Color::White;

        // CRITICAL: Compute the zobrist hash for the position
        board.hash = crate::search::compute_hash_board2(&board);

        // Create a TT and metrics for the test
        let minimax = Minimax::new();
        let mut tt = TranspositionTable::new_with_entries(1024);
        let mut metrics = SearchMetrics::new();
        let mut history = SearchHistory::new();
        let best_move = minimax.find_best_move(&board, 3, &mut history, &mut tt, &mut metrics);
        assert!(best_move.is_some());

        let chess_move = best_move.unwrap();
        let mut test_board = board;
        test_board.make_move(chess_move);

        // Check if it's checkmate
        let mut moves = Vec::new();
        MoveGenerator2::generate_legal_moves(&test_board, &mut moves);
        assert!(
            moves.is_empty() && test_board.in_check(test_board.side_to_move),
            "Should deliver checkmate"
        );
    }

    #[test]
    fn test_finds_checkmate_fools_game() {
        let mut board = Board2::new_standard();
        board.make_move(ChessMove {
            from: pos("f2"),
            to: pos("f3"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });
        board.make_move(ChessMove {
            from: pos("e7"),
            to: pos("e5"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });
        board.make_move(ChessMove {
            from: pos("g2"),
            to: pos("g4"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });

        let minimax = Minimax::new();
        let mut tt = TranspositionTable::new_with_entries(1024);
        let mut metrics = SearchMetrics::new();
        let mut history = SearchHistory::new();
        let best_move = minimax.find_best_move(&board, 3, &mut history, &mut tt, &mut metrics);
        assert!(best_move.is_some());

        let chess_move = best_move.unwrap();
        let mut test_board = board;
        test_board.make_move(chess_move);

        // Check if it's checkmate
        let mut moves = Vec::new();
        MoveGenerator2::generate_legal_moves(&test_board, &mut moves);
        assert!(
            moves.is_empty() && test_board.in_check(test_board.side_to_move),
            "Should deliver checkmate"
        )
    }

    fn pos(s: &str) -> usize {
        let bytes = s.as_bytes();
        let file = (bytes[0] - b'a') as usize;
        let rank = (bytes[1] - b'1') as usize;
        rank * 8 + file
    }
}
