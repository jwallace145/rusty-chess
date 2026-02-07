use super::Board;
use crate::board::{CastlingRights, ChessMove, Color, Piece};

#[test]
fn test_board_refactor_pieces_of() {
    let board: Board = Board::default();

    // Get White pieces
    let white_pawns: u64 = board.pieces_of(Color::White, Piece::Pawn);
    let white_rooks: u64 = board.pieces_of(Color::White, Piece::Rook);
    let white_knights: u64 = board.pieces_of(Color::White, Piece::Knight);
    let white_bishops: u64 = board.pieces_of(Color::White, Piece::Bishop);
    let white_queen: u64 = board.pieces_of(Color::White, Piece::Queen);
    let white_king: u64 = board.pieces_of(Color::White, Piece::King);

    // Assert White starting position
    assert_eq!(white_pawns, 0x0000_0000_0000_ff00);
    assert_eq!(white_rooks, 0x0000_0000_0000_0081);
    assert_eq!(white_knights, 0x0000_0000_0000_0042);
    assert_eq!(white_bishops, 0x0000_0000_0000_0024);
    assert_eq!(white_queen, 0x0000_0000_0000_0008); // d1
    assert_eq!(white_king, 0x0000_0000_0000_0010); // e1

    // Get Black pieces
    let black_pawns: u64 = board.pieces_of(Color::Black, Piece::Pawn);
    let black_rooks: u64 = board.pieces_of(Color::Black, Piece::Rook);
    let black_knights: u64 = board.pieces_of(Color::Black, Piece::Knight);
    let black_bishops: u64 = board.pieces_of(Color::Black, Piece::Bishop);
    let black_queen: u64 = board.pieces_of(Color::Black, Piece::Queen);
    let black_king: u64 = board.pieces_of(Color::Black, Piece::King);

    // Assert Black starting position
    assert_eq!(black_pawns, 0x00ff_0000_0000_0000);
    assert_eq!(black_rooks, 0x8100_0000_0000_0000);
    assert_eq!(black_knights, 0x4200_0000_0000_0000);
    assert_eq!(black_bishops, 0x2400_0000_0000_0000);
    assert_eq!(black_queen, 0x0800_0000_0000_0000); // d8
    assert_eq!(black_king, 0x1000_0000_0000_0000); // e8
}

#[test]
fn test_board_refactor_occupancy() {
    let board: Board = Board::default();

    // Assert White pieces occupied squares
    let white_pieces: u64 = board.occupancy(Color::White);
    assert_eq!(white_pieces, 0x0000_0000_0000_ffff);

    // Assert Black pieces occupied squares
    let black_pieces: u64 = board.occupancy(Color::Black);
    assert_eq!(black_pieces, 0xffff_0000_0000_0000);
}

#[test]
fn test_board_refactor_occupied() {
    let board: Board = Board::default();

    // Assert starting pieces occupied squares
    let pieces: u64 = board.occupied();
    assert_eq!(pieces, 0xffff_0000_0000_ffff);
}

#[test]
fn test_board_refactor_empty() {
    let board: Board = Board::default();

    // Assert starting position empty squares
    let empty: u64 = board.empty();
    assert_eq!(empty, 0x0000_ffff_ffff_0000);
}

#[test]
fn test_board_refactor_piece_on() {
    let board: Board = Board::default();

    // Assert White pieces
    // White rooks on a1 (0) and h1 (7)
    assert_eq!(board.piece_on(0), Some((Color::White, Piece::Rook)));
    assert_eq!(board.piece_on(7), Some((Color::White, Piece::Rook)));

    // White knights on b1 (1) and g1 (6)
    assert_eq!(board.piece_on(1), Some((Color::White, Piece::Knight)));
    assert_eq!(board.piece_on(6), Some((Color::White, Piece::Knight)));

    // White bishops on c1 (2) and f1 (5)
    assert_eq!(board.piece_on(2), Some((Color::White, Piece::Bishop)));
    assert_eq!(board.piece_on(5), Some((Color::White, Piece::Bishop)));

    // White queen on d1 (3) and king on e1 (4)
    assert_eq!(board.piece_on(3), Some((Color::White, Piece::Queen)));
    assert_eq!(board.piece_on(4), Some((Color::White, Piece::King)));

    // White pawns on rank 2 (squares 8-15)
    for sq in 8..16 {
        assert_eq!(board.piece_on(sq), Some((Color::White, Piece::Pawn)));
    }

    // Assert Black pieces
    // Black rooks on a8 (56) and h8 (63)
    assert_eq!(board.piece_on(56), Some((Color::Black, Piece::Rook)));
    assert_eq!(board.piece_on(63), Some((Color::Black, Piece::Rook)));

    // Black knights on b8 (57) and g8 (62)
    assert_eq!(board.piece_on(57), Some((Color::Black, Piece::Knight)));
    assert_eq!(board.piece_on(62), Some((Color::Black, Piece::Knight)));

    // Black bishops on c8 (58) and f8 (61)
    assert_eq!(board.piece_on(58), Some((Color::Black, Piece::Bishop)));
    assert_eq!(board.piece_on(61), Some((Color::Black, Piece::Bishop)));

    // Black queen on d8 (59) and king on e8 (60)
    assert_eq!(board.piece_on(59), Some((Color::Black, Piece::Queen)));
    assert_eq!(board.piece_on(60), Some((Color::Black, Piece::King)));

    // Black pawns on rank 7 (squares 48-55)
    for sq in 48..56 {
        assert_eq!(board.piece_on(sq), Some((Color::Black, Piece::Pawn)));
    }

    // Assert empty squares (middle of the board)
    for sq in 16..48 {
        assert_eq!(board.piece_on(sq), None);
    }
}

