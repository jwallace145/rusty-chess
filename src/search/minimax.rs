use crate::board::{Board, ChessMove, Piece};
use crate::eval::Evaluator;
use crate::movegen::MoveGenerator;
use crate::search::SearchHistory;
use crate::search::quiescence::quiescence_search;
use crate::transpositions::{Bound, TranspositionTable};
use std::time::Instant;

/// Maximum search depth for PV table
const MAX_PLY: usize = 64;

/// History heuristic table for move ordering.
/// Tracks scores for quiet moves that cause beta cutoffs.
/// Indexed by [from_square][to_square] where squares are 0-63.
#[derive(Clone)]
pub struct HistoryTable {
    scores: [[i32; 64]; 64],
}

impl HistoryTable {
    pub fn new() -> Self {
        Self {
            scores: [[0; 64]; 64],
        }
    }

    /// Get the history score for a move
    pub fn get(&self, chess_move: &ChessMove) -> i32 {
        self.scores[chess_move.from][chess_move.to]
    }

    /// Increment history score for a move that caused a beta cutoff
    pub fn increment(&mut self, chess_move: &ChessMove, depth: u8) {
        let from = chess_move.from;
        let to = chess_move.to;
        // Bonus based on depth: deeper searches get higher bonuses
        self.scores[from][to] += (depth as i32) * (depth as i32);
    }

    /// Clear all history scores
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.scores = [[0; 64]; 64];
    }
}

/// Principal Variation table.
/// Stores the best move found at each depth from previous iterations.
#[derive(Clone)]
pub struct PVTable {
    moves: [Option<ChessMove>; MAX_PLY],
}

impl PVTable {
    pub fn new() -> Self {
        Self {
            moves: [None; MAX_PLY],
        }
    }

    /// Get the PV move for a given depth
    pub fn get(&self, depth: usize) -> Option<ChessMove> {
        if depth < MAX_PLY {
            self.moves[depth]
        } else {
            None
        }
    }

    /// Store the PV move for a given depth
    pub fn set(&mut self, depth: usize, chess_move: ChessMove) {
        if depth < MAX_PLY {
            self.moves[depth] = Some(chess_move);
        }
    }

