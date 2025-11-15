use crate::board::{Board, Color, Piece};

pub struct ThreatenedEvaluator;

impl ThreatenedEvaluator {
    pub fn evaluate(board: &Board) -> i32 {
        let mut white_penalty: i32 = 0;
        let mut black_penalty: i32 = 0;

        // Evaluate each square on the board
        for (square_idx, square) in board.squares.iter().enumerate() {
            if let Some((piece, color)) = square.0 {
                // Don't evaluate king threats (king safety is handled elsewhere)
                if piece == Piece::King {
                    continue;
                }

                let opponent_color = color.opponent();

                // Count attackers and defenders
                let attackers = Self::count_attackers(board, square_idx, opponent_color);
                let defenders = Self::count_defenders(board, square_idx, color);

                if attackers > 0 {
                    let piece_value = Self::piece_value(piece);
                    let penalty = Self::calculate_threat_penalty(piece_value, attackers, defenders);

                    match color {
                        Color::White => white_penalty += penalty,
                        Color::Black => black_penalty += penalty,
                    }
                }
            }
        }

        // Return from white's perspective (penalties are negative)
        -white_penalty + black_penalty
    }

    /// Count how many pieces of the given color are attacking this square
    fn count_attackers(board: &Board, square: usize, attacker_color: Color) -> usize {
        let mut count = 0;

        for (i, sq) in board.squares.iter().enumerate() {
            if let Some((piece, color)) = sq.0
                && color == attacker_color
            {
                let moves = board.generate_piece_moves(i, piece, color);
                if moves.iter().any(|m| m.to == square) {
                    count += 1;
                }
            }
        }

        count
    }

    /// Count how many pieces of the given color are defending this square
    /// Note: We need to check if pieces *control* the square, not just if they can move there
    /// (since they can't move to squares occupied by friendly pieces, but they still defend them)
    fn count_defenders(board: &Board, square: usize, defender_color: Color) -> usize {
        let mut count = 0;

        for (i, sq) in board.squares.iter().enumerate() {
            // Don't count the piece defending itself
            if i == square {
                continue;
            }

            if let Some((piece, color)) = sq.0
                && color == defender_color
                && Self::piece_controls_square(board, i, piece, color, square)
            {
                count += 1;
            }
        }

        count
    }

    /// Check if a piece at a given position controls/attacks a target square
    /// This is different from generate_piece_moves because it includes squares with friendly pieces
    fn piece_controls_square(
        board: &Board,
        from: usize,
        piece: Piece,
        color: Color,
        target: usize,
    ) -> bool {
        // Temporarily place an enemy piece on the target to check if it can be "captured"
        let mut temp_board = *board;
        let enemy_color = color.opponent();
        temp_board.squares[target].0 = Some((Piece::Pawn, enemy_color)); // Use pawn as placeholder

        let moves = temp_board.generate_piece_moves(from, piece, color);
        moves.iter().any(|m| m.to == target)
    }

    /// Calculate penalty based on threat level
    /// Returns a positive penalty value (will be negated for the side being penalized)
    fn calculate_threat_penalty(piece_value: i32, attackers: usize, defenders: usize) -> i32 {
        if attackers == 0 {
            return 0;
        }

        // Hanging piece (attacked but not defended) - massive penalty
        if defenders == 0 {
            // Penalize by 80% of piece value - huge tactical problem
            return (piece_value * 80) / 100;
        }

        // More attackers than defenders - significant penalty
        if attackers > defenders {
            // Penalize by 40% of piece value - likely to be lost
            return (piece_value * 40) / 100;
        }

        // Equal attackers and defenders - small penalty
        // (Generally bad to have pieces under attack even if defended)
        if attackers == defenders {
            // Penalize by 15% of piece value
            return (piece_value * 15) / 100;
        }

        // More defenders than attackers - very small penalty
        // (Still slightly negative to encourage avoiding unnecessary tension)
        (piece_value * 5) / 100
    }