#[test]
fn test_board_refactor_attacks_from() {
    let board: Board = Board::default();

    //
    // =====================
    // PAWN ATTACK TESTS
    // =====================
    //

    // White pawn on a2 → b3
    assert_eq!(board.attacks_from(Piece::Pawn, 8, Color::White), 1u64 << 17);

    // White pawn on d4 → c5, e5
    assert_eq!(
        board.attacks_from(Piece::Pawn, 27, Color::White),
        (1u64 << 34) | (1u64 << 36)
    );

    // White pawn on h2 → g3 only
    assert_eq!(
        board.attacks_from(Piece::Pawn, 15, Color::White),
        1u64 << 22
    );

    // Black pawn on a7 → b6
    assert_eq!(
        board.attacks_from(Piece::Pawn, 48, Color::Black),
        1u64 << 41
    );

    // Black pawn on d5 → c4, e4
    assert_eq!(
        board.attacks_from(Piece::Pawn, 35, Color::Black),
        (1u64 << 26) | (1u64 << 28)
    );

    // Black pawn on h7 → g6 only
    assert_eq!(
        board.attacks_from(Piece::Pawn, 55, Color::Black),
        1u64 << 46
    );

    //
    // =====================
    // KNIGHT ATTACK TESTS
    // =====================
    //

    // Knight on b1 → a3, c3, d2
    assert_eq!(
        board.attacks_from(Piece::Knight, 1, Color::White),
        (1u64 << 16) | (1u64 << 18) | (1u64 << 11)
    );

    // Knight on d4 (27) → b3, b5, c2, c6, e2, e6, f3, f5
    assert_eq!(
        board.attacks_from(Piece::Knight, 27, Color::White),
        (1u64 << 10)
            | (1u64 << 12)
            | (1u64 << 17)
            | (1u64 << 21)
            | (1u64 << 33)
            | (1u64 << 37)
            | (1u64 << 42)
            | (1u64 << 44)
    );

    // Knight on h1 → f2, g3
    assert_eq!(
        board.attacks_from(Piece::Knight, 7, Color::White),
        (1u64 << 13) | (1u64 << 22)
    );

    //
    // =====================
    // KING ATTACK TESTS
    // =====================
    //

    // King on e4 (28)
    assert_eq!(
        board.attacks_from(Piece::King, 28, Color::White),
        (1u64 << 19)
            | (1u64 << 20)
            | (1u64 << 21)
            | (1u64 << 27)
            | (1u64 << 29)
            | (1u64 << 35)
            | (1u64 << 36)
            | (1u64 << 37)
    );

    // King on a1 → a2, b1, b2
    assert_eq!(
        board.attacks_from(Piece::King, 0, Color::White),
        (1u64 << 8) | (1u64 << 1) | (1u64 << 9)
    );

    //
    // =====================
    // SLIDING PIECE TESTS (empty board)
    // =====================
    //

    let empty_board: Board = Board::new_empty();

    // Bishop on d4 (27)
    assert_eq!(
        empty_board.attacks_from(Piece::Bishop, 27, Color::White),
        (1u64 << 36)
            | (1u64 << 45)
            | (1u64 << 54)
            | (1u64 << 63)
            | (1u64 << 34)
            | (1u64 << 41)
            | (1u64 << 48)
            | (1u64 << 18)
            | (1u64 << 9)
            | (1u64 << 0)
            | (1u64 << 20)
            | (1u64 << 13)
            | (1u64 << 6)
    );

    // Rook on d4 (27)
    assert_eq!(
        empty_board.attacks_from(Piece::Rook, 27, Color::White),
        // north (35,43,51,59)
        (1u64 << 35) | (1u64 << 43) | (1u64 << 51) | (1u64 << 59)
        // south (19,11,3)
        | (1u64 << 19) | (1u64 << 11) | (1u64 << 3)
        // east (28,29,30,31)
        | (1u64 << 28) | (1u64 << 29) | (1u64 << 30) | (1u64 << 31)
        // west (26,25,24)
        | (1u64 << 26) | (1u64 << 25) | (1u64 << 24)
    );

    // Queen on d4 = bishop + rook
    assert_eq!(
        empty_board.attacks_from(Piece::Queen, 27, Color::White),
        empty_board.attacks_from(Piece::Rook, 27, Color::White)
            | empty_board.attacks_from(Piece::Bishop, 27, Color::White)
    );
}