    /// Clear all PV moves
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.moves = [None; MAX_PLY];
    }
}

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
        board: &Board,
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
        MoveGenerator::generate_legal_moves(board, &mut move_buffer);

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
    ///
    /// Enhanced with PV table and history heuristic for better move ordering.
    pub fn find_best_move_iterative(
        &self,
        board: &Board,
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
        MoveGenerator::generate_legal_moves(board, &mut move_buffer);

        if move_buffer.is_empty() {
            history.pop();
            metrics.search_time = start_time.elapsed();
            return None;
        }

        // Initialize PV table and history table for enhanced move ordering
        let mut pv_table = PVTable::new();
        let mut history_table = HistoryTable::new();

        let mut best_move: Option<ChessMove> = None;
        let mut depth = 1;

        // Iterative deepening up to max_depth
        let time_constraints = TimeConstraints::new(&start_time, params.min_search_time_ms);
        while depth <= params.max_depth {
            let (_score, mv) = self.search_depth(
                board,
                depth,
                &time_constraints,
                history,
                tt,
                metrics,
                &mut history_table,
                &pv_table,
            );

            if let Some(mv) = mv {
                best_move = Some(mv);
                // Store the best move in PV table for next iteration
                pv_table.set(depth as usize, mv);
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
            let (_score, mv) = self.search_depth(
                board,
                depth,
                &time_constraints,
                history,
                tt,
                metrics,
                &mut history_table,
                &pv_table,
            );

            if let Some(mv) = mv {
                best_move = Some(mv);
                // Store the best move in PV table for next iteration
                pv_table.set(depth as usize, mv);
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
    /// Uses PV-first and history heuristic for enhanced move ordering.
    #[allow(clippy::too_many_arguments)]
    fn search_depth(
        &self,
        board: &Board,
        depth: u8,
        time_constraints: &TimeConstraints,
        history: &mut SearchHistory,
        tt: &mut TranspositionTable,
        metrics: &mut SearchMetrics,
        history_table: &mut HistoryTable,
        pv_table: &PVTable,
    ) -> (i32, Option<ChessMove>) {
        // Stop if time limit is exceeded
        if time_constraints.is_time_exceeded() {
            return (0, None);
        }

        // Generate moves
        let mut move_buffer = Vec::with_capacity(128);
        MoveGenerator::generate_legal_moves(board, &mut move_buffer);

        if move_buffer.is_empty() {
            return (0, None);
        }

        // Get PV move from previous iteration for this depth
        let pv_move = pv_table.get(depth as usize);

        // Order moves with PV-first and history heuristic
        Self::order_moves_with_history(board, &mut move_buffer, tt, history_table, pv_move);

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
                history_table,
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
    /// Enhanced with history heuristic for better move ordering.
    #[allow(clippy::too_many_arguments)]
    fn alpha_beta_with_time(
        &self,
        board: &Board,
        depth: u8,
        mut alpha: i32,
        mut beta: i32,
        history: &mut SearchHistory,
        tt: &mut TranspositionTable,
        metrics: &mut SearchMetrics,
        original_depth: u8,
        move_buffer: &mut Vec<ChessMove>,
        time_constraints: &TimeConstraints,
        history_table: &mut HistoryTable,
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

        // Store original alpha for bound determination
        let original_alpha = alpha;

        // Probe transposition table - use board.hash directly!
        if let Some(entry) = tt.probe(board.hash) {
            // Only use entry if it was searched to at least the current depth
            if entry.depth >= depth {
                match entry.bound {
                    Bound::Exact => {
                        // Exact score - can use directly
                        return entry.score;
                    }
                    Bound::LowerBound => {
                        // Score is at least this high (fail-high)
                        if entry.score >= beta {
                            return entry.score;
                        }
                        alpha = alpha.max(entry.score);
                    }
                    Bound::UpperBound => {
                        // Score is at most this high (fail-low)
                        if entry.score <= alpha {
                            return entry.score;
                        }
                        beta = beta.min(entry.score);
                    }
                }
                // Check for cutoff after updating alpha/beta
                if alpha >= beta {
                    return entry.score;
                }
            }
        }

        // Generate legal moves into the shared buffer
        MoveGenerator::generate_legal_moves(board, move_buffer);

        // Check for terminal positions (checkmate or stalemate)
        if move_buffer.is_empty() {
            let score = if board.in_check(board.side_to_move) {
                // Losing position - adjust score by depth to prefer faster checkmates
                -100_000 - (depth as i32)
            } else {
                // Stalemate - return draw score
                0
            };
            // Store terminal position in TT (exact score)
            tt.store(board.hash, depth, score, None, Bound::Exact);
            return score;
        }

        // Leaf node - use quiescence search to resolve tactical sequences
        if depth == 0 {
            let score = quiescence_search(board, alpha, beta, &self.evaluator);
            tt.store(board.hash, depth, score, None, Bound::Exact);
            return score;
        }

        // Order moves with history heuristic (no PV move at internal nodes)
        Self::order_moves_with_history(board, move_buffer, tt, history_table, None);

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
                history_table,
            );

            // Pop position after returning
            history.pop();

            // Beta cutoff - opponent won't allow this position
            if score >= beta {
                metrics.beta_cutoffs += 1;

                // Update history table for quiet moves that cause beta cutoffs
                if !chess_move.capture {
                    history_table.increment(&chess_move, depth);
                }

                tt.store(
                    board.hash,
                    depth,
                    score,
                    Some(chess_move),
                    Bound::LowerBound,
                );
                return score;
            }

            // Update alpha if we found a better move
            if score > alpha {
                alpha = score;
                best_move = Some(chess_move);
            }
        }

        // Store the result in transposition table
        // Determine bound type based on whether we improved alpha
        let bound = if alpha > original_alpha {
            Bound::Exact // We found a move that improved our position
        } else {
            Bound::UpperBound // All moves failed low
        };
        tt.store(board.hash, depth, alpha, best_move, bound);

        alpha
    }

    #[allow(clippy::too_many_arguments)]
    fn alpha_beta(
        &self,
        board: &Board,
        depth: u8,
        mut alpha: i32,
        mut beta: i32,
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

        // Store original alpha for bound determination
        let original_alpha = alpha;

        // Probe transposition table - use board.hash directly!
        if let Some(entry) = tt.probe(board.hash) {
            // Only use entry if it was searched to at least the current depth
            if entry.depth >= depth {
                match entry.bound {
                    Bound::Exact => {
                        // Exact score - can use directly
                        return entry.score;
                    }
                    Bound::LowerBound => {
                        // Score is at least this high (fail-high)
                        if entry.score >= beta {
                            return entry.score;
                        }
                        alpha = alpha.max(entry.score);
                    }
                    Bound::UpperBound => {
                        // Score is at most this high (fail-low)
                        if entry.score <= alpha {
                            return entry.score;
                        }
                        beta = beta.min(entry.score);
                    }
                }
                // Check for cutoff after updating alpha/beta
                if alpha >= beta {
                    return entry.score;
                }
            }
        }

        // Generate legal moves into the shared buffer
        MoveGenerator::generate_legal_moves(board, move_buffer);

        // Check for terminal positions (checkmate or stalemate)
        if move_buffer.is_empty() {
            let score = if board.in_check(board.side_to_move) {
                // Losing position - adjust score by depth to prefer faster checkmates
                -100_000 - (depth as i32)
            } else {
                // Stalemate - return draw score
                0
            };
            // Store terminal position in TT (exact score)
            tt.store(board.hash, depth, score, None, Bound::Exact);
            return score;
        }

        // Leaf node - use quiescence search to resolve tactical sequences
        if depth == 0 {
            let score = quiescence_search(board, alpha, beta, &self.evaluator);
            tt.store(board.hash, depth, score, None, Bound::Exact);
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
                tt.store(
                    board.hash,
                    depth,
                    score,
                    Some(chess_move),
                    Bound::LowerBound,
                );
                return score;
            }

            // Update alpha if we found a better move
            if score > alpha {
                alpha = score;
                best_move = Some(chess_move);
            }
        }

        // Store the result in transposition table
        // Determine bound type based on whether we improved alpha
        let bound = if alpha > original_alpha {
            Bound::Exact // We found a move that improved our position
        } else {
            Bound::UpperBound // All moves failed low
        };
        tt.store(board.hash, depth, alpha, best_move, bound);

        alpha
    }

    /// Order moves to search promising moves first (improves alpha-beta pruning)
    /// Priority: TT best move first, then captures by victim value, then non-captures
    ///
    /// This method operates in-place on the provided buffer for better performance.
    fn order_moves(board: &Board, moves: &mut [ChessMove], tt: &mut TranspositionTable) {
        // Try to get best move from transposition table
        if let Some(entry) = tt.probe(board.hash)
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

    /// Enhanced move ordering with PV-first and history heuristic.
    ///
    /// Priority order:
    /// 1. PV move (from previous iteration)
    /// 2. TT move (from transposition table)
    /// 3. Captures sorted by MVV-LVA
    /// 4. Quiet moves sorted by history score
    ///
    /// This method operates in-place on the provided buffer for better performance.
    fn order_moves_with_history(
        board: &Board,
        moves: &mut [ChessMove],
        tt: &mut TranspositionTable,
        history: &HistoryTable,
        pv_move: Option<ChessMove>,
    ) {
        if moves.is_empty() {
            return;
        }

        let mut priority_index = 0;

        // 1. Try to place PV move first
        if let Some(pv) = pv_move
            && let Some(pos) = moves.iter().position(|&m| m == pv)
        {
            moves.swap(priority_index, pos);
            priority_index += 1;
        }

        // 2. Try to place TT move second (if different from PV move)
        if let Some(entry) = tt.probe(board.hash)
            && let Some(tt_move) = entry.best_move
            && (pv_move.is_none() || pv_move.unwrap() != tt_move)
            && let Some(pos) = moves[priority_index..].iter().position(|&m| m == tt_move)
        {
            moves.swap(priority_index, priority_index + pos);
            priority_index += 1;
        }

        // 3. Sort remaining moves by combined priority
        // (captures by MVV-LVA, quiet moves by history score)
        moves[priority_index..]
            .sort_by_key(|m| Self::move_priority_with_history(board, m, history));
    }

    /// Calculate move priority combining MVV-LVA for captures and history for quiet moves.
    fn move_priority_with_history(
        board: &Board,
        chess_move: &ChessMove,
        history: &HistoryTable,
    ) -> i32 {
        if chess_move.capture {
            // Captures: use MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
            let victim_value = if let Some((_, piece)) = board.piece_on(chess_move.to as u8) {
                Self::piece_value(piece)
            } else {
                100 // En passant captures a pawn
            };

            let attacker_value = if let Some((_, piece)) = board.piece_on(chess_move.from as u8) {
                Self::piece_value(piece)
            } else {
                0 // Shouldn't happen
            };

            // MVV-LVA: High victim value, low attacker value = best
            // victim_value * 10 - attacker_value ensures victim is primary factor
            // Negative for descending sort order
            // Add large offset to ensure captures are always before quiet moves
            -(victim_value * 10 - attacker_value + 100_000)
        } else {
            // Quiet moves: use history heuristic score
            // Negative to sort in descending order (higher history score = better)
            -history.get(chess_move)
        }
    }

    fn move_priority(board: &Board, chess_move: &ChessMove) -> i32 {
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
        let mut board = Board::new_empty();

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
        board.hash = crate::search::compute_hash_board(&board);

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
        MoveGenerator::generate_legal_moves(&test_board, &mut moves);
        assert!(
            moves.is_empty() && test_board.in_check(test_board.side_to_move),
            "Should deliver checkmate"
        );
    }

    #[test]
    fn test_finds_checkmate_fools_game() {
        let mut board = Board::startpos();
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
        MoveGenerator::generate_legal_moves(&test_board, &mut moves);
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
