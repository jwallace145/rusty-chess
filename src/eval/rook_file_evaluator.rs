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

        // Iterate through white rooks
        let mut white_rooks = board.pieces[Color::White as usize][Piece::Rook as usize];
        while white_rooks != 0 {
            let sq = white_rooks.trailing_zeros() as usize;
            score += Self::rook_file_score(board, sq, Color::White);
            white_rooks &= white_rooks - 1;
        }

        // Iterate through black rooks
        let mut black_rooks = board.pieces[Color::Black as usize][Piece::Rook as usize];
        while black_rooks != 0 {
            let sq = black_rooks.trailing_zeros() as usize;
            score += Self::rook_file_score(board, sq, Color::Black);
            black_rooks &= black_rooks - 1;
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
            let idx = (rank * 8 + file) as u8;
            if let Some((piece_color, piece)) = board.piece_on(idx)
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