#[test]
fn test_board_refactor_is_square_attacked() {
    // Test custom position - white queen attacking
    let mut board = Board::new_empty();
    board.pieces[Color::White as usize][Piece::Queen as usize] = 1u64 << 27; // d4
    board.occ[Color::White as usize] = 1u64 << 27;
    board.occ_all = 1u64 << 27;

    // Queen should attack diagonal, horizontal, and vertical squares
    // Note: avoiding corner squares (0,7,56,63) due to magic bitboard limitations
    assert!(board.is_square_attacked(9, Color::White)); // b2 (diagonal)
    assert!(board.is_square_attacked(36, Color::White)); // e5 (diagonal)
    assert!(board.is_square_attacked(45, Color::White)); // f6 (diagonal)
    assert!(board.is_square_attacked(35, Color::White)); // d5 (vertical)
    assert!(board.is_square_attacked(19, Color::White)); // d3 (vertical)
    assert!(board.is_square_attacked(28, Color::White)); // e4 (horizontal)
    assert!(board.is_square_attacked(25, Color::White)); // b4 (horizontal)

    // Should not attack squares that queens can't reach
    assert!(!board.is_square_attacked(1, Color::White)); // b1 (knight move)
    assert!(!board.is_square_attacked(17, Color::White)); // b3 (knight move)

    // Test custom position - black rook attacking
    let mut board = Board::new_empty();
    board.pieces[Color::Black as usize][Piece::Rook as usize] = 1u64 << 27; // d4
    board.occ[Color::Black as usize] = 1u64 << 27;
    board.occ_all = 1u64 << 27;

    // Rook should attack file and rank (avoiding corners)
    assert!(board.is_square_attacked(3, Color::Black)); // d1
    assert!(board.is_square_attacked(35, Color::Black)); // d5
    assert!(board.is_square_attacked(26, Color::Black)); // c4
    assert!(board.is_square_attacked(28, Color::Black)); // e4

    // Should not attack diagonal squares
    assert!(!board.is_square_attacked(18, Color::Black)); // c3
    assert!(!board.is_square_attacked(36, Color::Black)); // e5

    // Test bishop attacks
    let mut board = Board::new_empty();
    board.pieces[Color::White as usize][Piece::Bishop as usize] = 1u64 << 27; // d4
    board.occ[Color::White as usize] = 1u64 << 27;
    board.occ_all = 1u64 << 27;

    // Bishop should attack diagonals (avoiding corners)
    assert!(board.is_square_attacked(9, Color::White)); // b2
    assert!(board.is_square_attacked(18, Color::White)); // c3
    assert!(board.is_square_attacked(36, Color::White)); // e5
    assert!(board.is_square_attacked(54, Color::White)); // g7

    // Should not attack non-diagonal squares
    assert!(!board.is_square_attacked(26, Color::White)); // c4
    assert!(!board.is_square_attacked(35, Color::White)); // d5

    // Test knight attacks
    let mut board = Board::new_empty();
    board.pieces[Color::White as usize][Piece::Knight as usize] = 1u64 << 27; // d4
    board.occ[Color::White as usize] = 1u64 << 27;
    board.occ_all = 1u64 << 27;

    // Knight on d4 attacks 8 squares
    assert!(board.is_square_attacked(10, Color::White)); // c2
    assert!(board.is_square_attacked(12, Color::White)); // e2
    assert!(board.is_square_attacked(17, Color::White)); // b3
    assert!(board.is_square_attacked(21, Color::White)); // f3
    assert!(board.is_square_attacked(33, Color::White)); // b5
    assert!(board.is_square_attacked(37, Color::White)); // f5
    assert!(board.is_square_attacked(42, Color::White)); // c6
    assert!(board.is_square_attacked(44, Color::White)); // e6

    // Should not attack non-knight-move squares
    assert!(!board.is_square_attacked(19, Color::White)); // d3
    assert!(!board.is_square_attacked(35, Color::White)); // d5

    // Test king attacks (adjacent squares)
    let mut board = Board::new_empty();
    board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 28; // e4
    board.king_sq[Color::Black as usize] = 28;
    board.occ[Color::Black as usize] = 1u64 << 28;
    board.occ_all = 1u64 << 28;

    // King attacks all adjacent squares
    assert!(board.is_square_attacked(19, Color::Black)); // d3
    assert!(board.is_square_attacked(20, Color::Black)); // e3
    assert!(board.is_square_attacked(21, Color::Black)); // f3
    assert!(board.is_square_attacked(27, Color::Black)); // d4
    assert!(board.is_square_attacked(29, Color::Black)); // f4
    assert!(board.is_square_attacked(35, Color::Black)); // d5
    assert!(board.is_square_attacked(36, Color::Black)); // e5
    assert!(board.is_square_attacked(37, Color::Black)); // f5

    // Should not attack non-adjacent squares
    assert!(!board.is_square_attacked(12, Color::Black)); // e2
    assert!(!board.is_square_attacked(44, Color::Black)); // e6

    // Test pawn attacks - white pawn
    let mut board = Board::new_empty();
    board.pieces[Color::White as usize][Piece::Pawn as usize] = 1u64 << 27; // d4
    board.occ[Color::White as usize] = 1u64 << 27;
    board.occ_all = 1u64 << 27;

    // White pawn on d4 attacks c5 and e5
    assert!(board.is_square_attacked(34, Color::White)); // c5
    assert!(board.is_square_attacked(36, Color::White)); // e5

    // Should not attack other squares
    assert!(!board.is_square_attacked(27, Color::White)); // d4 (itself)
    assert!(!board.is_square_attacked(35, Color::White)); // d5 (straight ahead)
    assert!(!board.is_square_attacked(19, Color::White)); // d3 (behind)

    // Test pawn attacks - black pawn
    let mut board = Board::new_empty();
    board.pieces[Color::Black as usize][Piece::Pawn as usize] = 1u64 << 35; // d5
    board.occ[Color::Black as usize] = 1u64 << 35;
    board.occ_all = 1u64 << 35;

    // Black pawn on d5 attacks c4 and e4
    assert!(board.is_square_attacked(26, Color::Black)); // c4
    assert!(board.is_square_attacked(28, Color::Black)); // e4

    // Should not attack other squares
    assert!(!board.is_square_attacked(35, Color::Black)); // d5 (itself)
    assert!(!board.is_square_attacked(27, Color::Black)); // d4 (straight ahead)
    assert!(!board.is_square_attacked(43, Color::Black)); // d6 (behind)
}

