use crate::{
    board::{Board, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

const OPEN_FILE_BONUS: i32 = 20;
const SEMI_OPEN_FILE_BONUS: i32 = 10;

/// Evaluates rooks on open or semi-open files.
/// - **Open file**: no pawns of either color on the file → +20
/// - **Semi-open file**: no friendly pawns on the file → +10
///
///   Positive for White, negative for Black.
pub struct RookFileEvaluator;

impl BoardEvaluator for RookFileEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        let mut score = 0;

        for (sq, square) in board.squares.iter().enumerate() {
            if let Some((piece, color)) = square.0
                && piece == Piece::Rook
            {
                score += Self::rook_file_score(board, sq, color);
            }
        }

        score
    }
}

impl RookFileEvaluator {
    fn rook_file_score(board: &Board, sq: usize, color: Color) -> i32 {
        let file = sq % 8;
        let mut has_friendly_pawn = false;
        let mut has_enemy_pawn = false;

        for rank in 0..8 {
            let idx = rank * 8 + file;
            if let Some((piece, piece_color)) = board.squares[idx].0
                && piece == Piece::Pawn
            {
                if piece_color == color {
                    has_friendly_pawn = true;
                } else {
                    has_enemy_pawn = true;
                }
            }
        }

        let bonus = if !has_friendly_pawn && !has_enemy_pawn {
            OPEN_FILE_BONUS
        } else if !has_friendly_pawn {
            SEMI_OPEN_FILE_BONUS
        } else {
            0
        };

        if color == Color::White { bonus } else { -bonus }
    }
}
