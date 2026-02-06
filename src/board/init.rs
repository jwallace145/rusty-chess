use crate::{
    board::{Board, CastlingRights, Color, Piece},
    fen::{FENParser, ParsedFEN},
    search::compute_hash_board,
};

impl Board {
    /// Create an empty Chess board
    pub fn new_empty() -> Self {
        Self {
            pieces: [[0u64; 6]; 2],
            occ: [0u64; 2],
            occ_all: 0u64,
            side_to_move: Color::White,
            castling: CastlingRights::empty(),
            en_passant: 64,
            halfmove_clock: 0,
            king_sq: [64; 2],
            hash: 0u64,
        }
    }

    /// Create a Chess board in standard starting position
    pub fn startpos() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    /// Create a Chess board from FEN (Forsyth–Edwards Notation)
    pub fn from_fen(fen: &str) -> Self {
        let parsed: ParsedFEN = FENParser::parse(fen).expect("Invalid FEN string");

        let mut board: Board = Self::new_empty();

        // Set pieces from parsed board
        for rank in 0..8 {
            for file in 0..8 {
                if let Some(colored_piece) = parsed.board[rank][file] {
                    let sq: usize = rank * 8 + file;
                    let color: Color = colored_piece.color;
                    let piece: Piece = colored_piece.piece;

                    board.pieces[color as usize][piece as usize] |= 1u64 << sq;
                    board.occ[color as usize] |= 1u64 << sq;

                    if piece == Piece::King {
                        board.king_sq[color as usize] = sq as u8;
                    }
                }
            }
        }

        // Update combined occupancy
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        // Set game state from parsed FEN
        board.side_to_move = parsed.active_color;
        board.castling = parsed.castling_rights;
        board.en_passant = parsed.en_passant_square_index().unwrap_or(64);
        board.halfmove_clock = parsed.halfmove_clock;

        // Compute the Zobrist hash
        board.hash = compute_hash_board(&board);

        board
    }

    /// Convert the current board state to FEN notation
    pub fn to_fen(&self) -> String {
        crate::fen::board_fen(self)
    }
}
