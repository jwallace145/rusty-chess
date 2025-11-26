use crate::{
    board::{Board2, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

pub struct ThreatEvaluator;

impl ThreatEvaluator {
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
}

impl BoardEvaluator for ThreatEvaluator {
    fn evaluate(&self, board: &Board2) -> i32 {
        let mut score = 0;

        for &color in &[Color::White, Color::Black] {
            let enemy_color = color.opponent();

            // iterate all piece types including pawns
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
                    let sq_bb = 1u64 << sq;

                    // compute attackers
                    let mut num_attackers = 0;
                    let enemy_pieces = board.pieces[enemy_color as usize];
                    for &ep in &[
                        Piece::Pawn,
                        Piece::Knight,
                        Piece::Bishop,
                        Piece::Rook,
                        Piece::Queen,
                    ] {
                        let mut ep_bb = enemy_pieces[ep as usize];
                        while ep_bb != 0 {
                            let ep_sq = ep_bb.trailing_zeros() as u8;
                            let attacks = board.attacks_from(ep, ep_sq, enemy_color);
                            if attacks & sq_bb != 0 {
                                num_attackers += 1;
                            }
                            ep_bb &= ep_bb - 1;
                        }
                    }

                    // compute defenders
                    let mut num_defenders = 0;
                    let own_pieces = board.pieces[color as usize];
                    for &op in &[
                        Piece::Pawn,
                        Piece::Knight,
                        Piece::Bishop,
                        Piece::Rook,
                        Piece::Queen,
                    ] {
                        let mut op_bb = own_pieces[op as usize];
                        while op_bb != 0 {
                            let op_sq = op_bb.trailing_zeros() as u8;
                            let attacks = board.attacks_from(op, op_sq, color);
                            if attacks & sq_bb != 0 {
                                num_defenders += 1;
                            }
                            op_bb &= op_bb - 1;
                        }
                    }

                    // Hanging piece penalty if attacked and insufficiently defended
                    if num_attackers > 0 && num_attackers > num_defenders {
                        let penalty = (value * 6 * num_attackers) / (10 * (num_defenders + 1));
                        if color == Color::White {
                            score -= penalty;
                        } else {
                            score += penalty;
                        }
                    }

                    bb &= bb - 1; // clear the least significant bit
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
