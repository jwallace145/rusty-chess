use super::chess_move::{ChessMove, ChessMoveState, ChessMoveType};
use super::{Color, Piece};
use crate::search::{CastlingRight, ZobristTable, compute_hash};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Square(pub Option<(Piece, Color)>);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Board {
    pub squares: [Square; 64],
    pub side_to_move: Color,
    pub white_king_pos: usize,
    pub black_king_pos: usize,
    white_king_moved: bool,
    white_kingside_rook_moved: bool,
    white_queenside_rook_moved: bool,
    black_king_moved: bool,
    black_kingside_rook_moved: bool,
    black_queenside_rook_moved: bool,
    pub en_passant_target: Option<usize>,
    pub zobrist_hash: u64,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    #[cfg(test)]
    pub fn empty() -> Self {
        let mut board = Self {
            squares: [Square(None); 64],
            side_to_move: Color::White,
            white_king_pos: 0,
            black_king_pos: 0,
            white_king_moved: false,
            white_kingside_rook_moved: false,
            white_queenside_rook_moved: false,
            black_king_moved: false,
            black_kingside_rook_moved: false,
            black_queenside_rook_moved: false,
            en_passant_target: None,
            zobrist_hash: 0,
        };

        // Compute and set initial Zobrist hash of board state
        board.zobrist_hash = compute_hash(&board);

        board
    }

    pub fn new() -> Self {
        use Color::*;
        use Piece::*;

        // Initialize board with 64 squares
        let mut squares = [Square(None); 64];

        // Populate White side
        let white_back_rank = [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook];
        for i in 0..8 {
            squares[i] = Square(Some((white_back_rank[i], White)));
            squares[i + 8] = Square(Some((Pawn, White)));
        }

        // Populate Black side
        let black_back_rank = [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook];
        for i in 0..8 {
            squares[i + 56] = Square(Some((black_back_rank[i], Black)));
            squares[i + 48] = Square(Some((Pawn, Black)));
        }

        let mut board = Self {
            squares,
            side_to_move: White,
            white_king_pos: 4,  // e1
            black_king_pos: 60, // e8
            white_king_moved: false,
            white_kingside_rook_moved: false,
            white_queenside_rook_moved: false,
            black_king_moved: false,
            black_kingside_rook_moved: false,
            black_queenside_rook_moved: false,
            en_passant_target: None,
            zobrist_hash: 0,
        };

        // Compute and set initial Zobrist hash of board state
        board.zobrist_hash = compute_hash(&board);

        board
    }

    pub fn switch_side(&mut self) {
        self.side_to_move = match self.side_to_move {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
    }

    /// Generates all legal moves for the current side to move.
    ///
    /// This method filters pseudo-legal moves by checking if they leave the king in check.
    /// It uses buffer reuse to minimize allocations during attack detection.
    ///
    /// # Performance
    /// This is a critical hot path during search. The implementation:
    /// - Creates a single buffer for pseudo-legal move generation
    /// - Reuses an attack-check buffer for all validation calls
    /// - Reduces allocations from ~560 per call to ~2 per call
    ///
    /// # Returns
    /// A vector containing all legal moves for the current position.
    pub fn generate_legal_moves(&self) -> Vec<ChessMove> {
        let mut pseudo_moves = self.generate_moves();
        let mut attack_buffer = Vec::with_capacity(64);

        pseudo_moves.retain(|&m| {
            let mut board_copy = *self;
            board_copy.apply_move(m);
            let king_square = board_copy.king_pos(self.side_to_move);
            !board_copy.is_square_attacked_buffered(
                king_square,
                self.side_to_move.opponent(),
                &mut attack_buffer,
            )
        });

        pseudo_moves
    }

    /// Parses a UCI move string (e.g., "e2e4", "e7e8q") and returns the corresponding ChessMove
    /// if it's legal in the current position.
    pub fn parse_uci(&self, uci: &str) -> Result<ChessMove, String> {
        // Validate UCI string length (4 for normal moves, 5 for promotions)
        if uci.len() != 4 && uci.len() != 5 {
            return Err(format!("Invalid UCI format: {}", uci));
        }

        // Parse from square
        let from = Self::parse_square(&uci[0..2])?;
        // Parse to square
        let to = Self::parse_square(&uci[2..4])?;

        // Parse optional promotion piece
        let promotion = if uci.len() == 5 {
            match uci.chars().nth(4) {
                Some('q') => Some(Piece::Queen),
                Some('r') => Some(Piece::Rook),
                Some('b') => Some(Piece::Bishop),
                Some('n') => Some(Piece::Knight),
                _ => return Err(format!("Invalid promotion piece: {}", uci)),
            }
        } else {
            None
        };

        // Generate legal moves and find matching move
        let legal_moves = self.generate_legal_moves();

        for &mv in &legal_moves {
            if mv.from == from && mv.to == to {
                // For promotions, ensure the move type matches
                if let Some(promo_piece) = promotion {
                    if let ChessMoveType::Promotion(piece) = mv.move_type
                        && piece == promo_piece
                    {
                        return Ok(mv);
                    }
                } else {
                    // For non-promotions, return the matching move
                    match mv.move_type {
                        ChessMoveType::Promotion(_) => continue,
                        _ => return Ok(mv),
                    }
                }
            }
        }

        Err(format!("Move {} is not legal in current position", uci))
    }

    /// Parses a square in algebraic notation (e.g., "e2") to a board index (0-63)
    fn parse_square(algebraic: &str) -> Result<usize, String> {
        if algebraic.len() != 2 {
            return Err(format!("Invalid square: {}", algebraic));
        }

        let mut chars = algebraic.chars();
        let file = chars.next().unwrap();
        let rank = chars.next().unwrap();

        if !('a'..='h').contains(&file) || !('1'..='8').contains(&rank) {
            return Err(format!("Invalid square: {}", algebraic));
        }

        let file_idx = (file as u8 - b'a') as usize;
        let rank_idx = (rank as u8 - b'1') as usize;

        Ok(rank_idx * 8 + file_idx)
    }

    /// Generates all pseudo-legal moves for the current side to move.
    ///
    /// This method creates a single buffer and reuses it for all piece move generation,
    /// eliminating multiple vector allocations. Pseudo-legal moves may leave the king
    /// in check and must be filtered by `generate_legal_moves()`.
    ///
    /// # Returns
    /// A vector containing all pseudo-legal moves for the current position.
    pub fn generate_moves(&self) -> Vec<ChessMove> {
        let mut moves = Vec::with_capacity(128);

        for (i, square) in self.squares.iter().enumerate() {
            if let Some((piece, color)) = square.0
                && color == self.side_to_move
            {
                self.generate_piece_moves(i, piece, color, &mut moves);
            }
        }

        moves
    }

    pub fn apply_move(&mut self, chess_move: ChessMove) -> ChessMoveState {
        let zobrist = ZobristTable::get();

        // Get moved and optional captured piece from move
        let moved_piece = self.squares[chess_move.from].0;
        let captured_piece = self.squares[chess_move.to].0;

        // Save current chess move state for return
        let state = ChessMoveState {
            chess_move,
            moved_piece,
            captured_piece,
            previous_side_to_move: self.side_to_move,
            white_king_moved: self.white_king_moved,
            white_kingside_rook_moved: self.white_kingside_rook_moved,
            white_queenside_rook_moved: self.white_queenside_rook_moved,
            black_king_moved: self.black_king_moved,
            black_kingside_rook_moved: self.black_kingside_rook_moved,
            black_queenside_rook_moved: self.black_queenside_rook_moved,
            previous_en_passant: self.en_passant_target,
            previous_zobrist_hash: self.zobrist_hash,
        };

        // Update hash and remove old en passant
        if let Some(ep_square) = self.en_passant_target {
            self.zobrist_hash ^= zobrist.en_passant(ep_square % 8);
        }

        // Update hash and remove moved piece from source square
        if let Some((piece, color)) = moved_piece {
            self.zobrist_hash ^= zobrist.piece(piece, color, chess_move.from);
        }

        // Update hash and remove optional captured piece
        if let Some((piece, color)) = captured_piece {
            self.zobrist_hash ^= zobrist.piece(piece, color, chess_move.to);
        }

        // Move the piece
        self.squares[chess_move.to].0 = moved_piece;
        self.squares[chess_move.from].0 = None;

        // Handle castling - move rook and update hash accordingly
        if chess_move.move_type == ChessMoveType::Castle
            && let Some((Piece::King, color)) = moved_piece
        {
            match color {
                Color::White => {
                    if chess_move.to == 6 {
                        // Kingside: h1 to f1
                        let rook = self.squares[7].0.unwrap();
                        self.zobrist_hash ^= zobrist.piece(rook.0, rook.1, 7);
                        self.zobrist_hash ^= zobrist.piece(rook.0, rook.1, 5);
                        self.squares[5].0 = self.squares[7].0;
                        self.squares[7].0 = None;
                    } else if chess_move.to == 2 {
                        // Queenside: a1 to d1
                        let rook = self.squares[0].0.unwrap();
                        self.zobrist_hash ^= zobrist.piece(rook.0, rook.1, 0);
                        self.zobrist_hash ^= zobrist.piece(rook.0, rook.1, 3);
                        self.squares[3].0 = self.squares[0].0;
                        self.squares[0].0 = None;
                    }
                }
                Color::Black => {
                    if chess_move.to == 62 {
                        // Kingside: h8 to f8
                        let rook = self.squares[63].0.unwrap();
                        self.zobrist_hash ^= zobrist.piece(rook.0, rook.1, 63);
                        self.zobrist_hash ^= zobrist.piece(rook.0, rook.1, 61);
                        self.squares[61].0 = self.squares[63].0;
                        self.squares[63].0 = None;
                    } else if chess_move.to == 58 {
                        // Queenside: a8 to d8
                        let rook = self.squares[56].0.unwrap();
                        self.zobrist_hash ^= zobrist.piece(rook.0, rook.1, 56);
                        self.zobrist_hash ^= zobrist.piece(rook.0, rook.1, 59);
                        self.squares[59].0 = self.squares[56].0;
                        self.squares[56].0 = None;
                    }
                }
            }
        }

        // Update the castling rights after handling
        let old_wk = self.white_king_moved;
        let old_wkr = self.white_kingside_rook_moved;
        let old_wqr = self.white_queenside_rook_moved;
        let old_bk = self.black_king_moved;
        let old_bkr = self.black_kingside_rook_moved;
        let old_bqr = self.black_queenside_rook_moved;

        // Update king position and castling flags
        if let Some((piece, color)) = moved_piece {
            match piece {
                Piece::King => match color {
                    Color::White => {
                        self.white_king_pos = chess_move.to;
                        self.white_king_moved = true;
                    }
                    Color::Black => {
                        self.black_king_pos = chess_move.to;
                        self.black_king_moved = true;
                    }
                },
                Piece::Rook => match color {
                    Color::White => {
                        if chess_move.from == 7 {
                            self.white_kingside_rook_moved = true;
                        } else if chess_move.from == 0 {
                            self.white_queenside_rook_moved = true;
                        }
                    }
                    Color::Black => {
                        if chess_move.from == 63 {
                            self.black_kingside_rook_moved = true;
                        } else if chess_move.from == 56 {
                            self.black_queenside_rook_moved = true;
                        }
                    }
                },
                _ => {}
            }
        }

        // XOR castling rights that changed
        // Check if the ability to castle changed (not just individual flags)
        let old_can_castle_wk = !old_wk && !old_wkr;
        let new_can_castle_wk = !self.white_king_moved && !self.white_kingside_rook_moved;
        if old_can_castle_wk != new_can_castle_wk {
            self.zobrist_hash ^= zobrist.castling(CastlingRight::WhiteKingside);
        }

        let old_can_castle_wq = !old_wk && !old_wqr;
        let new_can_castle_wq = !self.white_king_moved && !self.white_queenside_rook_moved;
        if old_can_castle_wq != new_can_castle_wq {
            self.zobrist_hash ^= zobrist.castling(CastlingRight::WhiteQueenside);
        }

        let old_can_castle_bk = !old_bk && !old_bkr;
        let new_can_castle_bk = !self.black_king_moved && !self.black_kingside_rook_moved;
        if old_can_castle_bk != new_can_castle_bk {
            self.zobrist_hash ^= zobrist.castling(CastlingRight::BlackKingside);
        }

        let old_can_castle_bq = !old_bk && !old_bqr;
        let new_can_castle_bq = !self.black_king_moved && !self.black_queenside_rook_moved;
        if old_can_castle_bq != new_can_castle_bq {
            self.zobrist_hash ^= zobrist.castling(CastlingRight::BlackQueenside);
        }

        // Handle pawn promotion
        if let Some((Piece::Pawn, color)) = self.squares[chess_move.to].0 {
            let rank = chess_move.to / 8;
            if (color == Color::White && rank == 7) || (color == Color::Black && rank == 0) {
                // Remove pawn from hash
                self.zobrist_hash ^= zobrist.piece(Piece::Pawn, color, chess_move.to);
                // Add queen to hash
                self.zobrist_hash ^= zobrist.piece(Piece::Queen, color, chess_move.to);
                // Update board
                self.squares[chess_move.to].0 = Some((Piece::Queen, color));
            }
        } else {
            // Update hash and add piece to destination square
            if let Some((piece, color)) = moved_piece {
                self.zobrist_hash ^= zobrist.piece(piece, color, chess_move.to);
            }
        }

        // Reset en passant target
        self.en_passant_target = None;

        // Handle en passant
        if let Some((Piece::Pawn, color)) = moved_piece {
            let rank_from = chess_move.from / 8;
            let rank_to = chess_move.to / 8;

            // Set en passant target for double pawn move
            if (color == Color::White && rank_from == 1 && rank_to == 3)
                || (color == Color::Black && rank_from == 6 && rank_to == 4)
            {
                let ep_square = if chess_move.to > chess_move.from {
                    chess_move.from + (chess_move.to - chess_move.from) / 2
                } else {
                    chess_move.to + (chess_move.from - chess_move.to) / 2
                };
                self.en_passant_target = Some(ep_square);
                // Add new en passant to hash
                self.zobrist_hash ^= zobrist.en_passant(ep_square % 8);
            }

            // Handle en passant capture
            if chess_move.move_type == ChessMoveType::EnPassant {
                let captured_pawn_square = if color == Color::White {
                    chess_move.to - 8
                } else {
                    chess_move.to + 8
                };

                // Remove captured pawn from hash
                if let Some((piece, pawn_color)) = self.squares[captured_pawn_square].0 {
                    self.zobrist_hash ^= zobrist.piece(piece, pawn_color, captured_pawn_square);
                }

                // Remove captured pawn from board
                self.squares[captured_pawn_square].0 = None;
            }
        }

        // Update the hash after toggling sides to move
        self.zobrist_hash ^= zobrist.side_to_move();

        // Switch sides for next move
        self.switch_side();

        state
    }

    pub fn undo_move(&mut self, state: ChessMoveState) {
        let chess_move = state.chess_move;

        // Restore pieces
        self.squares[chess_move.from].0 = state.moved_piece;
        self.squares[chess_move.to].0 = state.captured_piece;

        // Restore side to move
        self.side_to_move = state.previous_side_to_move;

        // Restore king positions
        if let Some((Piece::King, color)) = state.moved_piece {
            match color {
                Color::White => self.white_king_pos = chess_move.from,
                Color::Black => self.black_king_pos = chess_move.from,
            }
        }

        // Restore all king/rook moved flags for castling
        self.white_king_moved = state.white_king_moved;
        self.white_kingside_rook_moved = state.white_kingside_rook_moved;
        self.white_queenside_rook_moved = state.white_queenside_rook_moved;
        self.black_king_moved = state.black_king_moved;
        self.black_kingside_rook_moved = state.black_kingside_rook_moved;
        self.black_queenside_rook_moved = state.black_queenside_rook_moved;

        // Restore en passant
        self.en_passant_target = state.previous_en_passant;

        // Restore previous Zobrist hash
        self.zobrist_hash = state.previous_zobrist_hash;
    }

    /// Generates moves for a piece at the given position into the provided buffer.
    ///
    /// This function dispatches to the appropriate piece-specific move generator.
    ///
    /// # Arguments
    /// * `index` - The position of the piece
    /// * `piece` - The type of piece
    /// * `color` - The color of the piece
    /// * `buffer` - Mutable buffer to push moves into
    fn generate_piece_moves(
        &self,
        index: usize,
        piece: Piece,
        color: Color,
        buffer: &mut Vec<ChessMove>,
    ) {
        match piece {
            Piece::Pawn => self.generate_pawn_moves(index, color, buffer),
            Piece::Rook => self.generate_rook_moves(index, color, buffer),
            Piece::Knight => self.generate_knight_moves(index, color, buffer),
            Piece::Bishop => self.generate_bishop_moves(index, color, buffer),
            Piece::Queen => self.generate_queen_moves(index, color, buffer),
            Piece::King => self.generate_king_moves(index, color, buffer),
        }
    }

    /// Generates pawn moves into the provided buffer.
    ///
    /// Handles forward moves, double moves, captures, en passant, and promotions.
    ///
    /// # Arguments
    /// * `index` - The position of the pawn
    /// * `color` - The color of the pawn
    /// * `buffer` - Mutable buffer to push moves into
    fn generate_pawn_moves(&self, index: usize, color: Color, buffer: &mut Vec<ChessMove>) {
        let rank = index / 8;
        let file = index % 8;

        let (forward, start_rank, promotion_rank): (isize, usize, usize) = match color {
            Color::White => (1, 1, 7),
            Color::Black => (-1, 6, 0),
        };

        // move forward one square
        let new_rank = (rank as isize + forward) as usize;
        if new_rank < 8 {
            let forward_idx = new_rank * 8 + file;
            if self.squares[forward_idx].0.is_none() {
                let move_type = if new_rank == promotion_rank {
                    ChessMoveType::Promotion(Piece::Queen)
                } else {
                    ChessMoveType::Normal
                };

                buffer.push(ChessMove {
                    from: index,
                    to: forward_idx,
                    capture: false,
                    move_type,
                });

                if rank == start_rank {
                    let double_rank = (rank as isize + 2 * forward) as usize;
                    let double_idx = double_rank * 8 + file;
                    if self.squares[double_idx].0.is_none() {
                        buffer.push(ChessMove {
                            from: index,
                            to: double_idx,
                            capture: false,
                            move_type: ChessMoveType::Normal,
                        });
                    }
                }
            }
        }

        // Diagonal captures
        for df in [-1, 1] {
            let new_file = file as isize + df;
            if (0..8).contains(&new_file) && new_rank < 8 {
                let capture_idx = new_rank * 8 + new_file as usize;
                if let Some((_, target_color)) = self.squares[capture_idx].0 {
                    if target_color != color {
                        let move_type = if new_rank == promotion_rank {
                            ChessMoveType::Promotion(Piece::Queen)
                        } else {
                            ChessMoveType::Normal
                        };

                        buffer.push(ChessMove {
                            from: index,
                            to: capture_idx,
                            capture: true,
                            move_type,
                        });
                    }
                } else if Some(capture_idx) == self.en_passant_target {
                    // en passant capture
                    buffer.push(ChessMove {
                        from: index,
                        to: capture_idx,
                        capture: true,
                        move_type: ChessMoveType::EnPassant,
                    });
                }
            }
        }
    }

    /// Generates rook moves into the provided buffer.
    ///
    /// # Arguments
    /// * `index` - The position of the rook
    /// * `color` - The color of the rook
    /// * `buffer` - Mutable buffer to push moves into
    fn generate_rook_moves(&self, index: usize, color: Color, buffer: &mut Vec<ChessMove>) {
        self.generate_sliding_moves(index, color, &[(1, 0), (-1, 0), (0, 1), (0, -1)], buffer);
    }

    /// Generates knight moves into the provided buffer.
    ///
    /// # Arguments
    /// * `index` - The position of the knight
    /// * `color` - The color of the knight
    /// * `buffer` - Mutable buffer to push moves into
    fn generate_knight_moves(&self, index: usize, color: Color, buffer: &mut Vec<ChessMove>) {
        let rank = (index / 8) as isize;
        let file = (index % 8) as isize;
        let deltas = [
            (2, 1),
            (1, 2),
            (-1, 2),
            (-2, 1),
            (-2, -1),
            (-1, -2),
            (1, -2),
            (2, -1),
        ];

        for (dr, df) in deltas {
            let new_rank = rank + dr;
            let new_file = file + df;
            if (0..8).contains(&new_rank) && (0..8).contains(&new_file) {
                let to = (new_rank * 8 + new_file) as usize;
                match self.squares[to].0 {
                    None => buffer.push(ChessMove {
                        from: index,
                        to,
                        capture: false,
                        move_type: ChessMoveType::Normal,
                    }),
                    Some((_, target_color)) if target_color != color => buffer.push(ChessMove {
                        from: index,
                        to,
                        capture: true,
                        move_type: ChessMoveType::Normal,
                    }),
                    _ => {}
                }
            }
        }
    }

    /// Generates bishop moves into the provided buffer.
    ///
    /// # Arguments
    /// * `index` - The position of the bishop
    /// * `color` - The color of the bishop
    /// * `buffer` - Mutable buffer to push moves into
    fn generate_bishop_moves(&self, index: usize, color: Color, buffer: &mut Vec<ChessMove>) {
        self.generate_sliding_moves(index, color, &[(1, 1), (1, -1), (-1, 1), (-1, -1)], buffer);
    }

    /// Generates queen moves into the provided buffer.
    ///
    /// # Arguments
    /// * `index` - The position of the queen
    /// * `color` - The color of the queen
    /// * `buffer` - Mutable buffer to push moves into
    fn generate_queen_moves(&self, index: usize, color: Color, buffer: &mut Vec<ChessMove>) {
        self.generate_sliding_moves(
            index,
            color,
            &[
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
            ],
            buffer,
        );
    }

    /// Generates king moves into the provided buffer.
    ///
    /// Handles normal king moves and castling.
    ///
    /// # Arguments
    /// * `index` - The position of the king
    /// * `color` - The color of the king
    /// * `buffer` - Mutable buffer to push moves into
    fn generate_king_moves(&self, index: usize, color: Color, buffer: &mut Vec<ChessMove>) {
        let rank = (index / 8) as isize;
        let file = (index % 8) as isize;
        let deltas = [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];

        for (dr, df) in deltas {
            let new_rank = rank + dr;
            let new_file = file + df;
            if (0..8).contains(&new_rank) && (0..8).contains(&new_file) {
                let to = (new_rank * 8 + new_file) as usize;
                match self.squares[to].0 {
                    None => buffer.push(ChessMove {
                        from: index,
                        to,
                        capture: false,
                        move_type: ChessMoveType::Normal,
                    }),
                    Some((_, target_color)) if target_color != color => {
                        buffer.push(ChessMove {
                            from: index,
                            to,
                            capture: true,
                            move_type: ChessMoveType::Normal,
                        });
                    }
                    _ => {}
                }
            }
        }

        // Castling logic
        match color {
            Color::White => {
                // Kingside castling (O-O): e1 to g1
                if !self.white_king_moved
                    && !self.white_kingside_rook_moved
                    && index == 4 // e1
                    && self.squares[5].0.is_none() // f1 empty
                    && self.squares[6].0.is_none() // g1 empty
                    && self.squares[7].0 == Some((Piece::Rook, Color::White))
                // h1 has white rook
                {
                    buffer.push(ChessMove {
                        from: index,
                        to: 6, // g1
                        capture: false,
                        move_type: ChessMoveType::Castle,
                    });
                }

                // Queenside castling (O-O-O): e1 to c1
                if !self.white_king_moved
                    && !self.white_queenside_rook_moved
                    && index == 4 // e1
                    && self.squares[3].0.is_none() // d1 empty
                    && self.squares[2].0.is_none() // c1 empty
                    && self.squares[1].0.is_none() // b1 empty
                    && self.squares[0].0 == Some((Piece::Rook, Color::White))
                // a1 has white rook
                {
                    buffer.push(ChessMove {
                        from: index,
                        to: 2, // c1
                        capture: false,
                        move_type: ChessMoveType::Castle,
                    });
                }
            }
            Color::Black => {
                // Kingside castling (O-O): e8 to g8
                if !self.black_king_moved
                    && !self.black_kingside_rook_moved
                    && index == 60 // e8
                    && self.squares[61].0.is_none() // f8 empty
                    && self.squares[62].0.is_none() // g8 empty
                    && self.squares[63].0 == Some((Piece::Rook, Color::Black))
                // h8 has black rook
                {
                    buffer.push(ChessMove {
                        from: index,
                        to: 62, // g8
                        capture: false,
                        move_type: ChessMoveType::Castle,
                    });
                }

                // Queenside castling (O-O-O): e8 to c8
                if !self.black_king_moved
                    && !self.black_queenside_rook_moved
                    && index == 60 // e8
                    && self.squares[59].0.is_none() // d8 empty
                    && self.squares[58].0.is_none() // c8 empty
                    && self.squares[57].0.is_none() // b8 empty
                    && self.squares[56].0 == Some((Piece::Rook, Color::Black))
                // a8 has black rook
                {
                    buffer.push(ChessMove {
                        from: index,
                        to: 58, // c8
                        capture: false,
                        move_type: ChessMoveType::Castle,
                    });
                }
            }
        }
    }

    /// Generates sliding piece moves (rook, bishop, queen) into the provided buffer.
    ///
    /// This function pushes moves directly into the buffer without clearing it first.
    /// The caller is responsible for managing the buffer's state.
    ///
    /// # Arguments
    /// * `index` - The position of the sliding piece
    /// * `color` - The color of the sliding piece
    /// * `directions` - Direction vectors for sliding (e.g., [(1,0), (-1,0)] for rook)
    /// * `buffer` - Mutable buffer to push moves into
    fn generate_sliding_moves(
        &self,
        index: usize,
        color: Color,
        directions: &[(isize, isize)],
        buffer: &mut Vec<ChessMove>,
    ) {
        let rank = (index / 8) as isize;
        let file = (index % 8) as isize;

        for (dr, df) in directions {
            let mut r = rank + dr;
            let mut f = file + df;
            while (0..8).contains(&r) && (0..8).contains(&f) {
                let to = (r * 8 + f) as usize;
                match self.squares[to].0 {
                    None => buffer.push(ChessMove {
                        from: index,
                        to,
                        capture: false,
                        move_type: ChessMoveType::Normal,
                    }),
                    Some((_, target_color)) if target_color != color => {
                        buffer.push(ChessMove {
                            from: index,
                            to,
                            capture: true,
                            move_type: ChessMoveType::Normal,
                        });
                        break;
                    }
                    _ => break,
                }
                r += dr;
                f += df;
            }
        }
    }

    /// Checks if a square is attacked by pieces of the given color.
    ///
    /// This method creates an internal buffer for move generation. For better performance
    /// in loops, use `is_square_attacked_buffered()` with a reusable buffer.
    ///
    /// # Arguments
    /// * `square` - The square index to check (0-63)
    /// * `attacker_color` - The color of the attacking pieces
    ///
    /// # Returns
    /// `true` if the square is attacked, `false` otherwise
    pub fn is_square_attacked(&self, square: usize, attacker_color: Color) -> bool {
        let mut buffer = Vec::with_capacity(64);
        self.is_square_attacked_buffered(square, attacker_color, &mut buffer)
    }

    /// Checks if a square is attacked by pieces of the given color using a provided buffer.
    ///
    /// This is the optimized version that reuses a buffer for move generation, avoiding
    /// repeated allocations. The buffer is cleared before each piece's moves are generated.
    ///
    /// # Performance
    /// This method is critical for legal move generation performance. During move validation,
    /// this is called for every pseudo-legal move, making buffer reuse essential.
    ///
    /// # Arguments
    /// * `square` - The square index to check (0-63)
    /// * `attacker_color` - The color of the attacking pieces
    /// * `buffer` - Mutable buffer for temporary move storage
    ///
    /// # Returns
    /// `true` if the square is attacked, `false` otherwise
    pub fn is_square_attacked_buffered(
        &self,
        square: usize,
        attacker_color: Color,
        buffer: &mut Vec<ChessMove>,
    ) -> bool {
        for (i, sq) in self.squares.iter().enumerate() {
            if let Some((piece, color)) = sq.0
                && color == attacker_color
            {
                buffer.clear();
                self.generate_piece_moves(i, piece, color, buffer);
                if buffer.iter().any(|m| m.to == square) {
                    return true;
                }
            }
        }
        false
    }

    pub fn king_pos(&self, color: Color) -> usize {
        match color {
            Color::White => self.white_king_pos,
            Color::Black => self.black_king_pos,
        }
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        let king_square = self.king_pos(color);
        self.is_square_attacked(king_square, color.opponent())
    }

    pub fn is_checkmate(&self) -> bool {
        let legal_moves = self.generate_legal_moves();
        legal_moves.is_empty() && self.is_in_check(self.side_to_move)
    }

    pub fn is_stalemate(&self) -> bool {
        let legal_moves = self.generate_legal_moves();
        legal_moves.is_empty() && !self.is_in_check(self.side_to_move)
    }

    #[allow(dead_code)]
    pub fn can_castle_white_kingside(&self) -> bool {
        !self.white_king_moved && !self.white_kingside_rook_moved
    }

    #[allow(dead_code)]
    pub fn can_castle_white_queenside(&self) -> bool {
        !self.white_king_moved && !self.white_queenside_rook_moved
    }

    #[allow(dead_code)]
    pub fn can_castle_black_kingside(&self) -> bool {
        !self.black_king_moved && !self.black_kingside_rook_moved
    }

    #[allow(dead_code)]
    pub fn can_castle_black_queenside(&self) -> bool {
        !self.black_king_moved && !self.black_queenside_rook_moved
    }

    pub fn print(&self) {
        // Unicode chess symbols
        fn unicode_symbol(piece: Piece, color: Color) -> char {
            match (piece, color) {
                (Piece::Pawn, Color::White) => '♙',
                (Piece::Knight, Color::White) => '♘',
                (Piece::Bishop, Color::White) => '♗',
                (Piece::Rook, Color::White) => '♖',
                (Piece::Queen, Color::White) => '♕',
                (Piece::King, Color::White) => '♔',
                (Piece::Pawn, Color::Black) => '♟',
                (Piece::Knight, Color::Black) => '♞',
                (Piece::Bishop, Color::Black) => '♝',
                (Piece::Rook, Color::Black) => '♜',
                (Piece::Queen, Color::Black) => '♛',
                (Piece::King, Color::Black) => '♚',
            }
        }

        // ANSI color codes for board squares
        const LIGHT_SQUARE: &str = "\x1b[48;5;230m"; // beige
        const DARK_SQUARE: &str = "\x1b[48;5;94m"; // brown
        const RESET: &str = "\x1b[0m";

        println!("\n    a  b  c  d  e  f  g  h");
        println!("   -------------------------");

        for rank in (0..8).rev() {
            print!("{} ", rank + 1);
            for file in 0..8 {
                let idx = rank * 8 + file;
                let square_color = if (rank + file) % 2 == 0 {
                    LIGHT_SQUARE
                } else {
                    DARK_SQUARE
                };
                let symbol = match self.squares[idx].0 {
                    Some((piece, color)) => unicode_symbol(piece, color),
                    None => ' ',
                };
                print!("{} {} {}", square_color, symbol, RESET);
            }
            println!(" {}", rank + 1);
        }

        println!("   -------------------------");
        println!("    a  b  c  d  e  f  g  h");
        println!("\nSide to move: {:?}\n", self.side_to_move);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_board_initialization() {
        let board = Board::empty();

        assert_eq!(board.side_to_move, Color::White);

        for i in 0..64 {
            assert_eq!(board.squares[i].0, None);
        }
    }

    #[test]
    fn test_board_initialization() {
        let board = Board::new();

        assert_eq!(board.side_to_move, Color::White);

        let white_back_rank = [
            Piece::Rook,
            Piece::Knight,
            Piece::Bishop,
            Piece::Queen,
            Piece::King,
            Piece::Bishop,
            Piece::Knight,
            Piece::Rook,
        ];

        for (i, &piece) in white_back_rank.iter().enumerate() {
            assert_eq!(board.squares[i].0, Some((piece, Color::White)));
        }

        for i in 8..16 {
            assert_eq!(board.squares[i].0, Some((Piece::Pawn, Color::White)));
        }

        for i in 16..48 {
            assert_eq!(board.squares[i].0, None);
        }

        for i in 48..56 {
            assert_eq!(board.squares[i].0, Some((Piece::Pawn, Color::Black)));
        }

        let black_back_rank = [
            Piece::Rook,
            Piece::Knight,
            Piece::Bishop,
            Piece::Queen,
            Piece::King,
            Piece::Bishop,
            Piece::Knight,
            Piece::Rook,
        ];

        for i in 56..64 {
            assert_eq!(
                board.squares[i].0,
                Some((black_back_rank[i - 56], Color::Black))
            );
        }
    }

    #[test]
    fn test_switch_side() {
        let mut board = Board::new();
        assert_eq!(board.side_to_move, Color::White);

        board.switch_side();
        assert_eq!(board.side_to_move, Color::Black);

        board.switch_side();
        assert_eq!(board.side_to_move, Color::White);
    }

    #[test]
    fn test_board_copy() {
        let board = Board::new();
        let copied_board = board;

        assert_eq!(board, copied_board);

        // modify a copy should not affect the original
        let mut modified_board = copied_board;
        modified_board.switch_side();
        assert_ne!(board.side_to_move, modified_board.side_to_move)
    }

    #[test]
    fn test_generate_pawn_moves() {
        let mut board = Board::empty();

        let piece = Piece::Pawn;
        let color = Color::White;
        let from = pos("e2");
        let expected = vec![
            ChessMove {
                from: pos("e2"),
                to: pos("e3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("e2"),
                to: pos("e4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
        ];

        board.squares[from].0 = Some((piece, color));
        let mut moves = Vec::new();
        board.generate_pawn_moves(from, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_pawn_moves_single_move_after_start_rank() {
        let mut board = Board::empty();

        let piece = Piece::Pawn;
        let color = Color::White;
        let from = pos("e3");
        let expected = vec![ChessMove {
            from: pos("e3"),
            to: pos("e4"),
            capture: false,
            move_type: ChessMoveType::Normal,
        }];

        board.squares[from].0 = Some((piece, color));
        let mut moves = Vec::new();
        board.generate_pawn_moves(from, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_pawn_moves_blocked_double_move() {
        // Test that a pawn on its starting rank cannot double-move if destination is blocked
        let mut board = Board::empty();

        // Place white pawn on h2
        board.squares[pos("h2")].0 = Some((Piece::Pawn, Color::White));
        // Place a piece on h4 (blocking the double move destination)
        board.squares[pos("h4")].0 = Some((Piece::Queen, Color::Black));

        // The pawn should only be able to move to h3
        let expected = vec![ChessMove {
            from: pos("h2"),
            to: pos("h3"),
            capture: false,
            move_type: ChessMoveType::Normal,
        }];

        let mut moves = Vec::new();
        board.generate_pawn_moves(pos("h2"), Color::White, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_pawn_moves_capture_opponent() {
        let mut board = Board::empty();

        let piece1 = Piece::Pawn;
        let color1 = Color::White;
        let from1 = pos("d4");
        let piece2 = Piece::Pawn;
        let color2 = Color::Black;
        let from2 = pos("e5");
        let piece3 = Piece::Pawn;
        let color3 = Color::Black;
        let from3 = pos("c5");

        board.squares[from1].0 = Some((piece1, color1));
        board.squares[from2].0 = Some((piece2, color2));
        board.squares[from3].0 = Some((piece3, color3));

        let expected = vec![
            ChessMove {
                from: pos("d4"),
                to: pos("d5"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e5"),
                capture: true,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c5"),
                capture: true,
                move_type: ChessMoveType::Normal,
            },
        ];

        let mut moves = Vec::new();
        board.generate_pawn_moves(from1, color1, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_pawn_moves_en_passant() {
        // Test white pawn capturing black pawn via en passant
        let mut board = Board::empty();

        // Place white pawn on e5
        board.squares[pos("e5")].0 = Some((Piece::Pawn, Color::White));
        // Place black pawn on d5
        board.squares[pos("d5")].0 = Some((Piece::Pawn, Color::Black));
        // Set en passant target to d6 (as if black pawn just moved from d7 to d5)
        board.en_passant_target = Some(pos("d6"));
        board.side_to_move = Color::White;

        let expected = vec![
            // Normal forward move
            ChessMove {
                from: pos("e5"),
                to: pos("e6"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            // En passant capture to d6
            ChessMove {
                from: pos("e5"),
                to: pos("d6"),
                capture: true,
                move_type: ChessMoveType::EnPassant,
            },
        ];

        let mut moves = Vec::new();
        board.generate_pawn_moves(pos("e5"), Color::White, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }

        // Test black pawn capturing white pawn via en passant
        let mut board = Board::empty();

        // Place black pawn on d4
        board.squares[pos("d4")].0 = Some((Piece::Pawn, Color::Black));
        // Place white pawn on e4
        board.squares[pos("e4")].0 = Some((Piece::Pawn, Color::White));
        // Set en passant target to e3 (as if white pawn just moved from e2 to e4)
        board.en_passant_target = Some(pos("e3"));
        board.side_to_move = Color::Black;

        let expected = vec![
            // Normal forward move
            ChessMove {
                from: pos("d4"),
                to: pos("d3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            // En passant capture to e3
            ChessMove {
                from: pos("d4"),
                to: pos("e3"),
                capture: true,
                move_type: ChessMoveType::EnPassant,
            },
        ];

        let mut moves = Vec::new();
        board.generate_pawn_moves(pos("d4"), Color::Black, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_pawn_moves_promotion() {
        // Test white pawn promotion by moving forward
        let mut board = Board::empty();

        // Place white pawn on e7 (one square away from promotion)
        board.squares[pos("e7")].0 = Some((Piece::Pawn, Color::White));
        board.side_to_move = Color::White;

        let expected = vec![ChessMove {
            from: pos("e7"),
            to: pos("e8"),
            capture: false,
            move_type: ChessMoveType::Promotion(Piece::Queen),
        }];

        let mut moves = Vec::new();
        board.generate_pawn_moves(pos("e7"), Color::White, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }

        // Test white pawn promotion by capturing
        let mut board = Board::empty();

        // Place white pawn on d7
        board.squares[pos("d7")].0 = Some((Piece::Pawn, Color::White));
        // Place black pieces on e8 and c8 for diagonal captures
        board.squares[pos("e8")].0 = Some((Piece::Rook, Color::Black));
        board.squares[pos("c8")].0 = Some((Piece::Knight, Color::Black));
        board.side_to_move = Color::White;

        let expected = vec![
            // Forward promotion
            ChessMove {
                from: pos("d7"),
                to: pos("d8"),
                capture: false,
                move_type: ChessMoveType::Promotion(Piece::Queen),
            },
            // Capture promotion to e8
            ChessMove {
                from: pos("d7"),
                to: pos("e8"),
                capture: true,
                move_type: ChessMoveType::Promotion(Piece::Queen),
            },
            // Capture promotion to c8
            ChessMove {
                from: pos("d7"),
                to: pos("c8"),
                capture: true,
                move_type: ChessMoveType::Promotion(Piece::Queen),
            },
        ];

        let mut moves = Vec::new();
        board.generate_pawn_moves(pos("d7"), Color::White, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }

        // Test black pawn promotion by moving forward
        let mut board = Board::empty();

        // Place black pawn on e2 (one square away from promotion)
        board.squares[pos("e2")].0 = Some((Piece::Pawn, Color::Black));
        board.side_to_move = Color::Black;

        let expected = vec![ChessMove {
            from: pos("e2"),
            to: pos("e1"),
            capture: false,
            move_type: ChessMoveType::Promotion(Piece::Queen),
        }];

        let mut moves = Vec::new();
        board.generate_pawn_moves(pos("e2"), Color::Black, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }

        // Test black pawn promotion by capturing
        let mut board = Board::empty();

        // Place black pawn on d2
        board.squares[pos("d2")].0 = Some((Piece::Pawn, Color::Black));
        // Place white pieces on e1 and c1 for diagonal captures
        board.squares[pos("e1")].0 = Some((Piece::Rook, Color::White));
        board.squares[pos("c1")].0 = Some((Piece::Knight, Color::White));
        board.side_to_move = Color::Black;

        let expected = vec![
            // Forward promotion
            ChessMove {
                from: pos("d2"),
                to: pos("d1"),
                capture: false,
                move_type: ChessMoveType::Promotion(Piece::Queen),
            },
            // Capture promotion to e1
            ChessMove {
                from: pos("d2"),
                to: pos("e1"),
                capture: true,
                move_type: ChessMoveType::Promotion(Piece::Queen),
            },
            // Capture promotion to c1
            ChessMove {
                from: pos("d2"),
                to: pos("c1"),
                capture: true,
                move_type: ChessMoveType::Promotion(Piece::Queen),
            },
        ];

        let mut moves = Vec::new();
        board.generate_pawn_moves(pos("d2"), Color::Black, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_rook_moves() {
        let mut board = Board::empty();

        let piece = Piece::Rook;
        let color = Color::White;
        let from = pos("a1");

        let mut expected = vec![];

        for square in ["a2", "a3", "a4", "a5", "a6", "a7", "a8"] {
            expected.push(ChessMove {
                from: pos("a1"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        for square in ["b1", "c1", "d1", "e1", "f1", "g1", "h1"] {
            expected.push(ChessMove {
                from: pos("a1"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        board.squares[from].0 = Some((piece, color));
        let mut moves = Vec::new();
        board.generate_rook_moves(from, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_rook_moves_blocked_by_teammate() {
        let mut board = Board::empty();

        let color = Color::White;
        let piece1 = Piece::Rook;
        let from1 = pos("a1");
        let piece2 = Piece::Pawn;
        let from2 = pos("b1");

        let mut expected = vec![];
        for square in ["a2", "a3", "a4", "a5", "a6", "a7", "a8"] {
            expected.push(ChessMove {
                from: pos("a1"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        board.squares[from1].0 = Some((piece1, color));
        board.squares[from2].0 = Some((piece2, color));

        let mut moves = Vec::new();
        board.generate_rook_moves(from1, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_rook_moves_capture_opponent() {
        let mut board = Board::empty();

        let piece1 = Piece::Rook;
        let color1 = Color::White;
        let from1 = pos("a1");
        let piece2 = Piece::Rook;
        let color2 = Color::Black;
        let from2 = pos("h1");

        let mut expected = vec![];

        for square in ["a2", "a3", "a4", "a5", "a6", "a7", "a8"] {
            expected.push(ChessMove {
                from: pos("a1"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        for square in ["b1", "c1", "d1", "e1", "f1", "g1"] {
            expected.push(ChessMove {
                from: pos("a1"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        expected.push(ChessMove {
            from: pos("a1"),
            to: pos("h1"),
            capture: true,
            move_type: ChessMoveType::Normal,
        });

        board.squares[from1].0 = Some((piece1, color1));
        board.squares[from2].0 = Some((piece2, color2));

        let mut moves = Vec::new();
        board.generate_rook_moves(from1, color1, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_knight_moves() {
        let mut board = Board::empty();

        let color = Color::White;
        let piece1 = Piece::Knight;
        let from1 = pos("a2");
        let piece2 = Piece::Pawn;
        let from2 = pos("b2");

        board.squares[from1].0 = Some((piece1, color));
        board.squares[from2].0 = Some((piece2, color));

        let expected = vec![
            ChessMove {
                from: pos("a2"),
                to: pos("b4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c1"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
        ];

        let mut moves = Vec::new();
        board.generate_knight_moves(from1, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_knight_moves_blocked_by_teammate() {
        let mut board = Board::empty();

        let color = Color::White;
        let piece1 = Piece::Knight;
        let from1 = pos("a2");
        let piece2 = Piece::Pawn;
        let from2 = pos("c1");

        board.squares[from1].0 = Some((piece1, color));
        board.squares[from2].0 = Some((piece2, color));

        let expected = vec![
            ChessMove {
                from: pos("a2"),
                to: pos("b4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
        ];

        let mut moves = Vec::new();
        board.generate_knight_moves(from1, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_knight_moves_capture_opponent() {
        let mut board = Board::empty();

        let piece1 = Piece::Knight;
        let color1 = Color::White;
        let from1 = pos("a2");
        let piece2 = Piece::Pawn;
        let color2 = Color::Black;
        let from2 = pos("c3");

        board.squares[from1].0 = Some((piece1, color1));
        board.squares[from2].0 = Some((piece2, color2));

        let expected = vec![
            ChessMove {
                from: pos("a2"),
                to: pos("b4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c3"),
                capture: true,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c1"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
        ];

        let mut moves = Vec::new();
        board.generate_knight_moves(from1, color1, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_bishop_moves() {
        let mut board = Board::empty();

        let piece = Piece::Bishop;
        let color = Color::White;
        let from = pos("d4");

        let mut expected = vec![];

        // diagonal up-right
        for square in ["e5", "f6", "g7", "h8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        board.squares[from].0 = Some((piece, color));
        let mut moves = Vec::new();
        board.generate_bishop_moves(from, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_bishop_moves_blocked_by_teammate() {
        let mut board = Board::empty();

        let color = Color::White;
        let piece1 = Piece::Bishop;
        let from1 = pos("d4");
        let piece2 = Piece::Pawn;
        let from2 = pos("f6");

        let mut expected = vec![];

        // diagonal up-right (blocked at f6)
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("e5"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        board.squares[from1].0 = Some((piece1, color));
        board.squares[from2].0 = Some((piece2, color));

        let mut moves = Vec::new();
        board.generate_bishop_moves(from1, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_bishop_moves_capture_opponent() {
        let mut board = Board::empty();

        let piece1 = Piece::Bishop;
        let color1 = Color::White;
        let from1 = pos("d4");
        let piece2 = Piece::Pawn;
        let color2 = Color::Black;
        let from2 = pos("f6");

        let mut expected = vec![];

        // diagonal up-right (capture at f6)
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("e5"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("f6"),
            capture: true,
            move_type: ChessMoveType::Normal,
        });

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        board.squares[from1].0 = Some((piece1, color1));
        board.squares[from2].0 = Some((piece2, color2));

        let mut moves = Vec::new();
        board.generate_bishop_moves(from1, color1, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_queen_moves() {
        let mut board = Board::empty();

        let piece = Piece::Queen;
        let color = Color::White;
        let from = pos("d4");

        let mut expected = vec![];

        // horizontal right
        for square in ["e4", "f4", "g4", "h4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // horizontal left
        for square in ["c4", "b4", "a4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // vertical up
        for square in ["d5", "d6", "d7", "d8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // vertical down
        for square in ["d3", "d2", "d1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal up-right
        for square in ["e5", "f6", "g7", "h8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        board.squares[from].0 = Some((piece, color));
        let mut moves = Vec::new();
        board.generate_queen_moves(from, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_queen_moves_blocked_by_teammate() {
        let mut board = Board::empty();

        let color = Color::White;
        let piece1 = Piece::Queen;
        let from1 = pos("d4");
        let piece2 = Piece::Pawn;
        let from2 = pos("d6");

        let mut expected = vec![];

        // horizontal right
        for square in ["e4", "f4", "g4", "h4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // horizontal left
        for square in ["c4", "b4", "a4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // vertical up (blocked at d6)
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("d5"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });

        // vertical down
        for square in ["d3", "d2", "d1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal up-right
        for square in ["e5", "f6", "g7", "h8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        board.squares[from1].0 = Some((piece1, color));
        board.squares[from2].0 = Some((piece2, color));

        let mut moves = Vec::new();
        board.generate_queen_moves(from1, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_queen_moves_capture_opponent() {
        let mut board = Board::empty();

        let piece1 = Piece::Queen;
        let color1 = Color::White;
        let from1 = pos("d4");
        let piece2 = Piece::Pawn;
        let color2 = Color::Black;
        let from2 = pos("d6");

        let mut expected = vec![];

        // horizontal right
        for square in ["e4", "f4", "g4", "h4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // horizontal left
        for square in ["c4", "b4", "a4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // vertical up (capture at d6)
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("d5"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("d6"),
            capture: true,
            move_type: ChessMoveType::Normal,
        });

        // vertical down
        for square in ["d3", "d2", "d1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal up-right
        for square in ["e5", "f6", "g7", "h8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        board.squares[from1].0 = Some((piece1, color1));
        board.squares[from2].0 = Some((piece2, color2));

        let mut moves = Vec::new();
        board.generate_queen_moves(from1, color1, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_king_moves() {
        let mut board = Board::empty();

        let piece = Piece::King;
        let color = Color::White;
        let from = pos("d4");

        let expected = vec![
            ChessMove {
                from: pos("d4"),
                to: pos("d5"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("d3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e5"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c5"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
        ];

        board.squares[from].0 = Some((piece, color));
        let mut moves = Vec::new();
        board.generate_king_moves(from, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_king_moves_blocked_by_teammate() {
        let mut board = Board::empty();

        let color = Color::White;
        let piece1 = Piece::King;
        let from1 = pos("d4");
        let piece2 = Piece::Pawn;
        let from2 = pos("e5");

        let expected = vec![
            ChessMove {
                from: pos("d4"),
                to: pos("d5"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("d3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c5"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
        ];

        board.squares[from1].0 = Some((piece1, color));
        board.squares[from2].0 = Some((piece2, color));

        let mut moves = Vec::new();
        board.generate_king_moves(from1, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_king_moves_capture_opponent() {
        let mut board = Board::empty();

        let piece1 = Piece::King;
        let color1 = Color::White;
        let from1 = pos("d4");
        let piece2 = Piece::Pawn;
        let color2 = Color::Black;
        let from2 = pos("e5");

        let expected = vec![
            ChessMove {
                from: pos("d4"),
                to: pos("d5"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("d3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e5"),
                capture: true,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c5"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
        ];

        board.squares[from1].0 = Some((piece1, color1));
        board.squares[from2].0 = Some((piece2, color2));

        let mut moves = Vec::new();
        board.generate_king_moves(from1, color1, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_king_moves_castling() {
        // Test white kingside castling (O-O)
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("h1")].0 = Some((Piece::Rook, Color::White));
        board.white_king_pos = pos("e1");
        board.side_to_move = Color::White;

        let mut moves = Vec::new();
        board.generate_king_moves(pos("e1"), Color::White, &mut moves);
        let kingside_castle = ChessMove {
            from: pos("e1"),
            to: pos("g1"),
            capture: false,
            move_type: ChessMoveType::Castle,
        };
        assert!(
            moves.contains(&kingside_castle),
            "White kingside castling should be available"
        );

        // Test white queenside castling (O-O-O)
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("a1")].0 = Some((Piece::Rook, Color::White));
        board.white_king_pos = pos("e1");
        board.side_to_move = Color::White;

        let mut moves = Vec::new();
        board.generate_king_moves(pos("e1"), Color::White, &mut moves);
        let queenside_castle = ChessMove {
            from: pos("e1"),
            to: pos("c1"),
            capture: false,
            move_type: ChessMoveType::Castle,
        };
        assert!(
            moves.contains(&queenside_castle),
            "White queenside castling should be available"
        );

        // Test black kingside castling
        let mut board = Board::empty();
        board.squares[pos("e8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("h8")].0 = Some((Piece::Rook, Color::Black));
        board.black_king_pos = pos("e8");
        board.side_to_move = Color::Black;

        let mut moves = Vec::new();
        board.generate_king_moves(pos("e8"), Color::Black, &mut moves);
        let kingside_castle = ChessMove {
            from: pos("e8"),
            to: pos("g8"),
            capture: false,
            move_type: ChessMoveType::Castle,
        };
        assert!(
            moves.contains(&kingside_castle),
            "Black kingside castling should be available"
        );

        // Test black queenside castling
        let mut board = Board::empty();
        board.squares[pos("e8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("a8")].0 = Some((Piece::Rook, Color::Black));
        board.black_king_pos = pos("e8");
        board.side_to_move = Color::Black;

        let mut moves = Vec::new();
        board.generate_king_moves(pos("e8"), Color::Black, &mut moves);
        let queenside_castle = ChessMove {
            from: pos("e8"),
            to: pos("c8"),
            capture: false,
            move_type: ChessMoveType::Castle,
        };
        assert!(
            moves.contains(&queenside_castle),
            "Black queenside castling should be available"
        );

        // Test castling not available when king has moved
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("h1")].0 = Some((Piece::Rook, Color::White));
        board.white_king_pos = pos("e1");
        board.white_king_moved = true;
        board.side_to_move = Color::White;

        let mut moves = Vec::new();
        board.generate_king_moves(pos("e1"), Color::White, &mut moves);
        let kingside_castle = ChessMove {
            from: pos("e1"),
            to: pos("g1"),
            capture: false,
            move_type: ChessMoveType::Castle,
        };
        assert!(
            !moves.contains(&kingside_castle),
            "Castling should not be available when king has moved"
        );

        // Test castling not available when rook has moved
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("h1")].0 = Some((Piece::Rook, Color::White));
        board.white_king_pos = pos("e1");
        board.white_kingside_rook_moved = true;
        board.side_to_move = Color::White;

        let mut moves = Vec::new();
        board.generate_king_moves(pos("e1"), Color::White, &mut moves);
        let kingside_castle = ChessMove {
            from: pos("e1"),
            to: pos("g1"),
            capture: false,
            move_type: ChessMoveType::Castle,
        };
        assert!(
            !moves.contains(&kingside_castle),
            "Castling should not be available when rook has moved"
        );

        // Test castling not available when squares are occupied
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("h1")].0 = Some((Piece::Rook, Color::White));
        board.squares[pos("f1")].0 = Some((Piece::Knight, Color::White)); // blocking piece
        board.white_king_pos = pos("e1");
        board.side_to_move = Color::White;

        let mut moves = Vec::new();
        board.generate_king_moves(pos("e1"), Color::White, &mut moves);
        let kingside_castle = ChessMove {
            from: pos("e1"),
            to: pos("g1"),
            capture: false,
            move_type: ChessMoveType::Castle,
        };
        assert!(
            !moves.contains(&kingside_castle),
            "Castling should not be available when squares are occupied"
        );
    }

    #[test]
    fn test_is_in_check() {
        // Test white king in check from black rook
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("e8")].0 = Some((Piece::Rook, Color::Black));
        board.white_king_pos = pos("e1");
        board.black_king_pos = pos("h8");
        board.side_to_move = Color::White;

        assert!(
            board.is_in_check(Color::White),
            "White king should be in check from black rook"
        );
        assert!(
            !board.is_in_check(Color::Black),
            "Black king should not be in check"
        );

        // Test black king in check from white queen
        let mut board = Board::empty();
        board.squares[pos("e8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("e1")].0 = Some((Piece::Queen, Color::White));
        board.white_king_pos = pos("a1");
        board.black_king_pos = pos("e8");
        board.side_to_move = Color::Black;

        assert!(
            board.is_in_check(Color::Black),
            "Black king should be in check from white queen"
        );
    }

    #[test]
    fn test_checkmate_back_rank() {
        // Classic back rank mate: white king on g1, black rook on a1, white pawns blocking escape
        let mut board = Board::empty();
        board.squares[pos("g1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("f2")].0 = Some((Piece::Pawn, Color::White));
        board.squares[pos("g2")].0 = Some((Piece::Pawn, Color::White));
        board.squares[pos("h2")].0 = Some((Piece::Pawn, Color::White));
        board.squares[pos("a1")].0 = Some((Piece::Rook, Color::Black)); // Attacks along first rank
        board.squares[pos("e8")].0 = Some((Piece::King, Color::Black));
        board.white_king_pos = pos("g1");
        board.black_king_pos = pos("e8");
        board.side_to_move = Color::White;

        assert!(
            board.is_checkmate(),
            "White should be in checkmate (back rank mate)"
        );
    }

    #[test]
    fn test_checkmate_queen_and_king() {
        // Queen and king checkmate in the corner
        let mut board = Board::empty();
        board.squares[pos("a8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("b6")].0 = Some((Piece::Queen, Color::White));
        board.squares[pos("b8")].0 = Some((Piece::King, Color::White));
        board.white_king_pos = pos("b8");
        board.black_king_pos = pos("a8");
        board.side_to_move = Color::Black;

        assert!(
            board.is_checkmate(),
            "Black should be in checkmate (queen and king mate)"
        );
    }

    #[test]
    fn test_stalemate() {
        // Classic stalemate: black king on a8, white king on c7, white queen on c6
        let mut board = Board::empty();
        board.squares[pos("a8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("c6")].0 = Some((Piece::King, Color::White));
        board.squares[pos("c7")].0 = Some((Piece::Queen, Color::White));
        board.white_king_pos = pos("c6");
        board.black_king_pos = pos("a8");
        board.side_to_move = Color::Black;

        assert!(
            board.is_stalemate(),
            "Black should be in stalemate (no legal moves, not in check)"
        );
        assert!(
            !board.is_checkmate(),
            "This should be stalemate, not checkmate"
        );
    }

    #[test]
    fn test_not_checkmate_can_block() {
        // King in check but can be blocked
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("e8")].0 = Some((Piece::Rook, Color::Black));
        board.squares[pos("d2")].0 = Some((Piece::Bishop, Color::White)); // Can block on e2
        board.white_king_pos = pos("e1");
        board.black_king_pos = pos("h8");
        board.side_to_move = Color::White;

        assert!(board.is_in_check(Color::White), "White should be in check");
        assert!(
            !board.is_checkmate(),
            "White should not be in checkmate (can block with bishop)"
        );
    }

    #[test]
    fn test_not_checkmate_can_escape() {
        // King in check but has escape square
        let mut board = Board::empty();
        board.squares[pos("e4")].0 = Some((Piece::King, Color::White));
        board.squares[pos("e8")].0 = Some((Piece::Rook, Color::Black));
        board.white_king_pos = pos("e4");
        board.black_king_pos = pos("h8");
        board.side_to_move = Color::White;

        assert!(board.is_in_check(Color::White), "White should be in check");
        assert!(
            !board.is_checkmate(),
            "White should not be in checkmate (king can move)"
        );
    }

    #[test]
    fn test_not_checkmate_can_capture_attacker() {
        // King in check but can capture the attacking piece
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("e2")].0 = Some((Piece::Rook, Color::Black)); // King can capture
        board.white_king_pos = pos("e1");
        board.black_king_pos = pos("h8");
        board.side_to_move = Color::White;

        assert!(board.is_in_check(Color::White), "White should be in check");
        assert!(
            !board.is_checkmate(),
            "White should not be in checkmate (can capture rook)"
        );
    }

    #[test]
    fn test_hash_updates_correctly() {
        let mut board = Board::new();
        let initial_hash = board.zobrist_hash;

        // Make a move
        let moves = board.generate_legal_moves();
        let chess_move = moves[0];
        let state = board.apply_move(chess_move);

        let after_move_hash = board.zobrist_hash;
        assert_ne!(
            initial_hash, after_move_hash,
            "Hash should change after move"
        );

        // Undo the move
        board.undo_move(state);
        assert_eq!(
            initial_hash, board.zobrist_hash,
            "Hash should restore after undo"
        );

        // Verify hash matches full computation
        let computed_hash = compute_hash(&board);
        assert_eq!(
            board.zobrist_hash, computed_hash,
            "Incremental hash should match full computation"
        );
    }

    #[test]
    fn test_transposition_same_position() {
        // Two different move orders reaching same position
        let mut board1 = Board::new();

        // Path 1: e2e4, e7e5, Nf3, Nc6, d2d3
        let moves1 = parse_moves(&["e2e4", "e7e5", "g1f3", "b8c6", "d2d3"]);
        for m in moves1 {
            board1.apply_move(m);
        }
        let hash1 = board1.zobrist_hash;

        // Path 2: Nf3, Nc6, e2e4, e7e5, d2d3
        let mut board2 = Board::new();
        let moves2 = parse_moves(&["g1f3", "b8c6", "e2e4", "e7e5", "d2d3"]);
        for m in moves2 {
            board2.apply_move(m);
        }
        let hash2 = board2.zobrist_hash;

        assert_eq!(hash1, hash2, "Same position should have same hash!");
    }

    fn pos(s: &str) -> usize {
        let bytes = s.as_bytes();
        let file = (bytes[0] - b'a') as usize;
        let rank = (bytes[1] - b'1') as usize;
        rank * 8 + file
    }

    fn parse_moves(moves: &[&str]) -> Vec<ChessMove> {
        moves
            .iter()
            .map(|move_str| {
                // Parse move format: "e2e4" (from square, to square)
                let from = pos(&move_str[0..2]);
                let to = pos(&move_str[2..4]);
                ChessMove {
                    from,
                    to,
                    capture: false,
                    move_type: ChessMoveType::Normal,
                }
            })
            .collect()
    }
}
