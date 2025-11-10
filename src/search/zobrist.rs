use crate::board::{Color, Piece};
use std::sync::OnceLock;

static ZOBRIST_TABLE: OnceLock<ZobristTable> = OnceLock::new();

pub struct ZobristTable {
    pieces: [[[u64; 64]; 6]; 2], // [color][piece_type][square]
    black_to_move: u64,
    castling: [u64; 4], // white kingside, white queenside, black kingside, black queenside
    en_passant: [u64; 8], // one for each file
}

impl ZobristTable {
    fn new() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();

        let mut pieces = [[[0u64; 64]; 6]; 2];
        for color_pieces in &mut pieces {
            for piece_squares in color_pieces {
                for square_hash in piece_squares {
                    *square_hash = rng.random::<u64>();
                }
            }
        }

        let black_to_move = rng.random::<u64>();
        let castling = [
            rng.random::<u64>(),
            rng.random::<u64>(),
            rng.random::<u64>(),
            rng.random::<u64>(),
        ];
        let en_passant = [
            rng.random::<u64>(),
            rng.random::<u64>(),
            rng.random::<u64>(),
            rng.random::<u64>(),
            rng.random::<u64>(),
            rng.random::<u64>(),
            rng.random::<u64>(),
            rng.random::<u64>(),
        ];

        Self {
            pieces,
            black_to_move,
            castling,
            en_passant,
        }
    }

    pub fn get() -> &'static ZobristTable {
        ZOBRIST_TABLE.get_or_init(ZobristTable::new)
    }

    fn piece_index(piece: Piece) -> usize {
        match piece {
            Piece::Pawn => 0,
            Piece::Knight => 1,
            Piece::Bishop => 2,
            Piece::Rook => 3,
            Piece::Queen => 4,
            Piece::King => 5,
        }
    }

    fn color_index(color: Color) -> usize {
        match color {
            Color::White => 0,
            Color::Black => 1,
        }
    }

    pub fn piece_hash(&self, piece: Piece, color: Color, square: usize) -> u64 {
        self.pieces[Self::color_index(color)][Self::piece_index(piece)][square]
    }

    pub fn black_to_move_hash(&self) -> u64 {
        self.black_to_move
    }

    pub fn castling_hash(&self, index: usize) -> u64 {
        self.castling[index]
    }

    pub fn en_passant_hash(&self, file: usize) -> u64 {
        self.en_passant[file]
    }
}

pub fn compute_hash(board: &crate::board::Board) -> u64 {
    let table = ZobristTable::get();
    let mut hash = 0u64;

    // Hash all pieces
    for (square, sq) in board.squares.iter().enumerate() {
        if let Some((piece, color)) = sq.0 {
            hash ^= table.piece_hash(piece, color, square);
        }
    }

    // Hash castling rights
    if board.can_castle_white_kingside() {
        hash ^= table.castling_hash(0);
    }
    if board.can_castle_white_queenside() {
        hash ^= table.castling_hash(1);
    }
    if board.can_castle_black_kingside() {
        hash ^= table.castling_hash(2);
    }
    if board.can_castle_black_queenside() {
        hash ^= table.castling_hash(3);
    }

    // Hash side to move
    if board.side_to_move == Color::Black {
        hash ^= table.black_to_move_hash();
    }

    // Hash en passant
    if let Some(ep_square) = board.en_passant_target {
        let file = ep_square % 8;
        hash ^= table.en_passant_hash(file);
    }

    hash
}
