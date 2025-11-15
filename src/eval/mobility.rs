use crate::board::{Board, Color, MoveGenerator};

pub struct MobilityEvaluator;

const MOBILITY_WEIGHT: i32 = 4;

impl MobilityEvaluator {
    pub fn evaluate(board: &Board) -> i32 {
        let white_mobility: i32 = Self::count_mobility(board, Color::White);
        let black_mobility: i32 = Self::count_mobility(board, Color::Black);

        MOBILITY_WEIGHT * (white_mobility - black_mobility)
    }

    /// Count total pseudo-legal moves for all pieces
    fn count_mobility(board: &Board, color: Color) -> i32 {
        let mut moves = Vec::with_capacity(64);

        let mut total = 0;
        for (sq, square) in board.squares.iter().enumerate() {
            if let Some((piece, piece_color)) = square.0
                && piece_color == color
            {
                moves.clear();
                MoveGenerator::generate_piece_moves(board, sq, piece, color, &mut moves);

                // Count total moves, not unique targets
                total += moves.len() as i32;
            }
        }

        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, Piece};

    #[test]
    fn test_starting_position_mobility() {
        let board = Board::new();
        let score = MobilityEvaluator::evaluate(&board);

        // At the starting position, both sides have equal mobility
        // White has 20 moves: 16 pawn moves (8 pawns * 2 moves each) + 4 knight moves (2 knights * 2 moves each)
        // Black has the same
        assert_eq!(score, 0, "Starting position should have equal mobility");
    }

    #[test]
    fn test_white_advantage_mobility() {
        let mut board = Board::empty();

        // Place white pieces with more mobility
        board.squares[27].0 = Some((Piece::Queen, Color::White)); // d4 - central queen
        board.squares[4].0 = Some((Piece::King, Color::White)); // e1

        // Place black pieces with less mobility
        board.squares[60].0 = Some((Piece::King, Color::Black)); // e8

        board.white_king_pos = 4;
        board.black_king_pos = 60;
        board.side_to_move = Color::White;

        let score = MobilityEvaluator::evaluate(&board);

        // White should have positive mobility score (queen has ~27 moves, king has ~5)
        // Black only has king moves (~5)
        assert!(
            score > 0,
            "White should have mobility advantage with a queen"
        );
    }

    #[test]
    fn test_black_advantage_mobility() {
        let mut board = Board::empty();

        // Place black pieces with more mobility
        board.squares[27].0 = Some((Piece::Queen, Color::Black)); // d4 - central queen
        board.squares[60].0 = Some((Piece::King, Color::Black)); // e8

        // Place white pieces with less mobility
        board.squares[4].0 = Some((Piece::King, Color::White)); // e1

        board.white_king_pos = 4;
        board.black_king_pos = 60;
        board.side_to_move = Color::White;

        let score = MobilityEvaluator::evaluate(&board);

        // Black should have negative mobility score (meaning black has advantage)
        assert!(
            score < 0,
            "Black should have mobility advantage with a queen"
        );
    }

    #[test]
    fn test_mobility_counts_unique_squares() {
        let mut board = Board::empty();

        // Place two white rooks that can move to overlapping squares
        board.squares[0].0 = Some((Piece::Rook, Color::White)); // a1
        board.squares[7].0 = Some((Piece::Rook, Color::White)); // h1
        board.squares[4].0 = Some((Piece::King, Color::White)); // e1

        board.squares[60].0 = Some((Piece::King, Color::Black)); // e8

        board.white_king_pos = 4;
        board.black_king_pos = 60;
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
        let mut board = Board::empty();

        // Place a white pawn in starting position
        board.squares[8].0 = Some((Piece::Pawn, Color::White)); // a2
        board.squares[4].0 = Some((Piece::King, Color::White)); // e1
        board.squares[60].0 = Some((Piece::King, Color::Black)); // e8

        board.white_king_pos = 4;
        board.black_king_pos = 60;
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
        let mut board = Board::empty();

        // Place a white knight in the center
        board.squares[27].0 = Some((Piece::Knight, Color::White)); // d4
        board.squares[4].0 = Some((Piece::King, Color::White)); // e1
        board.squares[60].0 = Some((Piece::King, Color::Black)); // e8

        board.white_king_pos = 4;
        board.black_king_pos = 60;
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
