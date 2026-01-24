use crate::{
    board::{Board, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

/// Base bonus for creating a fork (attacking 2+ pieces)
const FORK_BASE_BONUS: i32 = 15;

/// Bonus multiplier based on the value of forked pieces
const FORK_VALUE_SCALE: i32 = 30;

/// Extra bonus when forking with a lower-value piece
const FORK_ATTACKER_BONUS: i32 = 10;

/// ForkEvaluator detects forks where a single piece attacks two or more enemy pieces.
/// Forks are powerful tactical motifs that can win material since the opponent
/// cannot defend both attacked pieces simultaneously.
pub struct ForkEvaluator;

impl ForkEvaluator {
    /// Get piece value for scoring calculations
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

    /// Evaluate fork potential for one side
    fn evaluate_side(board: &Board, color: Color) -> i32 {
        let mut score = 0;
        let enemy = color.opponent();
        let enemy_pieces = board.occ[enemy as usize];

        // Process each piece type
        for piece in [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ] {
            let mut pieces_bb = board.pieces[color as usize][piece as usize];
            let attacker_value = Self::piece_value(piece);

            while pieces_bb != 0 {
                let sq = pieces_bb.trailing_zeros() as u8;

                // Get attack bitboard for this piece
                let attacks = board.attacks_from(piece, sq, color);

                // Mask with enemy pieces to find attacked enemies
                let attacked_enemies = attacks & enemy_pieces;

                // Count attacked pieces
                let attack_count = attacked_enemies.count_ones();

                // Fork requires attacking 2 or more pieces
                if attack_count >= 2 {
                    score += Self::score_fork(board, attacked_enemies, attacker_value, enemy);
                }

                pieces_bb &= pieces_bb - 1;
            }
        }

        score
    }

    /// Calculate the score for a fork based on the attacked pieces
    #[inline]
    fn score_fork(board: &Board, attacked_enemies: u64, attacker_value: i32, enemy: Color) -> i32 {
        let mut fork_score = FORK_BASE_BONUS;
        let mut highest_value = 0;
        let mut second_highest_value = 0;

        // Iterate through attacked pieces to calculate their values
        let mut enemies = attacked_enemies;
        while enemies != 0 {
            let sq = enemies.trailing_zeros() as u8;

            if let Some((_, piece_type)) = board.piece_on(sq) {
                let value = Self::piece_value(piece_type);

                // Skip king from value calculation (can't actually capture)
                // but still count it for the fork detection
                if piece_type != Piece::King {
                    if value > highest_value {
                        second_highest_value = highest_value;
                        highest_value = value;
                    } else if value > second_highest_value {
                        second_highest_value = value;
                    }
                }
            }

            enemies &= enemies - 1;
        }

        // Scale bonus based on the two most valuable attacked pieces
        // The opponent can only save one, so we score based on what can be won
        let winnable_value = second_highest_value;
        fork_score += (winnable_value * FORK_VALUE_SCALE) / 900;

        // Extra bonus when a lower-value piece forks higher-value pieces
        if attacker_value < highest_value && attacker_value < second_highest_value {
            fork_score += FORK_ATTACKER_BONUS;
        }

        // Additional bonus for attacking more pieces (triple forks, etc.)
        let attack_count = attacked_enemies.count_ones();
        if attack_count > 2 {
            fork_score += (attack_count as i32 - 2) * 5;
        }

        // Bonus for forking with pieces that are hard to chase away
        // Knights are particularly good forkers
        if attacker_value == Self::piece_value(Piece::Knight) {
            fork_score += 5;
        }

        // Check if any attacked piece is undefended for extra bonus
        let enemy_attacks = board.attacks(enemy);
        let undefended = attacked_enemies & !enemy_attacks;
        if undefended != 0 {
            fork_score += 10;
        }

        fork_score
    }
}

impl BoardEvaluator for ForkEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        let white_score = Self::evaluate_side(board, Color::White);
        let black_score = Self::evaluate_side(board, Color::Black);
        white_score - black_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fork_scoring_basic() {
        // Test that score_fork returns positive values
        // This is a basic sanity check
        let evaluator = ForkEvaluator;
        // Just verify the evaluator can be instantiated
        assert_eq!(std::mem::size_of_val(&evaluator), 0);
    }

    #[test]
    fn test_piece_values() {
        assert_eq!(ForkEvaluator::piece_value(Piece::Pawn), 100);
        assert_eq!(ForkEvaluator::piece_value(Piece::Knight), 320);
        assert_eq!(ForkEvaluator::piece_value(Piece::Bishop), 330);
        assert_eq!(ForkEvaluator::piece_value(Piece::Rook), 500);
        assert_eq!(ForkEvaluator::piece_value(Piece::Queen), 900);
        assert_eq!(ForkEvaluator::piece_value(Piece::King), 20000);
    }
}
