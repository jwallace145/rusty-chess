use crate::{
    board::{Board2, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

pub struct KingSafetyEvaluator;

impl BoardEvaluator for KingSafetyEvaluator {
    fn evaluate(&self, board: &Board2) -> i32 {
        let white_king_safety: i32 = Self::king_safety(board, Color::White);
        let black_king_safety: i32 = Self::king_safety(board, Color::Black);

        white_king_safety - black_king_safety
    }
}

impl KingSafetyEvaluator {
    fn king_safety(board: &Board2, color: Color) -> i32 {
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
    fn pawn_shield(board: &Board2, color: Color, king_sq: u8) -> i32 {
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
    fn open_file_penalty(board: &Board2, king_sq: u8) -> i32 {
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
    fn enemy_piece_pressure(board: &Board2, color: Color, king_sq: u8) -> i32 {
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

    fn attackers_to_king_zone(board: &Board2, color: Color, king_sq: u8) -> i32 {
        let enemy: Color = color.opponent();
        let king_zone: Vec<u8> = Self::king_zone(king_sq);

        let mut score = 0;

        for &sq in &king_zone {
            let mut attackers = board.attackers_to(sq, enemy);

            // Iterate through attacker squares in the bitboard
            while attackers != 0 {
                let attacker_sq = attackers.trailing_zeros() as u8;
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
                attackers &= attackers - 1; // Clear the least significant bit
            }
        }

        score
    }

    fn king_zone(king_sq: u8) -> Vec<u8> {
        let mut zone = Vec::with_capacity(9);
        let f = (king_sq % 8) as i32;
        let r = (king_sq / 8) as i32;

        for df in -1..=1 {
            for dr in -1..=1 {
                let nf = f + df;
                let nr = r + dr;
                if (0..8).contains(&nf) && (0..8).contains(&nr) {
                    zone.push((nr * 8 + nf) as u8);
                }
            }
        }
        zone
    }
}
