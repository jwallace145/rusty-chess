use crate::board::{Board, Color, Piece};

pub struct Evaluator;

impl Evaluator {
    pub fn evaluate(board: &Board) -> i32 {
        let material = Self::material_count(board);

        match board.side_to_move {
            Color::White => material,
            Color::Black => -material,
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
                };
            }
        }

        white_material - black_material
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_position_equal() {
        let board = Board::new();
        let score = Evaluator::material_count(&board);
        assert_eq!(score, 0, "Starting position evaluation should be equal")
    }

    #[test]
    fn test_white_winning_position() {
        let mut board = Board::empty();

        // White pieces
        let piece1 = Piece::Queen;
        let from1 = pos("d4");
        let piece2 = Piece::Rook;
        let from2 = pos("d1");
        let piece3 = Piece::Knight;
        let from3 = pos("b4");

        // Black pieces
        let piece4 = Piece::Bishop;
        let from4 = pos("e5");
        let piece5 = Piece::Pawn;
        let from5 = pos("g7");

        board.squares[from1].0 = Some((piece1, Color::White));
        board.squares[from2].0 = Some((piece2, Color::White));
        board.squares[from3].0 = Some((piece3, Color::White));

        board.squares[from4].0 = Some((piece4, Color::Black));
        board.squares[from5].0 = Some((piece5, Color::Black));

        let score = Evaluator::material_count(&board);

        assert_eq!(score, 1290);
    }

    #[test]
    fn test_black_winning_position() {
        let mut board = Board::empty();

        // White pieces (700)
        let piece1 = Piece::Pawn;
        let from1 = pos("e4");
        let piece2 = Piece::Pawn;
        let from2 = pos("f1");
        let piece3 = Piece::Rook;
        let from3 = pos("b4");

        // Black pieces (1730)
        let piece4 = Piece::Bishop;
        let from4 = pos("e5");
        let piece5 = Piece::Rook;
        let from5 = pos("g7");
        let piece6 = Piece::Queen;
        let from6 = pos("g8");

        board.squares[from1].0 = Some((piece1, Color::White));
        board.squares[from2].0 = Some((piece2, Color::White));
        board.squares[from3].0 = Some((piece3, Color::White));

        board.squares[from4].0 = Some((piece4, Color::Black));
        board.squares[from5].0 = Some((piece5, Color::Black));
        board.squares[from6].0 = Some((piece6, Color::Black));

        let score = Evaluator::material_count(&board);

        assert_eq!(score, -1030);
    }

    fn pos(s: &str) -> usize {
        let bytes = s.as_bytes();
        let file = (bytes[0] - b'a') as usize;
        let rank = (bytes[1] - b'1') as usize;
        rank * 8 + file
    }
}
