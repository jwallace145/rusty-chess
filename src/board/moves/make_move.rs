use crate::board::castling::CastlingSide;
use crate::board::{Board, Color, Piece};
use crate::movegen::MoveGenerator;
use crate::search::{CastlingRight, ZobristTable};

use super::{ChessMove, MoveUndo};

impl Board {
    pub fn generate_moves(&self, moves: &mut Vec<ChessMove>) {
        MoveGenerator::generate_legal_moves(self, moves);
    }

    pub fn make_move(&mut self, mv: ChessMove) -> MoveUndo {
        let from = mv.from();
        let to = mv.to();
        let from_mask: u64 = 1u64 << from;
        let to_mask: u64 = 1u64 << to;

        // Get the piece being moved and optional captured piece at destination
        let moved_piece: Option<(Color, Piece)> = self.piece_on(from as u8);
        let captured_piece_at_to: Option<(Color, Piece)> = self.piece_on(to as u8);

        // Determine captured piece (EP captures a pawn not at the to square)
        let captured_piece: Option<Piece> = if mv.is_en_passant() {
            Some(Piece::Pawn)
        } else {
            captured_piece_at_to.map(|(_, p)| p)
        };
        let is_capture = captured_piece.is_some();

        // Save current state for undo
        let undo = MoveUndo {
            chess_move: mv,
            captured_piece,
            previous_castling: self.castling,
            previous_en_passant: self.en_passant,
            previous_halfmove_clock: self.halfmove_clock,
            previous_zobrist_hash: self.hash,
        };

        // Save castling rights and halfmove clock before the move
        let old_castling = self.castling;
        let old_en_passant = self.en_passant;

        // === INCREMENTAL ZOBRIST HASH UPDATE: XOR OUT OLD STATE ===
        let zobrist = ZobristTable::get();

        // 1. XOR out piece at from square
        if let Some((moving_color, moving_piece)) = moved_piece {
            self.hash ^= zobrist.piece(moving_piece, moving_color, from);
        }

        // 2. XOR out captured piece (if any)
        if is_capture {
            if mv.is_en_passant() {
                // En passant - the captured pawn is not on the to square
                let captured_pawn_sq = match self.side_to_move {
                    Color::White => to - 8,
                    Color::Black => to + 8,
                };
                self.hash ^=
                    zobrist.piece(Piece::Pawn, self.side_to_move.opponent(), captured_pawn_sq);
            } else if let Some((cap_color, cap_piece)) = captured_piece_at_to {
                self.hash ^= zobrist.piece(cap_piece, cap_color, to);
            }
        }

        // 3. XOR out rook if castling (will be moved to new square)
        if mv.is_castle() {
            match to {
                6 => self.hash ^= zobrist.piece(Piece::Rook, Color::White, 7), // h1
                2 => self.hash ^= zobrist.piece(Piece::Rook, Color::White, 0), // a1
                62 => self.hash ^= zobrist.piece(Piece::Rook, Color::Black, 63), // h8
                58 => self.hash ^= zobrist.piece(Piece::Rook, Color::Black, 56), // a8
                _ => {}
            }
        }

        // 4. XOR out old castling rights
        if old_castling.has(Color::White, CastlingSide::KingSide) {
            self.hash ^= zobrist.castling(CastlingRight::WhiteKingside);
        }
        if old_castling.has(Color::White, CastlingSide::QueenSide) {
            self.hash ^= zobrist.castling(CastlingRight::WhiteQueenside);
        }
        if old_castling.has(Color::Black, CastlingSide::KingSide) {
            self.hash ^= zobrist.castling(CastlingRight::BlackKingside);
        }
        if old_castling.has(Color::Black, CastlingSide::QueenSide) {
            self.hash ^= zobrist.castling(CastlingRight::BlackQueenside);
        }

        // 5. XOR out old en passant
        if old_en_passant < 64 {
            let file = (old_en_passant % 8) as usize;
            self.hash ^= zobrist.en_passant(file);
        }

        // 6. XOR out old side to move (if Black)
        if self.side_to_move == Color::Black {
            self.hash ^= zobrist.side_to_move();
        }

        // Apply the move
        if let Some((moving_color, moving_piece)) = moved_piece {
            // Remove piece from source square
            self.pieces[moving_color as usize][moving_piece as usize] &= !from_mask;
            self.occ[moving_color as usize] &= !from_mask;

            // Handle captures
            if is_capture {
                if mv.is_en_passant() {
                    // En passant - remove the captured pawn
                    let captured_pawn_sq = match moving_color {
                        Color::White => to - 8,
                        Color::Black => to + 8,
                    };
                    let captured_mask = 1u64 << captured_pawn_sq;
                    self.pieces[moving_color.opponent() as usize][Piece::Pawn as usize] &=
                        !captured_mask;
                    self.occ[moving_color.opponent() as usize] &= !captured_mask;
                } else if let Some((cap_color, cap_piece)) = captured_piece_at_to {
                    // Normal capture - remove piece from destination
                    self.pieces[cap_color as usize][cap_piece as usize] &= !to_mask;
                    self.occ[cap_color as usize] &= !to_mask;
                }
            }

            // Handle promotions
            let final_piece = if let Some(promo_piece) = mv.promotion_piece() {
                promo_piece
            } else {
                moving_piece
            };

            // Place piece on destination square
            self.pieces[moving_color as usize][final_piece as usize] |= to_mask;
            self.occ[moving_color as usize] |= to_mask;

            // Update king position
            if moving_piece == Piece::King {
                self.king_sq[moving_color as usize] = to as u8;
            }

            // Handle castling rook move
            if mv.is_castle() {
                match to {
                    6 => {
                        // White kingside: move rook from h1 to f1
                        self.pieces[Color::White as usize][Piece::Rook as usize] &= !(1u64 << 7);
                        self.pieces[Color::White as usize][Piece::Rook as usize] |= 1u64 << 5;
                        self.occ[Color::White as usize] &= !(1u64 << 7);
                        self.occ[Color::White as usize] |= 1u64 << 5;
                    }
                    2 => {
                        // White queenside: move rook from a1 to d1
                        self.pieces[Color::White as usize][Piece::Rook as usize] &= !(1u64 << 0);
                        self.pieces[Color::White as usize][Piece::Rook as usize] |= 1u64 << 3;
                        self.occ[Color::White as usize] &= !(1u64 << 0);
                        self.occ[Color::White as usize] |= 1u64 << 3;
                    }
                    62 => {
                        // Black kingside: move rook from h8 to f8
                        self.pieces[Color::Black as usize][Piece::Rook as usize] &= !(1u64 << 63);
                        self.pieces[Color::Black as usize][Piece::Rook as usize] |= 1u64 << 61;
                        self.occ[Color::Black as usize] &= !(1u64 << 63);
                        self.occ[Color::Black as usize] |= 1u64 << 61;
                    }
                    58 => {
                        // Black queenside: move rook from a8 to d8
                        self.pieces[Color::Black as usize][Piece::Rook as usize] &= !(1u64 << 56);
                        self.pieces[Color::Black as usize][Piece::Rook as usize] |= 1u64 << 59;
                        self.occ[Color::Black as usize] &= !(1u64 << 56);
                        self.occ[Color::Black as usize] |= 1u64 << 59;
                    }
                    _ => {}
                }
            }

            // Update castling rights
            if moving_piece == Piece::King {
                self.castling.remove(moving_color, CastlingSide::KingSide);
                self.castling.remove(moving_color, CastlingSide::QueenSide);
            } else if moving_piece == Piece::Rook {
                // Check which rook moved
                match from {
                    0 => self.castling.remove(Color::White, CastlingSide::QueenSide), // a1
                    7 => self.castling.remove(Color::White, CastlingSide::KingSide),  // h1
                    56 => self.castling.remove(Color::Black, CastlingSide::QueenSide), // a8
                    63 => self.castling.remove(Color::Black, CastlingSide::KingSide), // h8
                    _ => {}
                }
            }

            // If a rook is captured, remove castling rights
            if let Some((_, Piece::Rook)) = captured_piece_at_to {
                match to {
                    0 => self.castling.remove(Color::White, CastlingSide::QueenSide), // a1
                    7 => self.castling.remove(Color::White, CastlingSide::KingSide),  // h1
                    56 => self.castling.remove(Color::Black, CastlingSide::QueenSide), // a8
                    63 => self.castling.remove(Color::Black, CastlingSide::KingSide), // h8
                    _ => {}
                }
            }

            // Update combined occupancy
            self.occ_all = self.occ[Color::White as usize] | self.occ[Color::Black as usize];

            // Update en passant square
            self.en_passant = 64; // Clear by default
            if moving_piece == Piece::Pawn {
                let from_rank = from / 8;
                let to_rank = to / 8;
                // Check for double pawn push
                if (from_rank as i8 - to_rank as i8).abs() == 2 {
                    // Set en passant square to the square the pawn skipped over
                    self.en_passant = match moving_color {
                        Color::White => (from + 8) as u8,
                        Color::Black => (from - 8) as u8,
                    };
                }
            }

            // Update halfmove clock (reset on pawn move or capture)
            if moving_piece == Piece::Pawn || is_capture {
                self.halfmove_clock = 0;
            } else {
                self.halfmove_clock += 1;
            }

            // Switch side to move
            self.side_to_move = self.side_to_move.opponent();

            // === INCREMENTAL ZOBRIST HASH UPDATE: XOR IN NEW STATE ===

            // 7. XOR in piece at to square (might be promoted)
            let final_piece = if let Some(promo_piece) = mv.promotion_piece() {
                promo_piece
            } else {
                moving_piece
            };
            self.hash ^= zobrist.piece(final_piece, moving_color, to);

            // 8. XOR in rook at new position if castling
            if mv.is_castle() {
                match to {
                    6 => self.hash ^= zobrist.piece(Piece::Rook, Color::White, 5), // f1
                    2 => self.hash ^= zobrist.piece(Piece::Rook, Color::White, 3), // d1
                    62 => self.hash ^= zobrist.piece(Piece::Rook, Color::Black, 61), // f8
                    58 => self.hash ^= zobrist.piece(Piece::Rook, Color::Black, 59), // d8
                    _ => {}
                }
            }

            // 9. XOR in new castling rights
            if self.castling.has(Color::White, CastlingSide::KingSide) {
                self.hash ^= zobrist.castling(CastlingRight::WhiteKingside);
            }
            if self.castling.has(Color::White, CastlingSide::QueenSide) {
                self.hash ^= zobrist.castling(CastlingRight::WhiteQueenside);
            }
            if self.castling.has(Color::Black, CastlingSide::KingSide) {
                self.hash ^= zobrist.castling(CastlingRight::BlackKingside);
            }
            if self.castling.has(Color::Black, CastlingSide::QueenSide) {
                self.hash ^= zobrist.castling(CastlingRight::BlackQueenside);
            }

            // 10. XOR in new en passant
            if self.en_passant < 64 {
                let file = (self.en_passant % 8) as usize;
                self.hash ^= zobrist.en_passant(file);
            }

            // 11. XOR in new side to move (if Black)
            if self.side_to_move == Color::Black {
                self.hash ^= zobrist.side_to_move();
            }
        }

        undo
    }

