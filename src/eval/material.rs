use crate::board::{Board, Color, Piece};

pub struct MaterialEvaluator;

impl MaterialEvaluator {
    pub fn evaluate(board: &Board) -> i32 {
        let mut white_material: i32 = 0;
        let mut black_material: i32 = 0;

        for square in &board.squares {
            if let Some((piece, color)) = square.0 {
                let value: i32 = Self::piece_value(piece);

                match color {
                    Color::White => white_material += value,
                    Color::Black => black_material += value,
                }
            }
        }

        white_material - black_material
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
