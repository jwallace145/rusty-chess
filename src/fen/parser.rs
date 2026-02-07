use crate::board::{CastlingRights, Color, Piece, castling::CastlingSide};
use std::fmt;

/// Represents a piece on a square with its color
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ColoredPiece {
    pub piece: Piece,
    pub color: Color,
}

/// Error types that can occur when parsing a FEN string
#[derive(Debug, PartialEq, Eq)]
pub enum FENParseError {
    /// FEN string doesn't have enough parts (needs 6)
    InsufficientParts { found: usize },
    /// Invalid character in piece placement
    InvalidPieceChar(char),
    /// Rank doesn't have exactly 8 squares
    InvalidRankLength { rank: usize, squares: usize },
    /// Piece placement doesn't have exactly 8 ranks
    InvalidRankCount(usize),
    /// Invalid active color (must be 'w' or 'b')
    InvalidActiveColor(String),
    /// Invalid castling character
    InvalidCastlingChar(char),
    /// Invalid en passant square format
    InvalidEnPassantSquare(String),
    /// Invalid file in en passant square
    InvalidEnPassantFile(char),
    /// Invalid rank in en passant square
    InvalidEnPassantRank(char),
    /// Halfmove clock is not a valid number
    InvalidHalfmoveClock(String),
    /// Fullmove number is not a valid number
    InvalidFullmoveNumber(String),
}

impl fmt::Display for FENParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientParts { found } => {
                write!(f, "FEN requires 6 parts, found {}", found)
            }
            Self::InvalidPieceChar(c) => {
                write!(f, "Invalid piece character: '{}'", c)
            }
            Self::InvalidRankLength { rank, squares } => {
                write!(f, "Rank {} has {} squares, expected 8", rank + 1, squares)
            }
            Self::InvalidRankCount(count) => {
                write!(f, "FEN has {} ranks, expected 8", count)
            }
            Self::InvalidActiveColor(s) => {
                write!(f, "Invalid active color '{}', expected 'w' or 'b'", s)
            }
            Self::InvalidCastlingChar(c) => {
                write!(f, "Invalid castling character: '{}'", c)
            }
            Self::InvalidEnPassantSquare(s) => {
                write!(f, "Invalid en passant square: '{}'", s)
            }
            Self::InvalidEnPassantFile(c) => {
                write!(f, "Invalid en passant file: '{}', expected 'a'-'h'", c)
            }
            Self::InvalidEnPassantRank(c) => {
                write!(f, "Invalid en passant rank: '{}', expected '3' or '6'", c)
            }
            Self::InvalidHalfmoveClock(s) => {
                write!(f, "Invalid halfmove clock: '{}'", s)
            }
            Self::InvalidFullmoveNumber(s) => {
                write!(f, "Invalid fullmove number: '{}'", s)
            }
        }
    }
}

impl std::error::Error for FENParseError {}

/// A fully parsed FEN representation containing all chess position details
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedFEN {
    /// 8x8 board representation, indexed as board[rank][file]
    /// rank 0 = rank 1 (white's back rank), rank 7 = rank 8 (black's back rank)
    /// file 0 = a-file, file 7 = h-file
    pub board: [[Option<ColoredPiece>; 8]; 8],

    /// Which color moves next
    pub active_color: Color,

    /// Castling availability for both sides
    pub castling_rights: CastlingRights,

    /// En passant target square as (file, rank) where file: 0-7 (a-h), rank: 0-7 (1-8)
    /// None if no en passant is possible
    pub en_passant_square: Option<(u8, u8)>,

    /// Number of halfmoves since the last capture or pawn advance (for 50-move rule)
    pub halfmove_clock: u8,

    /// The fullmove number, starting at 1 and incremented after Black's move
    pub fullmove_number: u16,
}

impl ParsedFEN {
    /// Returns the en passant square as a square index (0-63) if available
    /// Square index: rank * 8 + file (a1=0, h1=7, a8=56, h8=63)
    pub fn en_passant_square_index(&self) -> Option<u8> {
        self.en_passant_square.map(|(file, rank)| rank * 8 + file)
    }

    /// Returns the en passant square in algebraic notation (e.g., "e3") if available
    pub fn en_passant_square_algebraic(&self) -> Option<String> {
        self.en_passant_square.map(|(file, rank)| {
            let file_char = (b'a' + file) as char;
            let rank_char = (b'1' + rank) as char;
            format!("{}{}", file_char, rank_char)
        })
    }

    /// Get the piece at a specific square using algebraic notation (e.g., "e4")
    pub fn piece_at(&self, square: &str) -> Option<ColoredPiece> {
        if square.len() != 2 {
            return None;
        }
        let mut chars = square.chars();
        let file = chars.next()?.to_ascii_lowercase();
        let rank = chars.next()?;

        if !('a'..='h').contains(&file) || !('1'..='8').contains(&rank) {
            return None;
        }

        let file_idx = (file as u8 - b'a') as usize;
        let rank_idx = (rank as u8 - b'1') as usize;

        self.board[rank_idx][file_idx]
    }