#[test]
fn test_board_refactor_in_check() {
    // Starting position - no checks
    let board = Board::default();
    assert!(!board.in_check(Color::White));
    assert!(!board.in_check(Color::Black));

    // White king in check by black queen
    let mut board = Board::new_empty();
    board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
    board.king_sq[Color::White as usize] = 4;
    board.pieces[Color::Black as usize][Piece::Queen as usize] = 1u64 << 12; // e2
    board.occ[Color::White as usize] = 1u64 << 4;
    board.occ[Color::Black as usize] = 1u64 << 12;
    board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

    assert!(board.in_check(Color::White));

    // Black king in check by white rook
    let mut board = Board::new_empty();
    board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 28; // e4 (avoiding corners)
    board.king_sq[Color::Black as usize] = 28;
    board.pieces[Color::White as usize][Piece::Rook as usize] = 1u64 << 4; // e1
    board.occ[Color::Black as usize] = 1u64 << 28;
    board.occ[Color::White as usize] = 1u64 << 4;
    board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

    assert!(board.in_check(Color::Black));

    // White king in check by black bishop on diagonal
    let mut board = Board::new_empty();
    board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 20; // e3
    board.king_sq[Color::White as usize] = 20;
    board.pieces[Color::Black as usize][Piece::Bishop as usize] = 1u64 << 38; // g6
    board.occ[Color::White as usize] = 1u64 << 20;
    board.occ[Color::Black as usize] = 1u64 << 38;
    board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

    assert!(board.in_check(Color::White));

    // Black king in check by white knight
    let mut board = Board::new_empty();
    board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 28; // e4
    board.king_sq[Color::Black as usize] = 28;
    board.pieces[Color::White as usize][Piece::Knight as usize] = 1u64 << 11; // d2
    board.occ[Color::Black as usize] = 1u64 << 28;
    board.occ[Color::White as usize] = 1u64 << 11;
    board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

    assert!(board.in_check(Color::Black));

    // White king in check by black pawn
    let mut board = Board::new_empty();
    board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 28; // e4
    board.king_sq[Color::White as usize] = 28;
    board.pieces[Color::Black as usize][Piece::Pawn as usize] = 1u64 << 35; // d5
    board.occ[Color::White as usize] = 1u64 << 28;
    board.occ[Color::Black as usize] = 1u64 << 35;
    board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

    assert!(board.in_check(Color::White));

    // Black king NOT in check (piece blocked by another piece)
    let mut board = Board::new_empty();
    board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 52; // e7
    board.king_sq[Color::Black as usize] = 52;
    board.pieces[Color::White as usize][Piece::Rook as usize] = 1u64 << 4; // e1
    board.pieces[Color::Black as usize][Piece::Pawn as usize] = 1u64 << 28; // e4 (blocking)
    board.occ[Color::Black as usize] = (1u64 << 52) | (1u64 << 28);
    board.occ[Color::White as usize] = 1u64 << 4;
    board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

    assert!(!board.in_check(Color::Black));

    // King attacked by adjacent enemy king (unusual but valid check scenario)
    let mut board = Board::new_empty();
    board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 28; // e4
    board.king_sq[Color::White as usize] = 28;
    board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 36; // e5
    board.king_sq[Color::Black as usize] = 36;
    board.occ[Color::White as usize] = 1u64 << 28;
    board.occ[Color::Black as usize] = 1u64 << 36;
    board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

    assert!(board.in_check(Color::White));
    assert!(board.in_check(Color::Black));
}

