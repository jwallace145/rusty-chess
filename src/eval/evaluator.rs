use crate::board::{Board, Color, Piece};

/// Evaluates chess board positions to guide the minimax search algorithm.
///
/// Converts board states into numerical scores by combining two components:
///
/// - **Material**: Assigns values to pieces (e.g., Queen=900, Rook=500, Pawn=100)
///   to encourage maintaining strong pieces
/// - **Positional**: Rewards pieces for occupying favorable squares using
///   piece-square tables (e.g., knights in the center, pawns advancing)
///
/// Returns positive scores when White is winning, negative when Black is winning.
pub struct Evaluator;

impl Evaluator {
    pub fn evaluate(board: &Board) -> i32 {
        let material = Self::material_count(board);
        let positional = Self::positional_score(board);

        let total = material + positional;

        // Return score from side to move's perspective
        match board.side_to_move {
            Color::White => total,
            Color::Black => -total,
        }
    }

    fn material_count(board: &Board) -> i32 {
        let mut white_material = 0;
        let mut black_material = 0;

        for square in &board.squares {
            if let Some((piece, color)) = square.0 {
                let value = match piece {
                    Piece::Pawn => 100,
                    Piece::Knight => 320,
                    Piece::Bishop => 330,
                    Piece::Rook => 500,
                    Piece::Queen => 900,
                    Piece::King => 0,
                };

                match color {
                    Color::White => white_material += value,
                    Color::Black => black_material += value,
                }
            }
        }

        white_material - black_material
    }

    fn positional_score(board: &Board) -> i32 {
        let mut white_position = 0;
        let mut black_position = 0;

        for (square, sq) in board.squares.iter().enumerate() {
            if let Some((piece, color)) = sq.0 {
                let bonus = Self::piece_square_value(piece, square, color);

                match color {
                    Color::White => white_position += bonus,
                    Color::Black => black_position += bonus,
                }
            }
        }

        white_position - black_position
    }

    fn piece_square_value(piece: Piece, square: usize, color: Color) -> i32 {
        // For black pieces, flip the board vertically
        let sq = match color {
            Color::White => square,
            Color::Black => square ^ 56, // Flip rank: XOR with 56
        };

        match piece {
            Piece::Pawn => PAWN_TABLE[sq],
            Piece::Knight => KNIGHT_TABLE[sq],
            Piece::Bishop => BISHOP_TABLE[sq],
            Piece::Rook => ROOK_TABLE[sq],
            Piece::Queen => QUEEN_TABLE[sq],
            Piece::King => KING_MIDDLEGAME_TABLE[sq],
        }
    }
}

// Piece-Square Tables (from white's perspective)
// Values are in centipawns (1/100 of a pawn)

const PAWN_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5, 5,
    10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10, -20,
    -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
];

const KNIGHT_TABLE: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15, 10,
    0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15, 15, 10,
    5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
];

const BISHOP_TABLE: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5, 0,
    -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10, 10, 10,
    -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
];

const ROOK_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0,
    0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 0, 0,
    0, 5, 5, 0, 0, 0,
];

const QUEEN_TABLE: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0, 5, 0, 0,
    0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
];

const KING_MIDDLEGAME_TABLE: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40,
    -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40, -40, -30,
    -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20, 30, 10, 0, 0,
    10, 30, 20,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_position_equal() {
        let board = Board::new();
        let score = Evaluator::evaluate(&board);
        // Should be close to 0 (slight advantage for white having first move)
        assert!(
            score.abs() < 50,
            "Starting position should be roughly equal"
        );
    }

    #[test]
    fn test_center_knight_better_than_edge() {
        let mut board = Board::empty();
        board.squares[pos("e4")].0 = Some((Piece::Knight, Color::White));
        board.squares[pos("e8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.white_king_pos = pos("e1");
        board.black_king_pos = pos("e8");
        board.side_to_move = Color::White;

        let center_score = Evaluator::evaluate(&board);

        let mut board2 = Board::empty();
        board2.squares[pos("a1")].0 = Some((Piece::Knight, Color::White));
        board2.squares[pos("e8")].0 = Some((Piece::King, Color::Black));
        board2.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board2.white_king_pos = pos("e1");
        board2.black_king_pos = pos("e8");
        board2.side_to_move = Color::White;

        let edge_score = Evaluator::evaluate(&board2);

        assert!(
            center_score > edge_score,
            "Knight in center should score higher"
        );
    }

    fn pos(s: &str) -> usize {
        let bytes = s.as_bytes();
        let file = (bytes[0] - b'a') as usize;
        let rank = (bytes[1] - b'1') as usize;
        rank * 8 + file
    }
}