    /// Get the piece at a specific square index (0-63)
    /// Square index: rank * 8 + file (a1=0, h1=7, a8=56, h8=63)
    pub fn piece_at_index(&self, index: u8) -> Option<ColoredPiece> {
        if index >= 64 {
            return None;
        }
        let rank = (index / 8) as usize;
        let file = (index % 8) as usize;
        self.board[rank][file]
    }

    /// Check if a specific castling right is available
    pub fn can_castle(&self, color: Color, side: CastlingSide) -> bool {
        self.castling_rights.has(color, side)
    }

    /// Convert back to a FEN string
    pub fn to_fen_string(&self) -> String {
        let mut fen = String::new();

        // Piece placement (from rank 8 down to rank 1)
        for rank in (0..8).rev() {
            let mut empty_count = 0;
            for file in 0..8 {
                match self.board[rank][file] {
                    Some(cp) => {
                        if empty_count > 0 {
                            fen.push_str(&empty_count.to_string());
                            empty_count = 0;
                        }
                        let c = match (cp.color, cp.piece) {
                            (Color::White, Piece::Pawn) => 'P',
                            (Color::White, Piece::Knight) => 'N',
                            (Color::White, Piece::Bishop) => 'B',
                            (Color::White, Piece::Rook) => 'R',
                            (Color::White, Piece::Queen) => 'Q',
                            (Color::White, Piece::King) => 'K',
                            (Color::Black, Piece::Pawn) => 'p',
                            (Color::Black, Piece::Knight) => 'n',
                            (Color::Black, Piece::Bishop) => 'b',
                            (Color::Black, Piece::Rook) => 'r',
                            (Color::Black, Piece::Queen) => 'q',
                            (Color::Black, Piece::King) => 'k',
                        };
                        fen.push(c);
                    }
                    None => {
                        empty_count += 1;
                    }
                }
            }
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        // Active color
        fen.push(' ');
        fen.push(match self.active_color {
            Color::White => 'w',
            Color::Black => 'b',
        });

        // Castling rights
        fen.push(' ');
        let mut castling = String::new();
        if self
            .castling_rights
            .has(Color::White, CastlingSide::KingSide)
        {
            castling.push('K');
        }
        if self
            .castling_rights
            .has(Color::White, CastlingSide::QueenSide)
        {
            castling.push('Q');
        }
        if self
            .castling_rights
            .has(Color::Black, CastlingSide::KingSide)
        {
            castling.push('k');
        }
        if self
            .castling_rights
            .has(Color::Black, CastlingSide::QueenSide)
        {
            castling.push('q');
        }
        if castling.is_empty() {
            fen.push('-');
        } else {
            fen.push_str(&castling);
        }

        // En passant
        fen.push(' ');
        match self.en_passant_square_algebraic() {
            Some(sq) => fen.push_str(&sq),
            None => fen.push('-'),
        }

        // Halfmove clock
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());

        // Fullmove number
        fen.push(' ');
        fen.push_str(&self.fullmove_number.to_string());

        fen
    }
}

pub struct FENParser;

impl FENParser {
    /// Parse a FEN string into a ParsedFEN struct
    ///
    /// # Arguments
    /// * `fen` - A FEN string with 6 space-separated parts:
    ///   1. Piece placement (e.g., "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR")
    ///   2. Active color ('w' or 'b')
    ///   3. Castling availability (e.g., "KQkq" or "-")
    ///   4. En passant target square (e.g., "e3" or "-")
    ///   5. Halfmove clock (e.g., "0")
    ///   6. Fullmove number (e.g., "1")
    ///
    /// # Returns
    /// * `Ok(ParsedFEN)` - Successfully parsed FEN
    /// * `Err(FENParseError)` - Parsing failed with specific error
    ///
    /// # Example
    /// ```
    /// use rusty_chess::fen::FENParser;
    ///
    /// let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    /// let parsed = FENParser::parse(fen).unwrap();
    /// ```
    pub fn parse(fen: &str) -> Result<ParsedFEN, FENParseError> {
        let parts: Vec<&str> = fen.split_whitespace().collect();

        if parts.len() < 6 {
            return Err(FENParseError::InsufficientParts { found: parts.len() });
        }

        let board = Self::parse_piece_placement(parts[0])?;
        let active_color = Self::parse_active_color(parts[1])?;
        let castling_rights = Self::parse_castling(parts[2])?;
        let en_passant_square = Self::parse_en_passant(parts[3])?;
        let halfmove_clock = Self::parse_halfmove_clock(parts[4])?;
        let fullmove_number = Self::parse_fullmove_number(parts[5])?;

        Ok(ParsedFEN {
            board,
            active_color,
            castling_rights,
            en_passant_square,
            halfmove_clock,
            fullmove_number,
        })
    }

