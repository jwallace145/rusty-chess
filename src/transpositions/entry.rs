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
