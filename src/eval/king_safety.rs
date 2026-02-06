use crate::{
    board::{Board, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

pub struct KingSafetyEvaluator;

impl BoardEvaluator for KingSafetyEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        let white_king_safety: i32 = Self::king_safety(board, Color::White);
        let black_king_safety: i32 = Self::king_safety(board, Color::Black);

        white_king_safety - black_king_safety
    }
}

impl KingSafetyEvaluator {
    #[inline]
    fn king_safety(board: &Board, color: Color) -> i32 {
        let king_pos: u8 = board.king_square(color);

        let mut score: i32 = 0;

        // 1. Castling bonus
        if board.has_castled(color) {
            score += 10;
        }

        // 2. Pawn shield
        score += Self::pawn_shield(board, color, king_pos) * 4;

        // 3. Open files next to king
        score -= Self::open_file_penalty(board, king_pos) * 5;

        // 4. Enemy proximity
        score -= Self::enemy_piece_pressure(board, color, king_pos) * 2;

        // 5. Enemy attack pressure
        score -= Self::attackers_to_king_zone(board, color, king_pos);

        score
    }

    /// Count pawns in the shield squares (3 squares in front of king)
    #[inline]
    fn pawn_shield(board: &Board, color: Color, king_sq: u8) -> i32 {
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
                let sq = (r * 8 + f) as u8;
                if let Some((c, p)) = board.piece_on(sq)
                    && p == Piece::Pawn
                    && c == color
                {
                    count += 1;
                }
            }
        }

        count
    }

    /// Penalty for open or semi-open files next to king
    #[inline]
    fn open_file_penalty(board: &Board, king_sq: u8) -> i32 {
        let file = king_sq % 8;

        let mut penalty = 0;

        for adj_file in [file.saturating_sub(1), file, (file + 1).min(7)] {
            let mut has_any_pawn = false;

            for rank in 0..8 {
                let sq = rank * 8 + adj_file;
                if let Some((_, p)) = board.piece_on(sq)
                    && p == Piece::Pawn
                {
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
    #[inline]
    fn enemy_piece_pressure(board: &Board, color: Color, king_sq: u8) -> i32 {
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

                let sq = (r * 8 + f) as u8;

                if let Some((c, _)) = board.piece_on(sq)
                    && c == enemy
                {
                    threats += 1;
                }
            }
        }

        threats
    }

    #[inline]
    fn attackers_to_king_zone(board: &Board, color: Color, king_sq: u8) -> i32 {
        let enemy: Color = color.opponent();
        let king_file = (king_sq % 8) as i32;
        let king_rank = (king_sq / 8) as i32;

        // Collect all unique attackers to any king zone square
        let mut all_attackers: u64 = 0;

        for df in -1..=1 {
            for dr in -1..=1 {
                let f = king_file + df;
                let r = king_rank + dr;

                if !(0..8).contains(&f) || !(0..8).contains(&r) {
                    continue;
                }

                let sq = (r * 8 + f) as u8;
                all_attackers |= board.attackers_to(sq, enemy);
            }
        }

        // Score each unique attacker once
        let mut score = 0;
        while all_attackers != 0 {
            let attacker_sq = all_attackers.trailing_zeros() as u8;
            if let Some((_, p)) = board.piece_on(attacker_sq) {
                score += match p {
                    Piece::Pawn => 10,
                    Piece::Knight => 30,
                    Piece::Bishop => 30,
                    Piece::Rook => 50,
                    Piece::Queen => 90,
                    _ => 0,
                };
            }
            all_attackers &= all_attackers - 1;
        }

        score
    }
}
