use crate::{
    board::{Board, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

pub struct KingSafetyEvaluator;

impl BoardEvaluator for KingSafetyEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        let white_king_safety = Self::king_safety(board, Color::White);
        let black_king_safety = Self::king_safety(board, Color::Black);

        white_king_safety - black_king_safety
    }
}

impl KingSafetyEvaluator {
    fn king_safety(board: &Board, color: Color) -> i32 {
        let king_pos: usize = board.king_pos(color);

        let mut score: i32 = 0;

        // 1. Castling bonus
        if board.has_castled(color) {
            score += 40;
        }

        // 2. Pawn shield
        score += Self::pawn_shield(board, color, king_pos) * 12;

        // 3. Open files next to king
        score -= Self::open_file_penalty(board, king_pos) * 15;

        // 4. Enemy proximity
        score -= Self::enemy_piece_pressure(board, color, king_pos) * 6;

        score
    }

    /// Count pawns in the shield squares (3 squares in front of king)
    fn pawn_shield(board: &Board, color: Color, king_sq: usize) -> i32 {
        let file = (king_sq % 8) as i32;
        let rank = (king_sq / 8) as i32;

        let forward = match color {
            Color::White => 1,
            Color::Black => -1,
        };

        let mut count = 0;

        for df in -1..=1 {
            let f = file + df;
            let r = rank + forward;
            if (0..8).contains(&f) && (0..8).contains(&r) {
                let sq = (r * 8 + f) as usize;
                if let Some((Piece::Pawn, c)) = board.squares[sq].0
                    && c == color
                {
                    count += 1;
                }
            }
        }

        count
    }

    /// Penalty for open or semi-open files next to king
    fn open_file_penalty(board: &Board, king_sq: usize) -> i32 {
        let file = king_sq % 8;

        let mut penalty = 0;

        for adj_file in [file.saturating_sub(1), file, (file + 1).min(7)] {
            let mut has_any_pawn = false;

            for rank in 0..8 {
                let sq = rank * 8 + adj_file;
                if let Some((Piece::Pawn, _)) = board.squares[sq].0 {
                    has_any_pawn = true;
                    break;
                }
            }

            if !has_any_pawn {
                penalty += 1;
            }
        }

        penalty
    }

    /// Count enemy pieces within 1 king move radius
    fn enemy_piece_pressure(board: &Board, color: Color, king_sq: usize) -> i32 {
        let enemy = color.opponent();

        let king_file = (king_sq % 8) as i32;
        let king_rank = (king_sq / 8) as i32;

        let mut threats = 0;

        for df in -1..=1 {
            for dr in -1..=1 {
                if df == 0 && dr == 0 {
                    continue;
                }
                let f = king_file + df;
                let r = king_rank + dr;

                if !(0..=7).contains(&f) || !(0..=7).contains(&r) {
                    continue;
                }

                let sq = (r * 8 + f) as usize;

                if let Some((_, c)) = board.squares[sq].0
                    && c == enemy
                {
                    threats += 1;
                }
            }
        }

        threats
    }
}