    fn parse_piece_placement(
        placement: &str,
    ) -> Result<[[Option<ColoredPiece>; 8]; 8], FENParseError> {
        let mut board = [[None; 8]; 8];
        let ranks: Vec<&str> = placement.split('/').collect();

        if ranks.len() != 8 {
            return Err(FENParseError::InvalidRankCount(ranks.len()));
        }

        for (rank_idx, rank_str) in ranks.iter().enumerate() {
            let rank = 7 - rank_idx; // FEN starts from rank 8 (index 7)
            let mut file = 0;

            for ch in rank_str.chars() {
                if let Some(digit) = ch.to_digit(10) {
                    file += digit as usize;
                } else {
                    let colored_piece = match ch {
                        'P' => ColoredPiece {
                            piece: Piece::Pawn,
                            color: Color::White,
                        },
                        'N' => ColoredPiece {
                            piece: Piece::Knight,
                            color: Color::White,
                        },
                        'B' => ColoredPiece {
                            piece: Piece::Bishop,
                            color: Color::White,
                        },
                        'R' => ColoredPiece {
                            piece: Piece::Rook,
                            color: Color::White,
                        },
                        'Q' => ColoredPiece {
                            piece: Piece::Queen,
                            color: Color::White,
                        },
                        'K' => ColoredPiece {
                            piece: Piece::King,
                            color: Color::White,
                        },
                        'p' => ColoredPiece {
                            piece: Piece::Pawn,
                            color: Color::Black,
                        },
                        'n' => ColoredPiece {
                            piece: Piece::Knight,
                            color: Color::Black,
                        },
                        'b' => ColoredPiece {
                            piece: Piece::Bishop,
                            color: Color::Black,
                        },
                        'r' => ColoredPiece {
                            piece: Piece::Rook,
                            color: Color::Black,
                        },
                        'q' => ColoredPiece {
                            piece: Piece::Queen,
                            color: Color::Black,
                        },
                        'k' => ColoredPiece {
                            piece: Piece::King,
                            color: Color::Black,
                        },
                        _ => return Err(FENParseError::InvalidPieceChar(ch)),
                    };

                    if file >= 8 {
                        return Err(FENParseError::InvalidRankLength {
                            rank: rank_idx,
                            squares: file + 1,
                        });
                    }

                    board[rank][file] = Some(colored_piece);
                    file += 1;
                }
            }

            if file != 8 {
                return Err(FENParseError::InvalidRankLength {
                    rank: rank_idx,
                    squares: file,
                });
            }
        }

        Ok(board)
    }

    fn parse_active_color(color: &str) -> Result<Color, FENParseError> {
        match color {
            "w" => Ok(Color::White),
            "b" => Ok(Color::Black),
            _ => Err(FENParseError::InvalidActiveColor(color.to_string())),
        }
    }

    fn parse_castling(castling: &str) -> Result<CastlingRights, FENParseError> {
        let mut rights = CastlingRights::empty();

        if castling == "-" {
            return Ok(rights);
        }

        for ch in castling.chars() {
            match ch {
                'K' => rights.add(Color::White, CastlingSide::KingSide),
                'Q' => rights.add(Color::White, CastlingSide::QueenSide),
                'k' => rights.add(Color::Black, CastlingSide::KingSide),
                'q' => rights.add(Color::Black, CastlingSide::QueenSide),
                _ => return Err(FENParseError::InvalidCastlingChar(ch)),
            }
        }

        Ok(rights)
    }

    fn parse_en_passant(ep: &str) -> Result<Option<(u8, u8)>, FENParseError> {
        if ep == "-" {
            return Ok(None);
        }

        if ep.len() != 2 {
            return Err(FENParseError::InvalidEnPassantSquare(ep.to_string()));
        }

        let mut chars = ep.chars();
        let file_char = chars.next().unwrap();
        let rank_char = chars.next().unwrap();

        let file = match file_char {
            'a'..='h' => (file_char as u8) - b'a',
            _ => return Err(FENParseError::InvalidEnPassantFile(file_char)),
        };

        let rank = match rank_char {
            '3' | '6' => (rank_char as u8) - b'1',
            '1'..='8' => {
                // Allow any rank for flexibility, but typically only 3 or 6 are valid
                (rank_char as u8) - b'1'
            }
            _ => return Err(FENParseError::InvalidEnPassantRank(rank_char)),
        };

        Ok(Some((file, rank)))
    }

    fn parse_halfmove_clock(hmc: &str) -> Result<u8, FENParseError> {
        hmc.parse()
            .map_err(|_| FENParseError::InvalidHalfmoveClock(hmc.to_string()))
    }