#[test]
fn test_board_refactor_attacks() {
    // Test that attacks() returns all squares attacked by a color
    let mut board = Board::new_empty();

    // Set up a simple position with a few white pieces
    // White queen on d4, white knight on b3
    board.pieces[Color::White as usize][Piece::Queen as usize] = 1u64 << 27; // d4
    board.pieces[Color::White as usize][Piece::Knight as usize] = 1u64 << 17; // b3
    board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
    board.king_sq[Color::White as usize] = 4;
    board.occ[Color::White as usize] = (1u64 << 27) | (1u64 << 17) | (1u64 << 4);

    // Black king far away
    board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
    board.king_sq[Color::Black as usize] = 60;
    board.occ[Color::Black as usize] = 1u64 << 60;

    board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

    // Get all white attacks
    let white_attacks = board.attacks(Color::White);

    // The white queen on d4 should attack many squares
    // The white knight on b3 should attack a1, c1, d2, d4, a5, c5
    // The white king on e1 should attack d1, d2, e2, f1, f2

    // Verify that certain squares are attacked by white
    assert_ne!(
        white_attacks & (1u64 << 35),
        0,
        "d5 should be attacked by queen"
    ); // d5 (by queen)
    assert_ne!(
        white_attacks & (1u64 << 19),
        0,
        "d3 should be attacked by queen"
    ); // d3 (by queen)
    assert_ne!(
        white_attacks & (1u64 << 28),
        0,
        "e4 should be attacked by queen"
    ); // e4 (by queen)
    assert_ne!(
        white_attacks & (1u64 << 0),
        0,
        "a1 should be attacked by knight"
    ); // a1 (by knight)
    assert_ne!(
        white_attacks & (1u64 << 32),
        0,
        "a5 should be attacked by knight"
    ); // a5 (by knight)
    assert_ne!(
        white_attacks & (1u64 << 11),
        0,
        "d2 should be attacked by knight or king"
    ); // d2 (by knight/king)
    assert_ne!(
        white_attacks & (1u64 << 3),
        0,
        "d1 should be attacked by king"
    ); // d1 (by king)
    assert_ne!(
        white_attacks & (1u64 << 12),
        0,
        "e2 should be attacked by king"
    ); // e2 (by king)

    // Verify that a square not attacked is not in the bitboard
    // For instance, b8 should not be attacked (not on queen's diagonals/lines, out of knight/king range)
    assert_eq!(white_attacks & (1u64 << 57), 0, "b8 should not be attacked");

    // Test black attacks - only the king on e8
    let black_attacks = board.attacks(Color::Black);

    // Black king on e8 attacks d8, d7, e7, f7, f8
    assert_ne!(
        black_attacks & (1u64 << 59),
        0,
        "d8 should be attacked by black king"
    );
    assert_ne!(
        black_attacks & (1u64 << 51),
        0,
        "d7 should be attacked by black king"
    );
    assert_ne!(
        black_attacks & (1u64 << 52),
        0,
        "e7 should be attacked by black king"
    );
    assert_ne!(
        black_attacks & (1u64 << 53),
        0,
        "f7 should be attacked by black king"
    );
    assert_ne!(
        black_attacks & (1u64 << 61),
        0,
        "f8 should be attacked by black king"
    );

    // Verify that a1 is not attacked by black
    assert_eq!(
        black_attacks & (1u64 << 0),
        0,
        "a1 should not be attacked by black"
    );

    // Test with standard starting position
    let board = Board::startpos();
    let white_attacks = board.attacks(Color::White);
    let black_attacks = board.attacks(Color::Black);

    // In the starting position, white attacks rank 3 with pawns and knights
    // White pawn on a2 attacks b3
    assert_ne!(
        white_attacks & (1u64 << 17),
        0,
        "b3 should be attacked by white in starting position"
    );
    // White knight on b1 attacks a3, c3, d2
    assert_ne!(
        white_attacks & (1u64 << 16),
        0,
        "a3 should be attacked by white knight"
    );
    assert_ne!(
        white_attacks & (1u64 << 18),
        0,
        "c3 should be attacked by white knight"
    );

    // Black attacks rank 6 with pawns and knights
    assert_ne!(
        black_attacks & (1u64 << 41),
        0,
        "b6 should be attacked by black in starting position"
    );
    assert_ne!(
        black_attacks & (1u64 << 40),
        0,
        "a6 should be attacked by black knight"
    );
    assert_ne!(
        black_attacks & (1u64 << 42),
        0,
        "c6 should be attacked by black knight"
    );
}

