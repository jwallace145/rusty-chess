use super::{Color, Piece};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ChessMoveType {
    Normal,
    Castle,
    EnPassant,
    Promotion(Piece),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChessMove {
    pub from: usize,
    pub to: usize,
    pub capture: bool,
    pub moveType: ChessMoveType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChessMoveState {
    pub chess_move: ChessMove,
    pub moved_piece: Option<(Piece, Color)>,
    pub captured_piece: Option<(Piece, Color)>,
    pub previous_side_to_move: Color,
    pub white_king_moved: bool,
    pub white_kingside_rook_moved: bool,
    pub white_queenside_rook_moved: bool,
    pub black_king_moved: bool,
    pub black_kingside_rook_moved: bool,
    pub black_queenside_rook_moved: bool,
}