    fn parse_fullmove_number(fmn: &str) -> Result<u16, FENParseError> {
        fmn.parse()
            .map_err(|_| FENParseError::InvalidFullmoveNumber(fmn.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_starting_position() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let parsed = FENParser::parse(fen).unwrap();

        assert_eq!(parsed.active_color, Color::White);
        assert_eq!(parsed.halfmove_clock, 0);
        assert_eq!(parsed.fullmove_number, 1);
        assert!(parsed.en_passant_square.is_none());

        // Check castling rights
        assert!(parsed.can_castle(Color::White, CastlingSide::KingSide));
        assert!(parsed.can_castle(Color::White, CastlingSide::QueenSide));
        assert!(parsed.can_castle(Color::Black, CastlingSide::KingSide));
        assert!(parsed.can_castle(Color::Black, CastlingSide::QueenSide));

        // Check some pieces
        assert_eq!(
            parsed.piece_at("e1"),
            Some(ColoredPiece {
                piece: Piece::King,
                color: Color::White
            })
        );
        assert_eq!(
            parsed.piece_at("e8"),
            Some(ColoredPiece {
                piece: Piece::King,
                color: Color::Black
            })
        );
        assert_eq!(
            parsed.piece_at("a1"),
            Some(ColoredPiece {
                piece: Piece::Rook,
                color: Color::White
            })
        );
        assert_eq!(parsed.piece_at("e4"), None);
    }

    #[test]
    fn test_parse_with_en_passant() {
        let fen = "rnbqkbnr/pppp1ppp/8/4pP2/8/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 3";
        let parsed = FENParser::parse(fen).unwrap();

        assert_eq!(parsed.en_passant_square, Some((4, 5))); // e6
        assert_eq!(parsed.en_passant_square_algebraic(), Some("e6".to_string()));
        assert_eq!(parsed.en_passant_square_index(), Some(44)); // 5 * 8 + 4
    }

    #[test]
    fn test_parse_no_castling() {
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w - - 0 1";
        let parsed = FENParser::parse(fen).unwrap();

        assert!(!parsed.can_castle(Color::White, CastlingSide::KingSide));
        assert!(!parsed.can_castle(Color::White, CastlingSide::QueenSide));
        assert!(!parsed.can_castle(Color::Black, CastlingSide::KingSide));
        assert!(!parsed.can_castle(Color::Black, CastlingSide::QueenSide));
    }

    #[test]
    fn test_parse_partial_castling() {
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b Kq - 5 10";
        let parsed = FENParser::parse(fen).unwrap();

        assert_eq!(parsed.active_color, Color::Black);
        assert!(parsed.can_castle(Color::White, CastlingSide::KingSide));
        assert!(!parsed.can_castle(Color::White, CastlingSide::QueenSide));
        assert!(!parsed.can_castle(Color::Black, CastlingSide::KingSide));
        assert!(parsed.can_castle(Color::Black, CastlingSide::QueenSide));
        assert_eq!(parsed.halfmove_clock, 5);
        assert_eq!(parsed.fullmove_number, 10);
    }

    #[test]
    fn test_to_fen_string_roundtrip() {
        let original = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let parsed = FENParser::parse(original).unwrap();
        let regenerated = parsed.to_fen_string();
        assert_eq!(original, regenerated);
    }

    #[test]
    fn test_to_fen_string_with_en_passant() {
        let original = "rnbqkbnr/pppp1ppp/8/4pP2/8/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 3";
        let parsed = FENParser::parse(original).unwrap();
        let regenerated = parsed.to_fen_string();
        assert_eq!(original, regenerated);
    }

    #[test]
    fn test_piece_at_index() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let parsed = FENParser::parse(fen).unwrap();

        // a1 = 0, should be white rook
        assert_eq!(
            parsed.piece_at_index(0),
            Some(ColoredPiece {
                piece: Piece::Rook,
                color: Color::White
            })
        );

        // e1 = 4, should be white king
        assert_eq!(
            parsed.piece_at_index(4),
            Some(ColoredPiece {
                piece: Piece::King,
                color: Color::White
            })
        );

        // e4 = 28, should be empty
        assert_eq!(parsed.piece_at_index(28), None);
    }

    #[test]
    fn test_error_insufficient_parts() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w";
        let result = FENParser::parse(fen);
        assert!(matches!(
            result,
            Err(FENParseError::InsufficientParts { found: 2 })
        ));
    }

    #[test]
    fn test_error_invalid_piece_char() {
        let fen = "rnbqkbxr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let result = FENParser::parse(fen);
        assert!(matches!(result, Err(FENParseError::InvalidPieceChar('x'))));
    }

    #[test]
    fn test_error_invalid_active_color() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1";
        let result = FENParser::parse(fen);
        assert!(matches!(result, Err(FENParseError::InvalidActiveColor(_))));
    }
}
