use super::{ChessMove, ChessMoveState, Color, Piece};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Square(pub Option<(Piece, Color)>);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Board {
    pub squares: [Square; 64],
    pub side_to_move: Color,
}

impl Board {
    pub fn empty() -> Self {
        Self {
            squares: [Square(None); 64],
            side_to_move: Color::White,
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
        }
    }

    pub fn switch_side(&mut self) {
        self.side_to_move = match self.side_to_move {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
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

        self.squares[chess_move.to].0 = moved_piece;
        self.squares[chess_move.from].0 = None;

        let previous_side_to_move = self.side_to_move;

        self.switch_side();

        ChessMoveState {
            chess_move,
            moved_piece,
            captured_piece,
            previous_side_to_move,
        }
    }

    pub fn undo_move(&mut self, state: ChessMoveState) {
        let chess_move = state.chess_move;

        self.squares[chess_move.from].0 = state.moved_piece;
        self.squares[chess_move.to].0 = state.captured_piece;
        self.side_to_move = state.previous_side_to_move;
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

        let (forward, start_rank): (isize, usize) = match color {
            Color::White => (1, 1),
            Color::Black => (-1, 6),
        };

        // move forward one square
        let new_rank = (rank as isize + forward) as usize;
        if new_rank < 8 {
            let forward_idx = new_rank * 8 + file;
            if self.squares[forward_idx].0.is_none() {
                moves.push(ChessMove {
                    from: index,
                    to: forward_idx,
                    capture: false,
                });

                if rank == start_rank {
                    let double_rank = (rank as isize + 2 * forward) as usize;
                    let double_idx = double_rank * 8 + file;
                    moves.push(ChessMove {
                        from: index,
                        to: double_idx,
                        capture: false,
                    });
                }
            }
        }

        for df in [-1, 1] {
            let new_file = file as isize + df;
            if new_file >= 0 && new_file < 8 && new_rank < 8 {
                let capture_idx = new_rank * 8 + new_file as usize;
                if let Some((_, target_color)) = self.squares[capture_idx].0 {
                    if target_color != color {
                        moves.push(ChessMove {
                            from: index,
                            to: capture_idx,
                            capture: true,
                        })
                    }
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
                    }),
                    Some((_, target_color)) if target_color != color => moves.push(ChessMove {
                        from: index,
                        to,
                        capture: true,
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
                    }),
                    Some((_, target_color)) if target_color != color => {
                        moves.push(ChessMove {
                            from: index,
                            to,
                            capture: true,
                        });
                    }
                    _ => {}
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
                    }),
                    Some((_, target_color)) if target_color != color => {
                        moves.push(ChessMove {
                            from: index,
                            to,
                            capture: true,
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
            },
            ChessMove {
                from: pos("e2"),
                to: pos("e4"),
                capture: false,
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
        }];

        board.squares[from].0 = Some((piece, color));
        let moves = board.generate_pawn_moves(from, color);

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
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e5"),
                capture: true,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c5"),
                capture: true,
            },
        ];

        let moves = board.generate_pawn_moves(from1, color1);

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
            });
        }

        for square in ["b1", "c1", "d1", "e1", "f1", "g1", "h1"] {
            expected.push(ChessMove {
                from: pos("a1"),
                to: pos(square),
                capture: false,
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
            });
        }

        for square in ["b1", "c1", "d1", "e1", "f1", "g1"] {
            expected.push(ChessMove {
                from: pos("a1"),
                to: pos(square),
                capture: false,
            });
        }

        expected.push(ChessMove {
            from: pos("a1"),
            to: pos("h1"),
            capture: true,
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
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c3"),
                capture: false,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c1"),
                capture: false,
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
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c3"),
                capture: false,
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
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c3"),
                capture: true,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c1"),
                capture: false,
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
            });
        }

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
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
        });

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
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
        });
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("f6"),
            capture: true,
        });

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
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
            });
        }

        // horizontal left
        for square in ["c4", "b4", "a4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // vertical up
        for square in ["d5", "d6", "d7", "d8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // vertical down
        for square in ["d3", "d2", "d1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal up-right
        for square in ["e5", "f6", "g7", "h8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
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
            });
        }

        // horizontal left
        for square in ["c4", "b4", "a4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // vertical up (blocked at d6)
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("d5"),
            capture: false,
        });

        // vertical down
        for square in ["d3", "d2", "d1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal up-right
        for square in ["e5", "f6", "g7", "h8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
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
            });
        }

        // horizontal left
        for square in ["c4", "b4", "a4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // vertical up (capture at d6)
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("d5"),
            capture: false,
        });
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("d6"),
            capture: true,
        });

        // vertical down
        for square in ["d3", "d2", "d1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal up-right
        for square in ["e5", "f6", "g7", "h8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
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
            },
            ChessMove {
                from: pos("d4"),
                to: pos("d3"),
                capture: false,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e4"),
                capture: false,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c4"),
                capture: false,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e5"),
                capture: false,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e3"),
                capture: false,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c5"),
                capture: false,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c3"),
                capture: false,
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
            },
            ChessMove {
                from: pos("d4"),
                to: pos("d3"),
                capture: false,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e4"),
                capture: false,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c4"),
                capture: false,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e3"),
                capture: false,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c5"),
                capture: false,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c3"),
                capture: false,
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
            },
            ChessMove {
                from: pos("d4"),
                to: pos("d3"),
                capture: false,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e4"),
                capture: false,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c4"),
                capture: false,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e5"),
                capture: true,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e3"),
                capture: false,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c5"),
                capture: false,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c3"),
                capture: false,
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

    fn pos(s: &str) -> usize {
        let bytes = s.as_bytes();
        let file = (bytes[0] - b'a') as usize;
        let rank = (bytes[1] - b'1') as usize;
        rank * 8 + file
    }
}