    fn piece_value(piece: Piece) -> i32 {
        match piece {
            Piece::Pawn => 100,
            Piece::Knight => 320,
            Piece::Bishop => 330,
            Piece::Rook => 500,
            Piece::Queen => 900,
            Piece::King => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_position_no_threats() {
        let board = Board::new();
        let score = ThreatenedEvaluator::evaluate(&board);

        // Starting position should have no hanging pieces
        assert_eq!(
            score, 0,
            "Starting position should have no threatened pieces"
        );
    }

    #[test]
    fn test_hanging_piece_penalty() {
        // Create a position with a hanging white knight
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("e8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("e4")].0 = Some((Piece::Knight, Color::White)); // Hanging knight
        board.squares[pos("c6")].0 = Some((Piece::Bishop, Color::Black)); // Attacks e4 diagonally
        board.white_king_pos = pos("e1");
        board.black_king_pos = pos("e8");
        board.side_to_move = Color::White;

        let score = ThreatenedEvaluator::evaluate(&board);

        // Should be negative (white has hanging piece)
        assert!(
            score < 0,
            "White should be penalized for hanging knight: {}",
            score
        );

        // Should be significant penalty (around 80% of knight value = ~256)
        assert!(score <= -200, "Penalty should be substantial: {}", score);
    }

    #[test]
    fn test_defended_piece_minimal_penalty() {
        // Create a position with an attacked but well-defended piece
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("e8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("e4")].0 = Some((Piece::Knight, Color::White)); // Knight
        board.squares[pos("d3")].0 = Some((Piece::Pawn, Color::White)); // Defends knight (pawns attack diagonally forward)
        board.squares[pos("c6")].0 = Some((Piece::Bishop, Color::Black)); // Attacks knight diagonally
        board.white_king_pos = pos("e1");
        board.black_king_pos = pos("e8");
        board.side_to_move = Color::White;

        let score = ThreatenedEvaluator::evaluate(&board);

        // Should be a small penalty (equal attackers/defenders)
        assert!(
            score < 0,
            "Should still have small penalty for piece under attack: {}",
            score
        );
        assert!(
            score > -100,
            "Penalty should be minimal when defended: {}",
            score
        );
    }

    #[test]
    fn test_more_attackers_than_defenders() {
        // Create a position where a piece is attacked twice but defended once
        let mut board = Board::empty();
        board.squares[pos("a1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("h8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("d4")].0 = Some((Piece::Rook, Color::White)); // Rook under attack
        board.squares[pos("c3")].0 = Some((Piece::Pawn, Color::White)); // Defends rook
        // Use knights to attack - they can't be counter-attacked by the rook easily
        board.squares[pos("c6")].0 = Some((Piece::Knight, Color::Black)); // Attacks d4 (knight move: 1 file, 2 ranks)
        board.squares[pos("e6")].0 = Some((Piece::Knight, Color::Black)); // Also attacks d4
        // Make sure the knights are safe from the rook
        board.squares[pos("b7")].0 = Some((Piece::Pawn, Color::Black)); // Defends c6 knight
        board.squares[pos("f7")].0 = Some((Piece::Pawn, Color::Black)); // Defends e6 knight
        board.white_king_pos = pos("a1");
        board.black_king_pos = pos("h8");
        board.side_to_move = Color::White;

        let score = ThreatenedEvaluator::evaluate(&board);

        // Should be a significant penalty (more attackers than defenders)
        // Rook value 500, with 2 attackers and 1 defender = 40% penalty = 200
        assert!(
            score <= -150,
            "Should have significant penalty when outnumbered: {}",
            score
        );
    }

    fn pos(s: &str) -> usize {
        let bytes = s.as_bytes();
        let file = (bytes[0] - b'a') as usize;
        let rank = (bytes[1] - b'1') as usize;
        rank * 8 + file
    }
}
