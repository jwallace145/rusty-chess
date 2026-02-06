use crate::{
    board::{Board, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

pub struct ThreatEvaluator;

impl ThreatEvaluator {
    /// Material values for pieces
    #[inline]
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
    /// Uses fixed-size array to avoid heap allocation
    #[inline]
    fn see_square(board: &Board, sq: u8, side: Color) -> i32 {
        let mut attackers = [
            board.attackers_to(sq, Color::White),
            board.attackers_to(sq, Color::Black),
        ];

        // Use fixed-size array instead of Vec (max 32 captures possible)
        let mut gain: [i32; 32] = [0; 32];
        let mut gain_len: usize = 0;

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
            gain[gain_len] = value_on_sq;
            gain_len += 1;

            value_on_sq = lva_value;
            attackers[s] &= !(1u64 << lva_sq);
            side_to_move = side_to_move.opponent();
        }

        // Negamax backward pass (need at least 2 elements)
        if gain_len >= 2 {
            for i in (0..gain_len - 1).rev() {
                gain[i] = std::cmp::max(-gain[i + 1], gain[i]);
            }
        }

        if gain_len == 0 { 0 } else { gain[0] }
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

                while bb != 0 {
                    let sq = bb.trailing_zeros() as u8;
                    let attacks = board.attacks_from(piece, sq, color);

                    // Compute hanging penalty: if enemy can profitably capture this piece
                    let enemy_attackers = board.attackers_to(sq, enemy_color);
                    let hanging_penalty = if enemy_attackers != 0 {
                        let see_score = Self::see_square(board, sq, enemy_color);
                        if see_score > 0 { see_score.min(50) } else { 0 }
                    } else {
                        0
                    };

                    // Offensive scoring: only count threats where capturing wins material
                    // and the target has limited escape squares
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

                            if attacks & (1u64 << ep_sq) != 0 {
                                // Would capturing this piece win material?
                                let capture_see = Self::see_square(board, ep_sq, color);

                                if capture_see > 0 {
                                    // Can the target escape? Count safe squares (empty squares it can move to)
                                    let escape_squares = board.attacks_from(ep, ep_sq, enemy_color)
                                        & !board.occupied();

                                    // If target has 3+ escape squares, it's not really threatened
                                    if escape_squares.count_ones() < 3 {
                                        // Scale bonus by SEE gain, cap at 25cp per threat
                                        let bonus = (capture_see / 30).min(25);
                                        if color == Color::White {
                                            score += bonus;
                                        } else {
                                            score -= bonus;
                                        }
                                    }
                                }
                            }
                            ep_bb &= ep_bb - 1;
                        }
                    }

                    // Apply hanging piece penalty
                    if hanging_penalty > 0 {
                        if color == Color::White {
                            score -= hanging_penalty;
                        } else {
                            score += hanging_penalty;
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
