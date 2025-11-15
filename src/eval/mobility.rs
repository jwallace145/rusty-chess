use crate::board::{Board, Color};

/// Evaluates board positions based on piece mobility (number of legal moves available).
///
/// Mobility is a strong indicator of position strength:
/// - More moves = more options and flexibility
/// - Restricted mobility often indicates a cramped or weak position
/// - Helps the search algorithm identify active vs passive positions
pub struct MobilityEvaluator;

impl MobilityEvaluator {
    /// Evaluates the mobility difference between White and Black.
    ///
    /// Returns a positive score when White has more mobility,
    /// negative when Black has more mobility.
    pub fn evaluate(board: &Board) -> i32 {
        // Count legal moves for White
        let mut white_board = *board;
        white_board.side_to_move = Color::White;
        let white_mobility = white_board.generate_moves().len() as i32;

        // Count legal moves for Black
        let mut black_board = *board;
        black_board.side_to_move = Color::Black;
        let black_mobility = black_board.generate_moves().len() as i32;

        // Weight the mobility difference
        // A value of 10 means each extra legal move is worth 0.1 pawns (10 centipawns)
        const MOBILITY_WEIGHT: i32 = 10;

        (white_mobility - black_mobility) * MOBILITY_WEIGHT
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Piece;

    #[test]
    fn test_mobility_initial_position() {
        let board = Board::default();
        let score = MobilityEvaluator::evaluate(&board);
        // In the starting position, both sides have 20 legal moves
        // So the mobility score should be 0
        assert_eq!(score, 0, "Initial position should have equal mobility");
    }

    #[test]
    fn test_mobility_white_advantage() {
        // Create a position where White has more mobility
        let mut board = Board::empty();

        // Place kings (required for legal position)
        board.squares[4].0 = Some((Piece::King, Color::White)); // e1
        board.squares[60].0 = Some((Piece::King, Color::Black)); // e8
        board.white_king_pos = 4;
        board.black_king_pos = 60;

        // White queen in center (lots of moves)
        board.squares[27].0 = Some((Piece::Queen, Color::White)); // d4

        // Black queen trapped in corner
        board.squares[63].0 = Some((Piece::Queen, Color::Black)); // h8
        board.squares[62].0 = Some((Piece::Pawn, Color::Black)); // g8
        board.squares[55].0 = Some((Piece::Pawn, Color::Black)); // h7
        board.squares[54].0 = Some((Piece::Pawn, Color::Black)); // g7

        board.side_to_move = Color::White;

        let score = MobilityEvaluator::evaluate(&board);
        assert!(score > 0, "White should have better mobility");
    }

    #[test]
    fn test_mobility_black_advantage() {
        // Create a position where Black has more mobility
        let mut board = Board::empty();

        // Place kings
        board.squares[4].0 = Some((Piece::King, Color::White)); // e1
        board.squares[60].0 = Some((Piece::King, Color::Black)); // e8
        board.white_king_pos = 4;
        board.black_king_pos = 60;

        // Black queen in center (lots of moves)
        board.squares[35].0 = Some((Piece::Queen, Color::Black)); // d5

        // White queen trapped
        board.squares[0].0 = Some((Piece::Queen, Color::White)); // a1
        board.squares[1].0 = Some((Piece::Pawn, Color::White)); // b1
        board.squares[8].0 = Some((Piece::Pawn, Color::White)); // a2

        board.side_to_move = Color::White;

        let score = MobilityEvaluator::evaluate(&board);
        assert!(score < 0, "Black should have better mobility");
    }

    #[test]
    fn test_mobility_cramped_position() {
        // Test that a cramped position (lots of pieces blocking each other)
        // has less mobility than an open position
        let cramped_board = Board::default();

        let mut open_board = Board::default();
        // Remove some central pawns to open up the position
        open_board.squares[11].0 = None; // d2 white pawn
        open_board.squares[12].0 = None; // e2 white pawn
        open_board.squares[51].0 = None; // d7 black pawn
        open_board.squares[52].0 = None; // e7 black pawn

        let cramped_white_moves = {
            let mut board = cramped_board;
            board.side_to_move = Color::White;
            board.generate_moves().len()
        };

        let open_white_moves = {
            let mut board = open_board;
            board.side_to_move = Color::White;
            board.generate_moves().len()
        };

        assert!(
            open_white_moves > cramped_white_moves,
            "Open position should have more mobility than cramped position"
        );
    }
}
