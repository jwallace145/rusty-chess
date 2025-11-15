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
    pub white_king_moved: bool,
    pub white_kingside_rook_moved: bool,
    pub white_queenside_rook_moved: bool,
    pub black_king_moved: bool,
    pub black_kingside_rook_moved: bool,
    pub black_queenside_rook_moved: bool,
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

    pub fn king_pos(&self, color: Color) -> usize {
        match color {
            Color::White => self.white_king_pos,
            Color::Black => self.black_king_pos,
        }
    }

    pub fn can_castle_white_kingside(&self) -> bool {
        !self.white_king_moved && !self.white_kingside_rook_moved
    }

    pub fn can_castle_white_queenside(&self) -> bool {
        !self.white_king_moved && !self.white_queenside_rook_moved
    }

    pub fn can_castle_black_kingside(&self) -> bool {
        !self.black_king_moved && !self.black_kingside_rook_moved
    }

    pub fn can_castle_black_queenside(&self) -> bool {
        !self.black_king_moved && !self.black_queenside_rook_moved
    }

    pub fn has_castled(&self, color: Color) -> bool {
        match color {
            Color::White => {
                // White king castled if it has moved and is on g1 (6) or c1 (2)
                self.white_king_moved && (self.white_king_pos == 6 || self.white_king_pos == 2)
            }
            Color::Black => {
                // Black king castled if it has moved and is on g8 (62) or c8 (58)
                self.black_king_moved && (self.black_king_pos == 62 || self.black_king_pos == 58)
            }
        }
    }

    pub fn count(&self, color: Color, piece: Piece) -> u32 {
        self.squares
            .iter()
            .filter(|square| {
                if let Some((p, c)) = square.0 {
                    p == piece && c == color
                } else {
                    false
                }
            })
            .count() as u32
    }

    pub fn parse_uci(&self, uci: &str) -> Result<ChessMove, String> {
        use super::move_generator::MoveGenerator;

        if uci.len() < 4 {
            return Err(format!("Invalid UCI move: {}", uci));
        }

        let from_str = &uci[0..2];
        let to_str = &uci[2..4];

        let from = parse_square(from_str)?;
        let to = parse_square(to_str)?;

        // Generate legal moves and find matching move
        let mut legal_moves = Vec::with_capacity(128);
        MoveGenerator::generate_legal_moves(self, &mut legal_moves);

        legal_moves
            .into_iter()
            .find(|m| m.from == from && m.to == to)
            .ok_or_else(|| format!("No legal move from {} to {}", from_str, to_str))
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

fn parse_square(s: &str) -> Result<usize, String> {
    if s.len() != 2 {
        return Err(format!("Invalid square: {}", s));
    }

    let bytes = s.as_bytes();
    let file = bytes[0];
    let rank = bytes[1];

    if !(b'a'..=b'h').contains(&file) {
        return Err(format!("Invalid file: {}", file as char));
    }

    if !(b'1'..=b'8').contains(&rank) {
        return Err(format!("Invalid rank: {}", rank as char));
    }

    let file_idx = (file - b'a') as usize;
    let rank_idx = (rank - b'1') as usize;

    Ok(rank_idx * 8 + file_idx)
}

#[cfg(test)]
mod tests {
    use super::super::MoveGenerator;
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
    fn test_hash_updates_correctly() {
        let mut board = Board::new();
        let initial_hash = board.zobrist_hash;

        // Make a move
        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_legal_moves(&board, &mut moves);
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
