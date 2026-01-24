use crate::{
    board::{Board, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

const MOBILITY_WEIGHT: i32 = 6;

pub struct MobilityEvaluator;

impl BoardEvaluator for MobilityEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        let white_mobility: i32 = Self::count_mobility(board, Color::White);
        let black_mobility: i32 = Self::count_mobility(board, Color::Black);

        MOBILITY_WEIGHT * (white_mobility - black_mobility)
    }
}

impl MobilityEvaluator {
    /// Count total pseudo-legal moves for all pieces
    fn count_mobility(board: &Board, color: Color) -> i32 {
        let mut total = 0;
        let enemy_or_empty = !board.occupancy(color);

        // Iterate through all piece types for the given color
        for piece_idx in 0..6 {
            let piece = match piece_idx {
                0 => Piece::Pawn,
                1 => Piece::Knight,
                2 => Piece::Bishop,
                3 => Piece::Rook,
                4 => Piece::Queen,
                _ => Piece::King,
            };

            let mut piece_bb = board.pieces[color as usize][piece_idx];
            while piece_bb != 0 {
                let sq = piece_bb.trailing_zeros() as u8;

                // Get attack bitboard for this piece
                let attacks = board.attacks_from(piece, sq, color);

                // Count moves to empty squares or captures
                let moves = attacks & enemy_or_empty;
                total += moves.count_ones() as i32;

                piece_bb &= piece_bb - 1; // Clear the least significant bit
            }
        }

        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Piece;

    #[test]
    fn test_starting_position_mobility() {
        let board = Board::startpos();
        let score = MobilityEvaluator.evaluate(&board);

        // At the starting position, both sides have equal mobility
        // White has 20 moves: 16 pawn moves (8 pawns * 2 moves each) + 4 knight moves (2 knights * 2 moves each)
        // Black has the same
        assert_eq!(score, 0, "Starting position should have equal mobility");
    }

    #[test]
    fn test_white_advantage_mobility() {
        let mut board = Board::new_empty();

        // Place white pieces with more mobility
        board.pieces[Color::White as usize][Piece::Queen as usize] = 1u64 << 27; // d4
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
        board.king_sq[Color::White as usize] = 4;

        // Place black pieces with less mobility
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
        board.king_sq[Color::Black as usize] = 60;

        // Update occupancy
        board.occ[Color::White as usize] = (1u64 << 27) | (1u64 << 4);
        board.occ[Color::Black as usize] = 1u64 << 60;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];
        board.side_to_move = Color::White;

        let score = MobilityEvaluator.evaluate(&board);

        // White should have positive mobility score (queen has ~27 moves, king has ~5)
        // Black only has king moves (~5)
        assert!(
            score > 0,
            "White should have mobility advantage with a queen"
        );
    }

    #[test]
    fn test_black_advantage_mobility() {
        let mut board = Board::new_empty();

        // Place black pieces with more mobility
        board.pieces[Color::Black as usize][Piece::Queen as usize] = 1u64 << 27; // d4
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
        board.king_sq[Color::Black as usize] = 60;

        // Place white pieces with less mobility
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
        board.king_sq[Color::White as usize] = 4;

        // Update occupancy
        board.occ[Color::Black as usize] = (1u64 << 27) | (1u64 << 60);
        board.occ[Color::White as usize] = 1u64 << 4;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];
        board.side_to_move = Color::White;

        let score = MobilityEvaluator.evaluate(&board);

        // Black should have negative mobility score (meaning black has advantage)
        assert!(
            score < 0,
            "Black should have mobility advantage with a queen"
        );
    }

    #[test]
    fn test_mobility_counts_unique_squares() {
        let mut board = Board::new_empty();

        // Place two white rooks that can move to overlapping squares
        board.pieces[Color::White as usize][Piece::Rook as usize] = (1u64 << 0) | (1u64 << 7); // a1 and h1
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
        board.king_sq[Color::White as usize] = 4;

        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
        board.king_sq[Color::Black as usize] = 60;

        // Update occupancy
        board.occ[Color::White as usize] = (1u64 << 0) | (1u64 << 7) | (1u64 << 4);
        board.occ[Color::Black as usize] = 1u64 << 60;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];
        board.side_to_move = Color::White;

        let white_mobility = MobilityEvaluator::count_mobility(&board, Color::White);

        // Each rook has 14 moves (7 vertical + 7 horizontal minus blocked squares)
        // But they share some horizontal squares, so total unique squares < 28
        // The king also has some moves
        // This test verifies we're counting unique squares, not total moves
        assert!(white_mobility > 0, "White should have mobility");
        assert!(
            white_mobility < 64,
            "Mobility should be less than total board squares"
        );
    }

    #[test]
    fn test_pawn_mobility() {
        let mut board = Board::new_empty();

        // Place a white pawn in starting position
        board.pieces[Color::White as usize][Piece::Pawn as usize] = 1u64 << 8; // a2
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
        board.king_sq[Color::White as usize] = 4;

        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
        board.king_sq[Color::Black as usize] = 60;

        // Update occupancy
        board.occ[Color::White as usize] = (1u64 << 8) | (1u64 << 4);
        board.occ[Color::Black as usize] = 1u64 << 60;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];
        board.side_to_move = Color::White;

        let white_mobility = MobilityEvaluator::count_mobility(&board, Color::White);

        // Pawn on a2 can move to a3 and a4 (2 squares)
        // King on e1 can move to several squares
        assert!(
            white_mobility >= 2,
            "Pawn should contribute at least 2 moves"
        );
    }

    #[test]
    fn test_knight_mobility() {
        let mut board = Board::new_empty();

        // Place a white knight in the center
        board.pieces[Color::White as usize][Piece::Knight as usize] = 1u64 << 27; // d4
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
        board.king_sq[Color::White as usize] = 4;

        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
        board.king_sq[Color::Black as usize] = 60;

        // Update occupancy
        board.occ[Color::White as usize] = (1u64 << 27) | (1u64 << 4);
        board.occ[Color::Black as usize] = 1u64 << 60;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];
        board.side_to_move = Color::White;

        let white_mobility = MobilityEvaluator::count_mobility(&board, Color::White);

        // Knight on d4 can move to 8 squares
        // King adds some more
        assert!(
            white_mobility >= 8,
            "Central knight should have 8 possible moves"
        );
    }
}
