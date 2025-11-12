use crate::board::{Board, Color, Piece};

pub struct PositionEvaluator;

impl PositionEvaluator {
    pub fn evaluate(board: &Board) -> i32 {
        let mut white_position: i32 = 0;
        let mut black_position: i32 = 0;

        let game_phase: i32 = Self::game_phase(board);

        for (position, square) in board.squares.iter().enumerate() {
            if let Some((piece, color)) = square.0 {
                let bonus: i32 = Self::piece_value(piece, position, color, game_phase);

                match color {
                    Color::White => white_position += bonus,
                    Color::Black => black_position += bonus,
                }
            }
        }

        white_position - black_position
    }

    fn piece_value(piece: Piece, position: usize, color: Color, game_phase: i32) -> i32 {
        // For Black pieces, flip the board vertically to normalize Piece-Square tables
        let normalized_position: usize = match color {
            Color::White => position,
            Color::Black => position ^ 56, // Flip rank (XOR with 56)
        };

        match piece {
            Piece::Pawn => PAWN_TABLE[normalized_position],
            Piece::Knight => KNIGHT_TABLE[normalized_position],
            Piece::Bishop => BISHOP_TABLE[normalized_position],
            Piece::Rook => ROOK_TABLE[normalized_position],
            Piece::Queen => QUEEN_TABLE[normalized_position],
            Piece::King => {
                let mg = KING_MIDDLEGAME_TABLE[normalized_position];
                let eg = KING_ENDGAME_TABLE[normalized_position];
                // Blend middle game and endgame king piece-square table positioning
                // proportionally to the phase of the game (early, mid, late)
                (mg * game_phase + eg * (256 - game_phase)) / 256
            }
        }
    }

    fn game_phase(board: &Board) -> i32 {
        const MAX_PHASE: i32 = 24; // Sum of all piece phase values

        let mut phase: i32 = 0;

        for square in &board.squares {
            if let Some((piece, _color)) = square.0 {
                phase += Self::piece_phase_value(piece);
            }
        }

        (phase * 256 + MAX_PHASE / 2) / MAX_PHASE
    }

    fn piece_phase_value(piece: Piece) -> i32 {
        match piece {
            Piece::Knight => 1,
            Piece::Bishop => 1,
            Piece::Rook => 2,
            Piece::Queen => 4,
            Piece::Pawn | Piece::King => 0,
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

const KING_ENDGAME_TABLE: [i32; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50, -30, -20, -10, 0, 0, -10, -20, -30, -30, -10, 20, 30,
    30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30,
    -10, 20, 30, 30, 20, -10, -30, -30, -30, 0, 0, 0, 0, -30, -30, -50, -30, -30, -30, -30, -30,
    -30, -50,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_evaluator_piece_game_phase_values() {
        let value: i32 = PositionEvaluator::piece_phase_value(Piece::Pawn);
        let expected_value: i32 = 0;
        assert_eq!(value, expected_value, "Pawn should have phase value 0");

        let value: i32 = PositionEvaluator::piece_phase_value(Piece::Knight);
        let expected_value: i32 = 1;
        assert_eq!(value, expected_value, "Knight should have phase value 1");

        let value: i32 = PositionEvaluator::piece_phase_value(Piece::Bishop);
        let expected_value: i32 = 1;
        assert_eq!(value, expected_value, "Bishop should have phase value 1");

        let value: i32 = PositionEvaluator::piece_phase_value(Piece::Rook);
        let expected_value: i32 = 2;
        assert_eq!(value, expected_value, "Rook should have phase value 2");

        let value: i32 = PositionEvaluator::piece_phase_value(Piece::Queen);
        let expected_value: i32 = 4;
        assert_eq!(value, expected_value, "Queen should have phase value 4");

        let value: i32 = PositionEvaluator::piece_phase_value(Piece::King);
        let expected_value: i32 = 0;
        assert_eq!(value, expected_value, "King should have phase value 0");
    }

    #[test]
    fn test_position_evaluator_detect_game_phase_early() {
        let board: Board = Board::default();

        let value: i32 = PositionEvaluator::game_phase(&board);
        let expected_value: i32 = 256;
        assert_eq!(
            value, expected_value,
            "Initial board state should map to early game phase"
        )
    }

    #[test]
    fn test_position_evaluator_detect_game_phase_mid() {
        let mut board: Board = Board::default();

        // Remove all Rooks
        board.squares[0].0 = None;
        board.squares[7].0 = None;
        board.squares[63].0 = None;
        board.squares[56].0 = None;

        let value: i32 = PositionEvaluator::game_phase(&board);
        let expected_value: i32 = 171;
        assert_eq!(value, expected_value, "Should detect mid game phase")
    }

    #[test]
    fn test_position_evaluator_detect_game_phase_late() {
        let mut board: Board = Board::default();

        // Remove all Rooks and Queens
        board.squares[0].0 = None;
        board.squares[7].0 = None;
        board.squares[63].0 = None;
        board.squares[56].0 = None;
        board.squares[3].0 = None;
        board.squares[59].0 = None;

        let value: i32 = PositionEvaluator::game_phase(&board);
        let expected_value: i32 = 85;
        assert_eq!(value, expected_value, "Should detect late game phase")
    }
}
