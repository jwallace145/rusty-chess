use super::{Color, Piece};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChessMoveType {
    Normal,
    Castle,
    EnPassant,
    Promotion(Piece),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    pub previous_halfmove_clock: u8,
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

    /// Converts a ChessMove to standard algebraic notation for castling (O-O or O-O-O)
    /// Returns None if this is not a castling move
    pub fn to_castling_notation(&self) -> Option<&'static str> {
        if self.move_type != ChessMoveType::Castle {
            return None;
        }

        // Kingside castling: king moves to g-file (file 6)
        // Queenside castling: king moves to c-file (file 2)
        let to_file = self.to % 8;
        match to_file {
            6 => Some("O-O"),   // Kingside
            2 => Some("O-O-O"), // Queenside
            _ => None,
        }
    }

    /// Returns a display-friendly string representation of the move
    /// Uses O-O/O-O-O for castling, standard notation otherwise
    pub fn to_display(&self) -> String {
        if let Some(castling) = self.to_castling_notation() {
            castling.to_string()
        } else {
            self.to_uci()
        }
    }
}
