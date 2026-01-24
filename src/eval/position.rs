use crate::{
    board::{Board, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

pub struct PositionEvaluator;

impl BoardEvaluator for PositionEvaluator {
    // Evaluate the positional score of a chess board state
    fn evaluate(&self, board: &Board) -> i32 {
        let mut white_position: i32 = 0;
        let mut black_position: i32 = 0;

        let game_phase: i32 = Self::game_phase(board);

        // Iterate through both colors
        for color in [Color::White, Color::Black] {
            for piece_idx in 0..6 {
                let piece = match piece_idx {
                    0 => Piece::Pawn,
                    1 => Piece::Knight,
                    2 => Piece::Bishop,
                    3 => Piece::Rook,
                    4 => Piece::Queen,
                    _ => Piece::King,
                };

                let mut bitboard = board.pieces[color as usize][piece_idx];
                while bitboard != 0 {
                    let square = bitboard.trailing_zeros() as usize;
                    bitboard &= bitboard - 1; // Clear the least significant bit

                    let bonus = Self::piece_value(piece, square, color, game_phase);
                    match color {
                        Color::White => white_position += bonus,
                        Color::Black => black_position += bonus,
                    }
                }
            }
        }

        white_position - black_position
    }
}

impl PositionEvaluator {
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

        // Count pieces for both colors
        for color in [Color::White, Color::Black] {
            phase += board.count_pieces(color, Piece::Knight) as i32;
            phase += board.count_pieces(color, Piece::Bishop) as i32;
            phase += board.count_pieces(color, Piece::Rook) as i32 * 2;
            phase += board.count_pieces(color, Piece::Queen) as i32 * 4;
        }

        (phase * 256 + MAX_PHASE / 2) / MAX_PHASE
    }
}

// Piece-Square Tables (from white's perspective)
// Values are in centipawns (1/100 of a pawn)

const PAWN_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5, 5,
    10, 27, 27, 10, 5, 5, 0, 0, 0, 25, 25, 0, 0, 0, 5, -5, -10, 10, 10, -10, -5, 5, 50, 50, 50, 50,
    50, 50, 50, 50, 200, 200, 200, 200, 200, 200, 200, 200,
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
    0, 0, 0, 5, 5, 0, 0, 0, // Back rank
    -5, 0, 0, 0, 0, 0, 0, -5, // Rank 2
    -5, 0, 0, 0, 0, 0, 0, -5, // Rank 3
    -5, 0, 0, 0, 0, 0, 0, -5, // Rank 4
    -5, 0, 0, 0, 0, 0, 0, -5, // Rank 5
    -5, 0, 0, 0, 0, 0, 0, -5, // Rank 6
    5, 10, 10, 10, 10, 10, 10, 5, // Rank 7 (7th rank bonus!)
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 8
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
    fn test_position_evaluator_evaluate_initial_position() {
        let board: Board = Board::default();
        let value: i32 = PositionEvaluator.evaluate(&board);
        let expected_value: i32 = 0;
        assert_eq!(
            value, expected_value,
            "Initial position should evaluate to positional score of 0"
        );
    }

    // Test removed - piece_phase_value function was inlined into game_phase()

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

        // Remove all Rooks by clearing their bitboards
        board.pieces[Color::White as usize][Piece::Rook as usize] = 0;
        board.pieces[Color::Black as usize][Piece::Rook as usize] = 0;

        // Update occupancy
        board.occ[Color::White as usize] =
            board.pieces[Color::White as usize].iter().copied().sum();
        board.occ[Color::Black as usize] =
            board.pieces[Color::Black as usize].iter().copied().sum();
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        let value: i32 = PositionEvaluator::game_phase(&board);
        let expected_value: i32 = 171;
        assert_eq!(value, expected_value, "Should detect mid game phase")
    }

    #[test]
    fn test_position_evaluator_detect_game_phase_late() {
        let mut board: Board = Board::default();

        // Remove all Rooks and Queens
        board.pieces[Color::White as usize][Piece::Rook as usize] = 0;
        board.pieces[Color::Black as usize][Piece::Rook as usize] = 0;
        board.pieces[Color::White as usize][Piece::Queen as usize] = 0;
        board.pieces[Color::Black as usize][Piece::Queen as usize] = 0;

        // Update occupancy
        board.occ[Color::White as usize] =
            board.pieces[Color::White as usize].iter().copied().sum();
        board.occ[Color::Black as usize] =
            board.pieces[Color::Black as usize].iter().copied().sum();
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        let value: i32 = PositionEvaluator::game_phase(&board);
        let expected_value: i32 = 85;
        assert_eq!(value, expected_value, "Should detect late game phase")
    }
}
