use crate::{
    board::{Board2, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

pub struct ThreatEvaluator;

impl ThreatEvaluator {
    /// Material values for pieces
    fn piece_value(piece: Piece) -> i32 {
        match piece {
            Piece::Pawn => 100,
            Piece::Knight => 320,
            Piece::Bishop => 330,
            Piece::Rook => 500,
            Piece::Queen => 900,
            Piece::King => 20000,
        }
    }

    /// Static Exchange Evaluation for a square
    fn see_square(board: &Board2, sq: u8, side: Color) -> i32 {
        let mut attackers = [
            board.attackers_to(sq, Color::White),
            board.attackers_to(sq, Color::Black),
        ];

        let mut gain = Vec::with_capacity(32);

        let mut value_on_sq = if let Some((_, p)) = board.piece_on(sq) {
            Self::piece_value(p)
        } else {
            0
        };

        let mut side_to_move = side;

        loop {
            let s = side_to_move as usize;
            let mut lva_sq = None;
            let mut lva_value = i32::MAX;

            for piece_type in [
                Piece::Pawn,
                Piece::Knight,
                Piece::Bishop,
                Piece::Rook,
                Piece::Queen,
            ] {
                let bb = attackers[s] & board.pieces[s][piece_type as usize];
                if bb != 0 {
                    let sq_bit = bb.trailing_zeros() as u8;
                    let val = Self::piece_value(piece_type);
                    if val < lva_value {
                        lva_value = val;
                        lva_sq = Some(sq_bit);
                    }
                }
            }

            if lva_sq.is_none() {
                break;
            }

            let lva_sq = lva_sq.unwrap();
            gain.push(value_on_sq);

            value_on_sq = lva_value;
            attackers[s] &= !(1u64 << lva_sq);
            side_to_move = side_to_move.opponent();
        }

        // Negamax backward pass (need at least 2 elements)
        if gain.len() >= 2 {
            for i in (0..gain.len() - 1).rev() {
                gain[i] = std::cmp::max(-gain[i + 1], gain[i]);
            }
        }

        if gain.is_empty() { 0 } else { gain[0] }
    }
}

impl BoardEvaluator for ThreatEvaluator {
    fn evaluate(&self, board: &Board2) -> i32 {
        let mut score = 0;

        for &color in &[Color::White, Color::Black] {
            let enemy_color = color.opponent();

            for &piece in &[
                Piece::Pawn,
                Piece::Knight,
                Piece::Bishop,
                Piece::Rook,
                Piece::Queen,
            ] {
                let mut bb = board.pieces[color as usize][piece as usize];
                let value = Self::piece_value(piece);

                while bb != 0 {
                    let sq = bb.trailing_zeros() as u8;

                    // Offensive scoring: attacks on enemy pieces
                    let mut num_attacks = 0;
                    for &ep in &[
                        Piece::Pawn,
                        Piece::Knight,
                        Piece::Bishop,
                        Piece::Rook,
                        Piece::Queen,
                    ] {
                        let mut ep_bb = board.pieces[enemy_color as usize][ep as usize];
                        while ep_bb != 0 {
                            let ep_sq = ep_bb.trailing_zeros() as u8;
                            let attacks = board.attacks_from(piece, sq, color);
                            if attacks & (1u64 << ep_sq) != 0 {
                                num_attacks += 1;
                            }
                            ep_bb &= ep_bb - 1;
                        }
                    }

                    // Add small bonus for attacking enemy pieces
                    let attack_bonus = num_attacks * (value / 10);
                    if color == Color::White {
                        score += attack_bonus;
                    } else {
                        score -= attack_bonus;
                    }

                    // Defensive SEE scoring: penalize hanging / overextended pieces
                    let see_score = Self::see_square(board, sq, color);
                    if color == Color::White {
                        score += see_score; // SEE positive if beneficial
                    } else {
                        score -= see_score; // SEE negative if losing material
                    }

                    bb &= bb - 1;
                }
            }
        }

        // Small tempo bonus
        if board.side_to_move == Color::White {
            score += 10;
        } else {
            score -= 10;
        }

        score
    }
}
