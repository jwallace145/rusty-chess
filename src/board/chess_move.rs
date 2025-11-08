use super::{Color, Piece};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChessMove {
    pub from: usize,
    pub to: usize,
    pub capture: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChessMoveState {
    pub chess_move: ChessMove,
    pub moved_piece: Option<(Piece, Color)>,
    pub captured_piece: Option<(Piece, Color)>,
    pub previous_side_to_move: Color,
}