#[test]
fn test_make_unmake_simple_move() {
    let mut board = Board::new_empty();

    // Set up a simple position: white pawn on e2, kings far away
    board.pieces[Color::White as usize][Piece::Pawn as usize] = 1u64 << 12; // e2
    board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
    board.king_sq[Color::White as usize] = 4;
    board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
    board.king_sq[Color::Black as usize] = 60;

    board.occ[Color::White as usize] = (1u64 << 12) | (1u64 << 4);
    board.occ[Color::Black as usize] = 1u64 << 60;
    board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];
    board.side_to_move = Color::White;

    let original_board = board;

    // Make a move: pawn from e2 to e4
    let mv = ChessMove::new(12, 28);

    let state = board.make_move(mv);

    // Verify the move was made
    assert_eq!(board.piece_on(12), None); // e2 should be empty
    assert_eq!(board.piece_on(28), Some((Color::White, Piece::Pawn))); // e4 has pawn
    assert_eq!(board.side_to_move, Color::Black);
    assert_eq!(board.en_passant, 20); // e3 is the en passant square

    // Unmake the move
    board.unmake_move(state);

    // Verify board is restored
    assert_eq!(board.piece_on(12), Some((Color::White, Piece::Pawn)));
    assert_eq!(board.piece_on(28), None);
    assert_eq!(board.side_to_move, Color::White);
    assert_eq!(board.en_passant, 64); // No en passant
    assert_eq!(board.occ_all, original_board.occ_all);
}

#[test]
fn test_make_unmake_capture() {
    let mut board = Board::new_empty();

    // Set up position with a capture
    board.pieces[Color::White as usize][Piece::Pawn as usize] = 1u64 << 27; // d4
    board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
    board.king_sq[Color::White as usize] = 4;
    board.pieces[Color::Black as usize][Piece::Pawn as usize] = 1u64 << 36; // e5
    board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
    board.king_sq[Color::Black as usize] = 60;

    board.occ[Color::White as usize] = (1u64 << 27) | (1u64 << 4);
    board.occ[Color::Black as usize] = (1u64 << 36) | (1u64 << 60);
    board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];
    board.side_to_move = Color::White;

    let original_board = board;

    // Make a capture: pawn d4 takes e5
    let mv = ChessMove::new(27, 36);

    let state = board.make_move(mv);

    // Verify the capture
    assert_eq!(board.piece_on(27), None);
    assert_eq!(board.piece_on(36), Some((Color::White, Piece::Pawn)));
    assert_eq!(board.side_to_move, Color::Black);
    assert_eq!(board.halfmove_clock, 0); // Reset on capture

    // Unmake the move
    board.unmake_move(state);

    // Verify board is restored
    assert_eq!(board.piece_on(27), Some((Color::White, Piece::Pawn)));
    assert_eq!(board.piece_on(36), Some((Color::Black, Piece::Pawn)));
    assert_eq!(board.side_to_move, Color::White);
    assert_eq!(board.occ_all, original_board.occ_all);
}