    pub fn unmake_move(&mut self, undo: MoveUndo) {
        let mv = undo.chess_move;
        let from = mv.from();
        let to = mv.to();
        let from_mask = 1u64 << from;
        let to_mask = 1u64 << to;

        // Restore side to move (toggle back)
        self.side_to_move = self.side_to_move.opponent();
        let moving_color = self.side_to_move;

        // Find the piece at the destination square
        if let Some((_, piece_on_dest)) = self.piece_on(to as u8) {
            // For promotion, the original piece was a Pawn
            let original_piece = if mv.is_promotion() {
                Piece::Pawn
            } else {
                piece_on_dest
            };

            // Remove piece from destination square
            self.pieces[moving_color as usize][piece_on_dest as usize] &= !to_mask;
            self.occ[moving_color as usize] &= !to_mask;

            // Restore piece to source square
            self.pieces[moving_color as usize][original_piece as usize] |= from_mask;
            self.occ[moving_color as usize] |= from_mask;

            // Restore king position if king was moved
            if original_piece == Piece::King {
                self.king_sq[moving_color as usize] = from as u8;
            }

            // Handle castling undo
            if mv.is_castle() {
                match to {
                    6 => {
                        // White kingside: move rook back from f1 to h1
                        self.pieces[Color::White as usize][Piece::Rook as usize] &= !(1u64 << 5);
                        self.pieces[Color::White as usize][Piece::Rook as usize] |= 1u64 << 7;
                        self.occ[Color::White as usize] &= !(1u64 << 5);
                        self.occ[Color::White as usize] |= 1u64 << 7;
                    }
                    2 => {
                        // White queenside: move rook back from d1 to a1
                        self.pieces[Color::White as usize][Piece::Rook as usize] &= !(1u64 << 3);
                        self.pieces[Color::White as usize][Piece::Rook as usize] |= 1u64 << 0;
                        self.occ[Color::White as usize] &= !(1u64 << 3);
                        self.occ[Color::White as usize] |= 1u64 << 0;
                    }
                    62 => {
                        // Black kingside: move rook back from f8 to h8
                        self.pieces[Color::Black as usize][Piece::Rook as usize] &= !(1u64 << 61);
                        self.pieces[Color::Black as usize][Piece::Rook as usize] |= 1u64 << 63;
                        self.occ[Color::Black as usize] &= !(1u64 << 61);
                        self.occ[Color::Black as usize] |= 1u64 << 63;
                    }
                    58 => {
                        // Black queenside: move rook back from d8 to a8
                        self.pieces[Color::Black as usize][Piece::Rook as usize] &= !(1u64 << 59);
                        self.pieces[Color::Black as usize][Piece::Rook as usize] |= 1u64 << 56;
                        self.occ[Color::Black as usize] &= !(1u64 << 59);
                        self.occ[Color::Black as usize] |= 1u64 << 56;
                    }
                    _ => {}
                }
            }

            // Restore captured piece
            if let Some(captured) = undo.captured_piece {
                if mv.is_en_passant() {
                    // Restore en passant captured pawn
                    let captured_pawn_sq = match moving_color {
                        Color::White => to - 8,
                        Color::Black => to + 8,
                    };
                    let captured_mask = 1u64 << captured_pawn_sq;
                    self.pieces[moving_color.opponent() as usize][Piece::Pawn as usize] |=
                        captured_mask;
                    self.occ[moving_color.opponent() as usize] |= captured_mask;
                } else {
                    // Restore normal captured piece
                    self.pieces[moving_color.opponent() as usize][captured as usize] |= to_mask;
                    self.occ[moving_color.opponent() as usize] |= to_mask;
                }
            }

            // Update combined occupancy
            self.occ_all = self.occ[Color::White as usize] | self.occ[Color::Black as usize];
        }

        // Restore state directly from undo
        self.castling = undo.previous_castling;
        self.en_passant = undo.previous_en_passant;
        self.halfmove_clock = undo.previous_halfmove_clock;
        self.hash = undo.previous_zobrist_hash;
    }
}
