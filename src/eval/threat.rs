use crate::{
    board::{Board, Color, Piece},
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
    fn see_square(board: &Board, sq: u8, side: Color) -> i32 {
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

    /// Check if a piece is en prise (attacked by enemy and SEE is negative)
    fn is_piece_en_prise(board: &Board, sq: u8, color: Color) -> bool {
        let enemy = color.opponent();
        let attackers = board.attackers_to(sq, enemy);
        if attackers == 0 {
            return false;
        }
        // Use SEE to determine if piece is truly en prise
        Self::see_square(board, sq, enemy) > 0
    }
}

/// Maximum threat score in centipawns (hard clamp)
const MAX_THREAT_SCORE: i32 = 150;

impl BoardEvaluator for ThreatEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
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
                let attacker_value = Self::piece_value(piece);

                while bb != 0 {
                    let sq = bb.trailing_zeros() as u8;
                    let attacks = board.attacks_from(piece, sq, color);

                    // Check if the attacking piece is en prise (attacked by enemy)
                    let attacker_is_safe = !Self::is_piece_en_prise(board, sq, color);

                    // Offensive scoring: attacks on enemy pieces (with validation)
                    for &ep in &[
                        Piece::Pawn,
                        Piece::Knight,
                        Piece::Bishop,
                        Piece::Rook,
                        Piece::Queen,
                    ] {
                        let target_value = Self::piece_value(ep);
                        let mut ep_bb = board.pieces[enemy_color as usize][ep as usize];

                        while ep_bb != 0 {
                            let ep_sq = ep_bb.trailing_zeros() as u8;

                            if attacks & (1u64 << ep_sq) != 0 {
                                // Validate the threat:
                                // 1. Only count if attacker is safe OR threat is profitable
                                // 2. Only count if we're attacking something of equal/higher value
                                //    OR our attacker is safe
                                let threat_is_valid =
                                    attacker_is_safe || target_value >= attacker_value;

                                if threat_is_valid {
                                    // Bonus scales with target value, smaller for low-value attackers
                                    let bonus = (target_value / 20).min(30);
                                    if color == Color::White {
                                        score += bonus;
                                    } else {
                                        score -= bonus;
                                    }
                                }
                            }
                            ep_bb &= ep_bb - 1;
                        }
                    }

                    // Defensive SEE scoring: penalize hanging / overextended pieces
                    // Only apply if piece is genuinely hanging (SEE negative)
                    let see_score = Self::see_square(board, sq, color);
                    if see_score < 0 {
                        // Piece is hanging - apply penalty (capped)
                        let penalty = see_score.max(-50);
                        if color == Color::White {
                            score += penalty;
                        } else {
                            score -= penalty;
                        }
                    }

                    bb &= bb - 1;
                }
            }
        }

        // Apply hard clamp to ±MAX_THREAT_SCORE
        score.clamp(-MAX_THREAT_SCORE, MAX_THREAT_SCORE)
    }
}
