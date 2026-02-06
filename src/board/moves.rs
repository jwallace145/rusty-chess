use super::Board;
use crate::board::{CastlingRights, ChessMove, ChessMoveState, ChessMoveType, Color, Piece};
use crate::movegen::MoveGenerator;
use crate::search::{CastlingRight, ZobristTable};

impl Board {
    // Move generation

    pub fn generate_moves(&self, moves: &mut Vec<ChessMove>) {
        MoveGenerator::generate_legal_moves(self, moves);
    }

    // State operations

    pub fn make_move(&mut self, mv: ChessMove) -> ChessMoveState {
        let from: u8 = mv.from as u8;
        let to: u8 = mv.to as u8;
        let from_mask: u64 = 1u64 << from;
        let to_mask: u64 = 1u64 << to;

        // Get the piece being moved and optional captured piece
        let moved_piece: Option<(Color, Piece)> = self.piece_on(from);
        let captured_piece: Option<(Color, Piece)> = self.piece_on(to);

        // Save current state for undo
        // We repurpose the king_moved/rook_moved fields to encode castling rights
        use crate::board::castling::Side;
        let state = ChessMoveState {
            chess_move: mv,
            moved_piece: moved_piece.map(|(c, p)| (p, c)),
            captured_piece: captured_piece.map(|(c, p)| (p, c)),
            previous_side_to_move: self.side_to_move,
            // Encode castling rights in these boolean fields
            white_king_moved: !self.castling.has(Color::White, Side::KingSide),
            white_kingside_rook_moved: !self.castling.has(Color::White, Side::QueenSide),
            white_queenside_rook_moved: self.halfmove_clock > 127, // Use high bit for halfmove overflow
            black_king_moved: !self.castling.has(Color::Black, Side::KingSide),
            black_kingside_rook_moved: !self.castling.has(Color::Black, Side::QueenSide),
            black_queenside_rook_moved: false, // Reserved for future use
            previous_en_passant: if self.en_passant < 64 {
                Some(self.en_passant as usize)
            } else {
                None
            },
            previous_zobrist_hash: self.hash,
            previous_halfmove_clock: self.halfmove_clock,
        };

        // Save castling rights and halfmove clock before the move
        let old_castling = self.castling;
        let old_en_passant = self.en_passant;

        // === INCREMENTAL ZOBRIST HASH UPDATE: XOR OUT OLD STATE ===
        let zobrist = ZobristTable::get();

        // 1. XOR out piece at from square
        if let Some((moving_color, moving_piece)) = moved_piece {
            self.hash ^= zobrist.piece(moving_piece, moving_color, from as usize);
        }

        // 2. XOR out captured piece (if any)
        if mv.capture {
            if mv.move_type == ChessMoveType::EnPassant {
                // En passant - the captured pawn is not on the to square
                let captured_pawn_sq = match self.side_to_move {
                    Color::White => to - 8,
                    Color::Black => to + 8,
                };
                self.hash ^= zobrist.piece(
                    Piece::Pawn,
                    self.side_to_move.opponent(),
                    captured_pawn_sq as usize,
                );
            } else if let Some((cap_color, cap_piece)) = captured_piece {
                self.hash ^= zobrist.piece(cap_piece, cap_color, to as usize);
            }
        }

        // 3. XOR out rook if castling (will be moved to new square)
        if mv.move_type == ChessMoveType::Castle {
            match to {
                6 => self.hash ^= zobrist.piece(Piece::Rook, Color::White, 7), // h1
                2 => self.hash ^= zobrist.piece(Piece::Rook, Color::White, 0), // a1
                62 => self.hash ^= zobrist.piece(Piece::Rook, Color::Black, 63), // h8
                58 => self.hash ^= zobrist.piece(Piece::Rook, Color::Black, 56), // a8
                _ => {}
            }
        }

        // 4. XOR out old castling rights
        if old_castling.has(Color::White, Side::KingSide) {
            self.hash ^= zobrist.castling(CastlingRight::WhiteKingside);
        }
        if old_castling.has(Color::White, Side::QueenSide) {
            self.hash ^= zobrist.castling(CastlingRight::WhiteQueenside);
        }
        if old_castling.has(Color::Black, Side::KingSide) {
            self.hash ^= zobrist.castling(CastlingRight::BlackKingside);
        }
        if old_castling.has(Color::Black, Side::QueenSide) {
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
            if mv.capture {
                if mv.move_type == ChessMoveType::EnPassant {
                    // En passant - remove the captured pawn
                    let captured_pawn_sq = match moving_color {
                        Color::White => to - 8,
                        Color::Black => to + 8,
                    };
                    let captured_mask = 1u64 << captured_pawn_sq;
                    self.pieces[moving_color.opponent() as usize][Piece::Pawn as usize] &=
                        !captured_mask;
                    self.occ[moving_color.opponent() as usize] &= !captured_mask;
                } else if let Some((cap_color, cap_piece)) = captured_piece {
                    // Normal capture - remove piece from destination
                    self.pieces[cap_color as usize][cap_piece as usize] &= !to_mask;
                    self.occ[cap_color as usize] &= !to_mask;
                }
            }

            // Handle promotions
            let final_piece = if let ChessMoveType::Promotion(promo_piece) = mv.move_type {
                promo_piece
            } else {
                moving_piece
            };

            // Place piece on destination square
            self.pieces[moving_color as usize][final_piece as usize] |= to_mask;
            self.occ[moving_color as usize] |= to_mask;

            // Update king position
            if moving_piece == Piece::King {
                self.king_sq[moving_color as usize] = to;
            }

            // Handle castling rook move
            if mv.move_type == ChessMoveType::Castle {
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
                use crate::board::castling::Side;
                self.castling.remove(moving_color, Side::KingSide);
                self.castling.remove(moving_color, Side::QueenSide);
            } else if moving_piece == Piece::Rook {
                use crate::board::castling::Side;
                // Check which rook moved
                match from {
                    0 => self.castling.remove(Color::White, Side::QueenSide), // a1
                    7 => self.castling.remove(Color::White, Side::KingSide),  // h1
                    56 => self.castling.remove(Color::Black, Side::QueenSide), // a8
                    63 => self.castling.remove(Color::Black, Side::KingSide), // h8
                    _ => {}
                }
            }

            // If a rook is captured, remove castling rights
            if let Some((_cap_color, Piece::Rook)) = captured_piece {
                use crate::board::castling::Side;
                match to {
                    0 => self.castling.remove(Color::White, Side::QueenSide), // a1
                    7 => self.castling.remove(Color::White, Side::KingSide),  // h1
                    56 => self.castling.remove(Color::Black, Side::QueenSide), // a8
                    63 => self.castling.remove(Color::Black, Side::KingSide), // h8
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
                        Color::White => from + 8,
                        Color::Black => from - 8,
                    };
                }
            }

            // Update halfmove clock (reset on pawn move or capture)
            if moving_piece == Piece::Pawn || mv.capture {
                self.halfmove_clock = 0;
            } else {
                self.halfmove_clock += 1;
            }

            // Switch side to move
            self.side_to_move = self.side_to_move.opponent();

            // === INCREMENTAL ZOBRIST HASH UPDATE: XOR IN NEW STATE ===

            // 7. XOR in piece at to square (might be promoted)
            let final_piece = if let ChessMoveType::Promotion(promo_piece) = mv.move_type {
                promo_piece
            } else {
                moving_piece
            };
            self.hash ^= zobrist.piece(final_piece, moving_color, to as usize);

            // 8. XOR in rook at new position if castling
            if mv.move_type == ChessMoveType::Castle {
                match to {
                    6 => self.hash ^= zobrist.piece(Piece::Rook, Color::White, 5), // f1
                    2 => self.hash ^= zobrist.piece(Piece::Rook, Color::White, 3), // d1
                    62 => self.hash ^= zobrist.piece(Piece::Rook, Color::Black, 61), // f8
                    58 => self.hash ^= zobrist.piece(Piece::Rook, Color::Black, 59), // d8
                    _ => {}
                }
            }

            // 9. XOR in new castling rights
            if self.castling.has(Color::White, Side::KingSide) {
                self.hash ^= zobrist.castling(CastlingRight::WhiteKingside);
            }
            if self.castling.has(Color::White, Side::QueenSide) {
                self.hash ^= zobrist.castling(CastlingRight::WhiteQueenside);
            }
            if self.castling.has(Color::Black, Side::KingSide) {
                self.hash ^= zobrist.castling(CastlingRight::BlackKingside);
            }
            if self.castling.has(Color::Black, Side::QueenSide) {
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

        state
    }

    pub fn unmake_move(&mut self, state: ChessMoveState) {
        use crate::board::chess_move::ChessMoveType;

        let mv = state.chess_move;
        let from = mv.from as u8;
        let to = mv.to as u8;
        let from_mask = 1u64 << from;
        let to_mask = 1u64 << to;

        // Restore side to move first
        self.side_to_move = state.previous_side_to_move;

        if let Some((moved_piece, moving_color)) = state.moved_piece {
            // Determine the piece on the destination square (might be promoted)
            let piece_on_dest = if let ChessMoveType::Promotion(_) = mv.move_type {
                if let Some((_, p)) = self.piece_on(to) {
                    p
                } else {
                    moved_piece
                }
            } else {
                moved_piece
            };

            // Remove piece from destination square
            self.pieces[moving_color as usize][piece_on_dest as usize] &= !to_mask;
            self.occ[moving_color as usize] &= !to_mask;

            // Restore piece to source square
            self.pieces[moving_color as usize][moved_piece as usize] |= from_mask;
            self.occ[moving_color as usize] |= from_mask;

            // Restore king position if king was moved
            if moved_piece == Piece::King {
                self.king_sq[moving_color as usize] = from;
            }

            // Handle castling undo
            if mv.move_type == ChessMoveType::Castle {
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
            if mv.capture {
                if mv.move_type == ChessMoveType::EnPassant {
                    // Restore en passant captured pawn
                    let captured_pawn_sq = match moving_color {
                        Color::White => to - 8,
                        Color::Black => to + 8,
                    };
                    let captured_mask = 1u64 << captured_pawn_sq;
                    self.pieces[moving_color.opponent() as usize][Piece::Pawn as usize] |=
                        captured_mask;
                    self.occ[moving_color.opponent() as usize] |= captured_mask;
                } else if let Some((captured_piece, captured_color)) = state.captured_piece {
                    // Restore normal captured piece
                    self.pieces[captured_color as usize][captured_piece as usize] |= to_mask;
                    self.occ[captured_color as usize] |= to_mask;
                }
            }

            // Update combined occupancy
            self.occ_all = self.occ[Color::White as usize] | self.occ[Color::Black as usize];
        }

        // Restore en passant square
        self.en_passant = state.previous_en_passant.map(|sq| sq as u8).unwrap_or(64);

        // Restore castling rights from encoded boolean fields
        use crate::board::castling::Side;
        self.castling = CastlingRights::empty();
        if !state.white_king_moved {
            self.castling.add(Color::White, Side::KingSide);
        }
        if !state.white_kingside_rook_moved {
            self.castling.add(Color::White, Side::QueenSide);
        }
        if !state.black_king_moved {
            self.castling.add(Color::Black, Side::KingSide);
        }
        if !state.black_kingside_rook_moved {
            self.castling.add(Color::Black, Side::QueenSide);
        }

        // Restore halfmove clock
        self.halfmove_clock = state.previous_halfmove_clock;

        // Restore zobrist hash
        self.hash = state.previous_zobrist_hash;
    }
}
