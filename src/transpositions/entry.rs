use crate::board::ChessMove;

/// Type of bound stored in a transposition table entry.
///
/// In alpha-beta search, we may not always get an exact score:
/// - `Exact`: The score is exact (score was between alpha and beta)
/// - `LowerBound`: Fail-high, actual score is >= stored score (score >= beta)
/// - `UpperBound`: Fail-low, actual score is <= stored score (score <= alpha)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Bound {
    Exact,
    LowerBound,
    UpperBound,
}

/// A cached evaluation of a chess position.
///
/// # Fields
/// - `hash`: Zobrist hash of the position (0 indicates empty entry)
/// - `depth`: Search depth at which this position was evaluated
/// - `score`: Evaluated score of the position
/// - `best_move`: Best move found from this position, if any
/// - `bound`: Type of score bound (exact, lower bound, or upper bound)
#[derive(Copy, Clone)]
pub struct TTEntry {
    pub hash: u64,
    pub depth: u8,
    pub score: i32,
    pub best_move: Option<ChessMove>,
    pub bound: Bound,
}