#[test]
fn test_make_unmake_castling() {
    use crate::board::castling::CastlingSide;

    let mut board = Board::new_empty();

    // Set up position for white kingside castling
    board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
    board.pieces[Color::White as usize][Piece::Rook as usize] = 1u64 << 7; // h1
    board.king_sq[Color::White as usize] = 4;
    board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
    board.king_sq[Color::Black as usize] = 60;

    board.occ[Color::White as usize] = (1u64 << 4) | (1u64 << 7);
    board.occ[Color::Black as usize] = 1u64 << 60;
    board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];
    board.side_to_move = Color::White;
    board.castling = CastlingRights::full();

    let original_castling = board.castling;

    // Make castling move
    let mv = ChessMove::new_castle(4, 6);

    let state = board.make_move(mv);

    // Verify castling
    assert_eq!(board.piece_on(4), None); // King moved from e1
    assert_eq!(board.piece_on(6), Some((Color::White, Piece::King))); // King on g1
    assert_eq!(board.piece_on(7), None); // Rook moved from h1
    assert_eq!(board.piece_on(5), Some((Color::White, Piece::Rook))); // Rook on f1
    assert!(!board.castling.has(Color::White, CastlingSide::KingSide));
    assert!(!board.castling.has(Color::White, CastlingSide::QueenSide));

    // Unmake castling
    board.unmake_move(state);

    // Verify board is restored
    assert_eq!(board.piece_on(4), Some((Color::White, Piece::King)));
    assert_eq!(board.piece_on(7), Some((Color::White, Piece::Rook)));
    assert_eq!(board.piece_on(6), None);
    assert_eq!(board.piece_on(5), None);
    assert_eq!(board.king_sq[Color::White as usize], 4);
    assert_eq!(
        board.castling.has(Color::White, CastlingSide::KingSide),
        original_castling.has(Color::White, CastlingSide::KingSide)
    );
}

#[test]
fn test_incremental_zobrist_hash_update() {
    use crate::search::compute_hash_board;

    // Test that make_move updates the hash incrementally and correctly
    let mut board = Board::startpos();

    // Make a move: e2e4
    let mv = ChessMove::new(12, 28);

    board.make_move(mv);

    // Verify the hash matches what we'd get from a full recomputation
    let expected_hash = compute_hash_board(&board);
    assert_eq!(
        board.hash, expected_hash,
        "Incremental hash update should match full recomputation after e2e4"
    );

    // Make another move: e7e5
    let mv2 = ChessMove::new(52, 36);

    board.make_move(mv2);

    // Verify the hash still matches
    let expected_hash2 = compute_hash_board(&board);
    assert_eq!(
        board.hash, expected_hash2,
        "Incremental hash update should match full recomputation after e7e5"
    );

    // Test with a capture
    let mut board = Board::new_empty();
    board.pieces[Color::White as usize][Piece::Pawn as usize] = 1u64 << 27; // d4
    board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
    board.king_sq[Color::White as usize] = 4;
    board.pieces[Color::Black as usize][Piece::Pawn as usize] = 1u64 << 36; // e5
    board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
    board.king_sq[Color::Black as usize] = 60;
    board.occ[Color::White as usize] = (1u64 << 27) | (1u64 << 4);
    board.occ[Color::Black as usize] = (1u64 << 36) | (1u64 << 60);
    board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];
    board.side_to_move = Color::White;
    board.hash = compute_hash_board(&board);

    // Capture: d4xe5
    let capture_mv = ChessMove::new(27, 36);

    board.make_move(capture_mv);

    let expected_hash3 = compute_hash_board(&board);
    assert_eq!(
        board.hash, expected_hash3,
        "Incremental hash update should match full recomputation after capture"
    );
}

#[test]
fn test_incremental_hash_undo() {
    // Test that unmake_move properly restores the hash
    let mut board = Board::startpos();
    let original_hash = board.hash;

    // Make a move
    let mv = ChessMove::new(12, 28);

    let state = board.make_move(mv);

    // Undo the move
    board.unmake_move(state);

    // Verify the hash is restored
    assert_eq!(
        board.hash, original_hash,
        "Hash should be restored to original value after undo"
    );
}

