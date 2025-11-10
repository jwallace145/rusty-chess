use super::chess_move::{ChessMove, ChessMoveState, ChessMoveType};
use super::{Color, Piece};

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
}

impl Board {
    pub fn empty() -> Self {
        Self {
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
        }
    }

    pub fn new() -> Self {
        use Color::*;
        use Piece::*;

        let mut squares = [Square(None); 64];

        let white_back_rank = [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook];
        for i in 0..8 {
            squares[i] = Square(Some((white_back_rank[i], White)));
            squares[i + 8] = Square(Some((Pawn, White)));
        }

        let black_back_rank = [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook];
        for i in 0..8 {
            squares[i + 56] = Square(Some((black_back_rank[i], Black)));
            squares[i + 48] = Square(Some((Pawn, Black)));
        }

        Self {
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
        }
    }

    pub fn switch_side(&mut self) {
        self.side_to_move = match self.side_to_move {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
    }

    pub fn generate_legal_moves(&self) -> Vec<ChessMove> {
        let pseudo_moves = self.generate_moves();
        let mut legal_moves = Vec::new();

        for m in pseudo_moves {
            let mut board_copy = *self;
            board_copy.apply_move(m);
            let king_square = board_copy.king_pos(self.side_to_move);
            if !board_copy.is_square_attacked(king_square, self.side_to_move.opponent()) {
                legal_moves.push(m);
            }
        }

        legal_moves
    }

    pub fn generate_moves(&self) -> Vec<ChessMove> {
        let mut moves = Vec::new();

        for (i, square) in self.squares.iter().enumerate() {
            if let Some((piece, color)) = square.0 {
                if color == self.side_to_move {
                    let piece_moves = self.generate_piece_moves(i, piece, color);
                    moves.extend(piece_moves);
                }
            }
        }

        moves
    }

    pub fn apply_move(&mut self, chess_move: ChessMove) -> ChessMoveState {
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
        };

        // Move the piece
        self.squares[chess_move.to].0 = moved_piece;
        self.squares[chess_move.from].0 = None;

        // Handle castling: move the rook as well
        if chess_move.moveType == ChessMoveType::Castle {
            if let Some((Piece::King, color)) = moved_piece {
                match color {
                    Color::White => {
                        if chess_move.to == 6 {
                            // Kingside castling: move rook from h1 (7) to f1 (5)
                            self.squares[5].0 = self.squares[7].0;
                            self.squares[7].0 = None;
                        } else if chess_move.to == 2 {
                            // Queenside castling: move rook from a1 (0) to d1 (3)
                            self.squares[3].0 = self.squares[0].0;
                            self.squares[0].0 = None;
                        }
                    }
                    Color::Black => {
                        if chess_move.to == 62 {
                            // Kingside castling: move rook from h8 (63) to f8 (61)
                            self.squares[61].0 = self.squares[63].0;
                            self.squares[63].0 = None;
                        } else if chess_move.to == 58 {
                            // Queenside castling: move rook from a8 (56) to d8 (59)
                            self.squares[59].0 = self.squares[56].0;
                            self.squares[56].0 = None;
                        }
                    }
                }
            }
        }

        // Update king position and king/rook moved flags for castling
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

        // Promote pawns to queens for simplicity
        if let Some((Piece::Pawn, color)) = self.squares[chess_move.to].0 {
            let rank = chess_move.to / 8;
            if (color == Color::White && rank == 7) || (color == Color::Black && rank == 0) {
                self.squares[chess_move.to].0 = Some((Piece::Queen, color));
            }
        }

        // Reset en passant target
        self.en_passant_target = None;

        if let Some((Piece::Pawn, color)) = moved_piece {
            let rank_from = chess_move.from / 8;
            let rank_to = chess_move.to / 8;

            // Set en passant target for double pawn move
            if (color == Color::White && rank_from == 1 && rank_to == 3)
                || (color == Color::Black && rank_from == 6 && rank_to == 4)
            {
                self.en_passant_target = if chess_move.to > chess_move.from {
                    Some(chess_move.from + (chess_move.to - chess_move.from) / 2)
                } else {
                    Some(chess_move.to + (chess_move.from - chess_move.to) / 2)
                };
            }

            // Capture en passant
            if Some(chess_move.to) == self.en_passant_target && chess_move.capture {
                let captured_pawn_square = if color == Color::White {
                    chess_move.to - 8
                } else {
                    chess_move.to + 8
                };
                // Remove captured pawn
                self.squares[captured_pawn_square].0 = None;
            }
        }

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

        // Restore king positions if the moved piece was a king for castling
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
    }

