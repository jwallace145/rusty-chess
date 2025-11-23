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
        let white_attacks = board.attacks(Color::White);
        let black_attacks = board.attacks(Color::Black);

        let mut score = 0;

        for color in [Color::White, Color::Black] {
            let enemy_attacks = if color == Color::White {
                black_attacks
            } else {
                white_attacks
            };
            let own_attacks = if color == Color::White {
                white_attacks
            } else {
                black_attacks
            };

            for piece in [
                Piece::Pawn,
                Piece::Knight,
                Piece::Bishop,
                Piece::Rook,
                Piece::Queen,
            ] {
                let mut bb = board.pieces[color as usize][piece as usize];
                let value = Self::piece_value(piece);

                // Iterate through all pieces of this type using bit manipulation
                while bb != 0 {
                    let sq = bb.trailing_zeros();
                    let sq_bb = 1u64 << sq;

                    let attacked = enemy_attacks & sq_bb != 0;
                    let defended = own_attacks & sq_bb != 0;

                    // Hanging piece penalty
                    if attacked && !defended {
                        let penalty = (value as f32 * 0.6) as i32;
                        if color == Color::White {
                            score -= penalty
                        } else {
                            score += penalty
                        };
                    }

                    // Pawn threats bonus - check if enemy pawn is attacking this square
                    let enemy_color = if color == Color::White {
                        Color::Black
                    } else {
                        Color::White
                    };
                    let enemy_pawns = board.pieces[enemy_color as usize][Piece::Pawn as usize];

                    // Check if any enemy pawn attacks this square
                    let pawn_threat = attacked && {
                        let mut enemy_pawns_copy = enemy_pawns;
                        let mut is_threatened_by_pawn = false;
                        while enemy_pawns_copy != 0 {
                            let pawn_sq = enemy_pawns_copy.trailing_zeros() as u8;
                            let pawn_attacks =
                                board.attacks_from(Piece::Pawn, pawn_sq, enemy_color);
                            if pawn_attacks & sq_bb != 0 {
                                is_threatened_by_pawn = true;
                                break;
                            }
                            enemy_pawns_copy &= enemy_pawns_copy - 1;
                        }
                        is_threatened_by_pawn
                    };

                    if pawn_threat {
                        let pawn_bonus = match piece {
                            Piece::Pawn => 0,
                            Piece::Knight | Piece::Bishop => 40,
                            Piece::Rook => 30,
                            Piece::Queen => 15,
                            _ => 0,
                        };
                        if color == Color::White {
                            score -= pawn_bonus
                        } else {
                            score += pawn_bonus
                        };
                    }

                    // Note: Counting individual attackers/defenders is complex and expensive.
                    // The simple attack/defend check above is more practical.
                    // Removing the flawed count_ones logic.

                    bb &= bb - 1; // Clear the least significant bit
                }
            }
        }

        // Small tempo bonus
        if board.side_to_move == Color::White {
            score += 10
        } else {
            score -= 10
        };

        score
    }
}