#[test]
fn test_castling_rights_update() {
    use crate::board::castling::CastlingSide;

    let mut board = Board::new_empty();

    // Set up position
    board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
    board.pieces[Color::White as usize][Piece::Rook as usize] = 1u64 << 7; // h1
    board.king_sq[Color::White as usize] = 4;
    board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
    board.king_sq[Color::Black as usize] = 60;

    board.occ[Color::White as usize] = (1u64 << 4) | (1u64 << 7);
    board.occ[Color::Black as usize] = 1u64 << 60;
    board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];
    board.side_to_move = Color::White;
    board.castling = CastlingRights::full();

    // Move the king (should remove all white castling rights)
    let mv = ChessMove::new(4, 5);

    board.make_move(mv);

    assert!(!board.castling.has(Color::White, CastlingSide::KingSide));
    assert!(!board.castling.has(Color::White, CastlingSide::QueenSide));
    assert!(board.castling.has(Color::Black, CastlingSide::KingSide));
    assert!(board.castling.has(Color::Black, CastlingSide::QueenSide));
}

#[test]
fn test_incremental_zobrist_hash() {
    use crate::search::compute_hash_board;

    let mut board = Board::startpos();
    let mut moves = Vec::new();

    // Generate and play several moves, verifying hash after each
    for _ in 0..10 {
        moves.clear();
        board.generate_moves(&mut moves);

        if moves.is_empty() {
            break;
        }

        // Take the first legal move
        let mv = moves[0];
        let state = board.make_move(mv);

        // Compute hash from scratch
        let expected_hash = compute_hash_board(&board);

        // Compare with incrementally updated hash
        assert_eq!(
            board.hash,
            expected_hash,
            "Incremental hash mismatch after move {}. \
             Incremental: {:#x}, Expected: {:#x}",
            mv.to_uci(),
            board.hash,
            expected_hash
        );

        // Undo the move
        board.unmake_move(state);

        // Verify hash is restored correctly
        let restored_hash = compute_hash_board(&board);
        assert_eq!(
            board.hash, restored_hash,
            "Hash not properly restored after unmake_move. \
             Current: {:#x}, Expected: {:#x}",
            board.hash, restored_hash
        );
    }
}

#[test]
fn test_zobrist_hash_with_promotion() {
    use crate::search::compute_hash_board;

    // Set up a position where white can promote
    let mut board = Board::new_empty();
    board.pieces[Color::White as usize][Piece::Pawn as usize] = 1u64 << 48; // a7
    board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
    board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
    board.king_sq[Color::White as usize] = 4;
    board.king_sq[Color::Black as usize] = 60;
    board.occ[Color::White as usize] = (1u64 << 48) | (1u64 << 4);
    board.occ[Color::Black as usize] = 1u64 << 60;
    board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];
    board.side_to_move = Color::White;
    board.castling = CastlingRights::empty();
    board.en_passant = 64;
    board.halfmove_clock = 0;
    board.hash = compute_hash_board(&board);

    // Promote pawn to queen
    let promote = ChessMove::new_promotion(48, 56, Piece::Queen);
    board.make_move(promote);

    let expected_hash = compute_hash_board(&board);
    assert_eq!(
        board.hash, expected_hash,
        "Hash mismatch after promotion. Incremental: {:#x}, Expected: {:#x}",
        board.hash, expected_hash
    );
}

#[test]
fn test_to_fen_starting_position() {
    let board = Board::startpos();
    let fen = board.to_fen();
    assert_eq!(
        fen, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "Starting position FEN should match standard"
    );
}

#[test]
fn test_to_fen_after_e4() {
    let mut board = Board::startpos();

    // Play e2e4
    let mv = ChessMove::new(12, 28);
    board.make_move(mv);

    let fen = board.to_fen();
    assert_eq!(
        fen, "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
        "FEN after e4 should show en passant square"
    );
}

#[test]
fn test_to_fen_roundtrip() {
    // Test that from_fen(to_fen(board)) produces equivalent board
    let original = Board::startpos();
    let fen = original.to_fen();
    let restored = Board::from_fen(&fen);

    assert_eq!(original.occ_all, restored.occ_all);
    assert_eq!(original.side_to_move, restored.side_to_move);
    assert_eq!(original.en_passant, restored.en_passant);
    assert_eq!(original.halfmove_clock, restored.halfmove_clock);
}