    fn generate_piece_moves(&self, index: usize, piece: Piece, color: Color) -> Vec<ChessMove> {
        match piece {
            Piece::Pawn => self.generate_pawn_moves(index, color),
            Piece::Rook => self.generate_rook_moves(index, color),
            Piece::Knight => self.generate_knight_moves(index, color),
            Piece::Bishop => self.generate_bishop_moves(index, color),
            Piece::Queen => self.generate_queen_moves(index, color),
            Piece::King => self.generate_king_moves(index, color),
        }
    }

    fn generate_pawn_moves(&self, index: usize, color: Color) -> Vec<ChessMove> {
        let mut moves = Vec::new();
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

                moves.push(ChessMove {
                    from: index,
                    to: forward_idx,
                    capture: false,
                    moveType: move_type,
                });

                if rank == start_rank {
                    let double_rank = (rank as isize + 2 * forward) as usize;
                    let double_idx = double_rank * 8 + file;
                    if self.squares[double_idx].0.is_none() {
                        moves.push(ChessMove {
                            from: index,
                            to: double_idx,
                            capture: false,
                            moveType: ChessMoveType::Normal,
                        });
                    }
                }
            }
        }

        // Diagonal captures
        for df in [-1, 1] {
            let new_file = file as isize + df;
            if new_file >= 0 && new_file < 8 && new_rank < 8 {
                let capture_idx = new_rank * 8 + new_file as usize;
                if let Some((_, target_color)) = self.squares[capture_idx].0 {
                    if target_color != color {
                        let move_type = if new_rank == promotion_rank {
                            ChessMoveType::Promotion(Piece::Queen)
                        } else {
                            ChessMoveType::Normal
                        };

                        moves.push(ChessMove {
                            from: index,
                            to: capture_idx,
                            capture: true,
                            moveType: move_type,
                        });
                    }
                } else if Some(capture_idx) == self.en_passant_target {
                    // en passant capture
                    moves.push(ChessMove {
                        from: index,
                        to: capture_idx,
                        capture: true,
                        moveType: ChessMoveType::EnPassant,
                    });
                }
            }
        }

        moves
    }

    fn generate_rook_moves(&self, index: usize, color: Color) -> Vec<ChessMove> {
        self.generate_sliding_moves(index, color, &[(1, 0), (-1, 0), (0, 1), (0, -1)])
    }

    fn generate_knight_moves(&self, index: usize, color: Color) -> Vec<ChessMove> {
        let mut moves = Vec::new();
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
            if new_rank >= 0 && new_rank < 8 && new_file >= 0 && new_file < 8 {
                let to = (new_rank * 8 + new_file) as usize;
                match self.squares[to].0 {
                    None => moves.push(ChessMove {
                        from: index,
                        to,
                        capture: false,
                        moveType: ChessMoveType::Normal,
                    }),
                    Some((_, target_color)) if target_color != color => moves.push(ChessMove {
                        from: index,
                        to,
                        capture: true,
                        moveType: ChessMoveType::Normal,
                    }),
                    _ => {}
                }
            }
        }

        moves
    }

    fn generate_bishop_moves(&self, index: usize, color: Color) -> Vec<ChessMove> {
        self.generate_sliding_moves(index, color, &[(1, 1), (1, -1), (-1, 1), (-1, -1)])
    }

    fn generate_queen_moves(&self, index: usize, color: Color) -> Vec<ChessMove> {
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
        )
    }

    fn generate_king_moves(&self, index: usize, color: Color) -> Vec<ChessMove> {
        let mut moves = Vec::new();
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
            if new_rank >= 0 && new_rank < 8 && new_file >= 0 && new_file < 8 {
                let to = (new_rank * 8 + new_file) as usize;
                match self.squares[to].0 {
                    None => moves.push(ChessMove {
                        from: index,
                        to,
                        capture: false,
                        moveType: ChessMoveType::Normal,
                    }),
                    Some((_, target_color)) if target_color != color => {
                        moves.push(ChessMove {
                            from: index,
                            to,
                            capture: true,
                            moveType: ChessMoveType::Normal,
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
                    moves.push(ChessMove {
                        from: index,
                        to: 6, // g1
                        capture: false,
                        moveType: ChessMoveType::Castle,
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
                    moves.push(ChessMove {
                        from: index,
                        to: 2, // c1
                        capture: false,
                        moveType: ChessMoveType::Castle,
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
                    moves.push(ChessMove {
                        from: index,
                        to: 62, // g8
                        capture: false,
                        moveType: ChessMoveType::Castle,
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
                    moves.push(ChessMove {
                        from: index,
                        to: 58, // c8
                        capture: false,
                        moveType: ChessMoveType::Castle,
                    });
                }
            }
        }

        moves
    }

    fn generate_sliding_moves(
        &self,
        index: usize,
        color: Color,
        directions: &[(isize, isize)],
    ) -> Vec<ChessMove> {
        let mut moves = Vec::new();
        let rank = (index / 8) as isize;
        let file = (index % 8) as isize;

        for (dr, df) in directions {
            let mut r = rank + dr;
            let mut f = file + df;
            while r >= 0 && r < 8 && f >= 0 && f < 8 {
                let to = (r * 8 + f) as usize;
                match self.squares[to].0 {
                    None => moves.push(ChessMove {
                        from: index,
                        to,
                        capture: false,
                        moveType: ChessMoveType::Normal,
                    }),
                    Some((_, target_color)) if target_color != color => {
                        moves.push(ChessMove {
                            from: index,
                            to,
                            capture: true,
                            moveType: ChessMoveType::Normal,
                        });
                        break;
                    }
                    _ => break,
                }
                r += dr;
                f += df;
            }
        }

        moves
    }

    pub fn is_square_attacked(&self, square: usize, attacker_color: Color) -> bool {
        for (i, sq) in self.squares.iter().enumerate() {
            if let Some((piece, color)) = sq.0 {
                if color == attacker_color {
                    let attacks = self.generate_piece_moves(i, piece, color);
                    if attacks.iter().any(|m| m.to == square) {
                        return true;
                    }
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

        for i in 0..8 {
            assert_eq!(board.squares[i].0, Some((white_back_rank[i], Color::White)));
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
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("e2"),
                to: pos("e4"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
        ];

        board.squares[from].0 = Some((piece, color));
        let moves = board.generate_pawn_moves(from, color);

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
            moveType: ChessMoveType::Normal,
        }];

        board.squares[from].0 = Some((piece, color));
        let moves = board.generate_pawn_moves(from, color);

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
            moveType: ChessMoveType::Normal,
        }];

        let moves = board.generate_pawn_moves(pos("h2"), Color::White);

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
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e5"),
                capture: true,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c5"),
                capture: true,
                moveType: ChessMoveType::Normal,
            },
        ];

        let moves = board.generate_pawn_moves(from1, color1);

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
                moveType: ChessMoveType::Normal,
            },
            // En passant capture to d6
            ChessMove {
                from: pos("e5"),
                to: pos("d6"),
                capture: true,
                moveType: ChessMoveType::EnPassant,
            },
        ];

        let moves = board.generate_pawn_moves(pos("e5"), Color::White);

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
                moveType: ChessMoveType::Normal,
            },
            // En passant capture to e3
            ChessMove {
                from: pos("d4"),
                to: pos("e3"),
                capture: true,
                moveType: ChessMoveType::EnPassant,
            },
        ];

        let moves = board.generate_pawn_moves(pos("d4"), Color::Black);

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
            moveType: ChessMoveType::Promotion(Piece::Queen),
        }];

        let moves = board.generate_pawn_moves(pos("e7"), Color::White);

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
                moveType: ChessMoveType::Promotion(Piece::Queen),
            },
            // Capture promotion to e8
            ChessMove {
                from: pos("d7"),
                to: pos("e8"),
                capture: true,
                moveType: ChessMoveType::Promotion(Piece::Queen),
            },
            // Capture promotion to c8
            ChessMove {
                from: pos("d7"),
                to: pos("c8"),
                capture: true,
                moveType: ChessMoveType::Promotion(Piece::Queen),
            },
        ];

        let moves = board.generate_pawn_moves(pos("d7"), Color::White);

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
            moveType: ChessMoveType::Promotion(Piece::Queen),
        }];

        let moves = board.generate_pawn_moves(pos("e2"), Color::Black);

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
                moveType: ChessMoveType::Promotion(Piece::Queen),
            },
            // Capture promotion to e1
            ChessMove {
                from: pos("d2"),
                to: pos("e1"),
                capture: true,
                moveType: ChessMoveType::Promotion(Piece::Queen),
            },
            // Capture promotion to c1
            ChessMove {
                from: pos("d2"),
                to: pos("c1"),
                capture: true,
                moveType: ChessMoveType::Promotion(Piece::Queen),
            },
        ];

        let moves = board.generate_pawn_moves(pos("d2"), Color::Black);

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
                moveType: ChessMoveType::Normal,
            });
        }

        for square in ["b1", "c1", "d1", "e1", "f1", "g1", "h1"] {
            expected.push(ChessMove {
                from: pos("a1"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        board.squares[from].0 = Some((piece, color));
        let moves = board.generate_rook_moves(from, color);

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
                moveType: ChessMoveType::Normal,
            });
        }

        board.squares[from1].0 = Some((piece1, color));
        board.squares[from2].0 = Some((piece2, color));

        let moves = board.generate_rook_moves(from1, color);

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
                moveType: ChessMoveType::Normal,
            });
        }

        for square in ["b1", "c1", "d1", "e1", "f1", "g1"] {
            expected.push(ChessMove {
                from: pos("a1"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        expected.push(ChessMove {
            from: pos("a1"),
            to: pos("h1"),
            capture: true,
            moveType: ChessMoveType::Normal,
        });

        board.squares[from1].0 = Some((piece1, color1));
        board.squares[from2].0 = Some((piece2, color2));

        let moves = board.generate_rook_moves(from1, color1);

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
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c3"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c1"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
        ];

        let moves = board.generate_knight_moves(from1, color);

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
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c3"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
        ];

        let moves = board.generate_knight_moves(from1, color);

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
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c3"),
                capture: true,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c1"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
        ];

        let moves = board.generate_knight_moves(from1, color1);

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
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        board.squares[from].0 = Some((piece, color));
        let moves = board.generate_bishop_moves(from, color);

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
            moveType: ChessMoveType::Normal,
        });

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        board.squares[from1].0 = Some((piece1, color));
        board.squares[from2].0 = Some((piece2, color));

        let moves = board.generate_bishop_moves(from1, color);

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
            moveType: ChessMoveType::Normal,
        });
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("f6"),
            capture: true,
            moveType: ChessMoveType::Normal,
        });

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        board.squares[from1].0 = Some((piece1, color1));
        board.squares[from2].0 = Some((piece2, color2));

        let moves = board.generate_bishop_moves(from1, color1);

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
                moveType: ChessMoveType::Normal,
            });
        }

        // horizontal left
        for square in ["c4", "b4", "a4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // vertical up
        for square in ["d5", "d6", "d7", "d8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // vertical down
        for square in ["d3", "d2", "d1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal up-right
        for square in ["e5", "f6", "g7", "h8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        board.squares[from].0 = Some((piece, color));
        let moves = board.generate_queen_moves(from, color);

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
                moveType: ChessMoveType::Normal,
            });
        }

        // horizontal left
        for square in ["c4", "b4", "a4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // vertical up (blocked at d6)
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("d5"),
            capture: false,
            moveType: ChessMoveType::Normal,
        });

        // vertical down
        for square in ["d3", "d2", "d1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal up-right
        for square in ["e5", "f6", "g7", "h8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        board.squares[from1].0 = Some((piece1, color));
        board.squares[from2].0 = Some((piece2, color));

        let moves = board.generate_queen_moves(from1, color);

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
                moveType: ChessMoveType::Normal,
            });
        }

        // horizontal left
        for square in ["c4", "b4", "a4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // vertical up (capture at d6)
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("d5"),
            capture: false,
            moveType: ChessMoveType::Normal,
        });
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("d6"),
            capture: true,
            moveType: ChessMoveType::Normal,
        });

        // vertical down
        for square in ["d3", "d2", "d1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal up-right
        for square in ["e5", "f6", "g7", "h8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                moveType: ChessMoveType::Normal,
            });
        }

        board.squares[from1].0 = Some((piece1, color1));
        board.squares[from2].0 = Some((piece2, color2));

        let moves = board.generate_queen_moves(from1, color1);

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
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("d3"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e4"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c4"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e5"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e3"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c5"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c3"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
        ];

        board.squares[from].0 = Some((piece, color));
        let moves = board.generate_king_moves(from, color);

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
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("d3"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e4"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c4"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e3"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c5"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c3"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
        ];

        board.squares[from1].0 = Some((piece1, color));
        board.squares[from2].0 = Some((piece2, color));

        let moves = board.generate_king_moves(from1, color);

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
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("d3"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e4"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c4"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e5"),
                capture: true,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e3"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c5"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c3"),
                capture: false,
                moveType: ChessMoveType::Normal,
            },
        ];

        board.squares[from1].0 = Some((piece1, color1));
        board.squares[from2].0 = Some((piece2, color2));

        let moves = board.generate_king_moves(from1, color1);

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

        let moves = board.generate_king_moves(pos("e1"), Color::White);
        let kingside_castle = ChessMove {
            from: pos("e1"),
            to: pos("g1"),
            capture: false,
            moveType: ChessMoveType::Castle,
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

        let moves = board.generate_king_moves(pos("e1"), Color::White);
        let queenside_castle = ChessMove {
            from: pos("e1"),
            to: pos("c1"),
            capture: false,
            moveType: ChessMoveType::Castle,
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

        let moves = board.generate_king_moves(pos("e8"), Color::Black);
        let kingside_castle = ChessMove {
            from: pos("e8"),
            to: pos("g8"),
            capture: false,
            moveType: ChessMoveType::Castle,
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

        let moves = board.generate_king_moves(pos("e8"), Color::Black);
        let queenside_castle = ChessMove {
            from: pos("e8"),
            to: pos("c8"),
            capture: false,
            moveType: ChessMoveType::Castle,
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

        let moves = board.generate_king_moves(pos("e1"), Color::White);
        let kingside_castle = ChessMove {
            from: pos("e1"),
            to: pos("g1"),
            capture: false,
            moveType: ChessMoveType::Castle,
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

        let moves = board.generate_king_moves(pos("e1"), Color::White);
        let kingside_castle = ChessMove {
            from: pos("e1"),
            to: pos("g1"),
            capture: false,
            moveType: ChessMoveType::Castle,
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

        let moves = board.generate_king_moves(pos("e1"), Color::White);
        let kingside_castle = ChessMove {
            from: pos("e1"),
            to: pos("g1"),
            capture: false,
            moveType: ChessMoveType::Castle,
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

    fn pos(s: &str) -> usize {
        let bytes = s.as_bytes();
        let file = (bytes[0] - b'a') as usize;
        let rank = (bytes[1] - b'1') as usize;
        rank * 8 + file
    }
}
