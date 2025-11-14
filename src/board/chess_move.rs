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
    pub move_type: ChessMoveType,
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
    pub previous_en_passant: Option<usize>,
    pub previous_zobrist_hash: u64,
}

impl ChessMove {
    /// Converts a ChessMove to UCI notation (e.g., "e2e4", "e7e8q")
    pub fn to_uci(&self) -> String {
        let from_file = (self.from % 8) as u8;
        let from_rank = (self.from / 8) as u8;
        let to_file = (self.to % 8) as u8;
        let to_rank = (self.to / 8) as u8;

        let mut uci = format!(
            "{}{}{}{}",
            (b'a' + from_file) as char,
            (b'1' + from_rank) as char,
            (b'a' + to_file) as char,
            (b'1' + to_rank) as char
        );

        // Add promotion piece if applicable
        if let ChessMoveType::Promotion(piece) = self.move_type {
            let piece_char = match piece {
                Piece::Queen => 'q',
                Piece::Rook => 'r',
                Piece::Bishop => 'b',
                Piece::Knight => 'n',
                _ => unreachable!("Invalid promotion piece"),
            };
            uci.push(piece_char);
        }

        uci
    }
}
