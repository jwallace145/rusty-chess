use super::{ChessMove, ChessMoveState, Color, Piece};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Square(pub Option<(Piece, Color)>);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Board {
    pub squares: [Square; 64],
    pub side_to_move: Color,
}

impl Board {
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
            previous_side_to_move
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
}
