use crate::board::{Board, Color};
use std::{
    env,
    io::{self, Write},
};

/// Opening book options for when the AI plays as White.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum WhiteOpeningBook {
    #[default]
    None,
    LondonSystem,
    ColleSystem,
}

/// Opening book options for when the AI plays as Black.
/// Currently no opening books are implemented for Black,
/// but this enum makes it easy to add them in the future.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum BlackOpeningBook {
    #[default]
    None,
    // Future opening books can be added here, e.g.:
    // Sicilian,
    // KingsIndian,
}

pub struct ChessEngineSettings {
    pub player_color: Color,
    pub search_depth: u8,
    pub starting_position: Board,
    pub white_opening_book: WhiteOpeningBook,
    pub black_opening_book: BlackOpeningBook,
}

#[derive(Clone, Default)]
pub struct DisplaySettings {
    pub show_search_stats: bool,
    pub show_tt_info: bool,
    pub show_eval: bool,
    pub show_move_analysis: bool,
}

pub fn get_chess_engine_settings() -> ChessEngineSettings {
    let player_color: Color = get_player_color();
    let search_depth: u8 = get_search_depth();
    let starting_position: Board = get_starting_position();
    let (white_opening_book, black_opening_book) = get_opening_book_settings(player_color);

    ChessEngineSettings {
        player_color,
        search_depth,
        starting_position,
        white_opening_book,
        black_opening_book,
    }
}

fn get_player_color() -> Color {
    println!("┌─────────────────────────────────────────┐");
    println!("│         Choose Your Color               │");
    println!("├─────────────────────────────────────────┤");
    println!("│  [w] White  ♔  - Move first             │");
    println!("│  [b] Black  ♚  - AI moves first         │");
    println!("└─────────────────────────────────────────┘");

    loop {
        print!("  > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        match input.trim().to_lowercase().as_str() {
            "w" | "white" => {
                println!("  ✓ Playing as White\n");
                return Color::White;
            }
            "b" | "black" => {
                println!("  ✓ Playing as Black\n");
                return Color::Black;
            }
            _ => println!("  ✗ Invalid choice. Enter 'w' or 'b'."),
        }
    }
}

fn get_search_depth() -> u8 {
    println!("┌─────────────────────────────────────────┐");
    println!("│         AI Difficulty (1-10)            │");
    println!("├─────────────────────────────────────────┤");
    println!("│  1-3   Beginner    - Fast, weak play    │");
    println!("│  4-5   Intermediate - Balanced          │");
    println!("│  6-7   Advanced    - Strong, slower     │");
    println!("│  8-10  Expert      - Very strong        │");
    println!("├─────────────────────────────────────────┤");
    println!("│  Recommended: 5                         │");
    println!("└─────────────────────────────────────────┘");

    loop {
        print!("  > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        match input.trim().parse::<u8>() {
            Ok(depth) if (1..=10).contains(&depth) => {
                let difficulty = match depth {
                    1..=3 => "Beginner",
                    4..=5 => "Intermediate",
                    6..=7 => "Advanced",
                    _ => "Expert",
                };
                println!("  ✓ Difficulty: {} (depth {})\n", difficulty, depth);
                return depth;
            }
            _ => println!("  ✗ Invalid choice. Enter a number 1-10."),
        }
    }
}

fn get_starting_position() -> Board {
    println!("┌─────────────────────────────────────────┐");
    println!("│        Starting Position                │");
    println!("├─────────────────────────────────────────┤");
    println!("│  [n] Standard - Normal chess setup      │");
    println!("│  [y] Custom   - Load from FEN string    │");
    println!("└─────────────────────────────────────────┘");

    loop {
        print!("  > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        match input.trim().to_lowercase().as_str() {
            "n" | "no" => {
                println!("  ✓ Using standard starting position\n");
                return Board::startpos();
            }
            "y" | "yes" => {
                return get_fen_position();
            }
            _ => println!("  ✗ Invalid choice. Enter 'y' or 'n'."),
        }
    }
}

fn get_fen_position() -> Board {
    println!("┌─────────────────────────────────────────┐");
    println!("│          Enter FEN Position             │");
    println!("├─────────────────────────────────────────┤");
    println!("│  Enter a valid FEN string to load a     │");
    println!("│  custom position. Type 'cancel' to      │");
    println!("│  use the standard starting position.    │");
    println!("├─────────────────────────────────────────┤");
    println!("│  Example FEN (starting position):       │");
    println!("│  rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/    │");
    println!("│  RNBQKBNR w KQkq - 0 1                  │");
    println!("└─────────────────────────────────────────┘");

    loop {
        print!("  FEN> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let input = input.trim();

        if input.to_lowercase() == "cancel" {
            println!("  ✓ Using standard starting position\n");
            return Board::startpos();
        }

        // Try to parse the FEN
        let board = Board::from_fen(input);

        // Validate the board has kings for both sides
        if board.king_sq[0] >= 64 || board.king_sq[1] >= 64 {
            println!("  ✗ Invalid FEN: Both sides must have a king.");
            continue;
        }

        println!("  ✓ FEN loaded successfully!\n");
        return board;
    }
}

fn get_opening_book_settings(player_color: Color) -> (WhiteOpeningBook, BlackOpeningBook) {
    // Determine which color the AI is playing
    let ai_color = match player_color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };

    match ai_color {
        Color::White => {
            let white_book = get_white_opening_book();
            (white_book, BlackOpeningBook::None)
        }
        Color::Black => {
            let black_book = get_black_opening_book();
            (WhiteOpeningBook::None, black_book)
        }
    }
}

fn get_white_opening_book() -> WhiteOpeningBook {
    println!("┌─────────────────────────────────────────┐");
    println!("│      AI Opening Book (White)            │");
    println!("├─────────────────────────────────────────┤");
    println!("│  [n] None         - No opening book     │");
    println!("│  [l] London System - d4, Bf4, e3 setup  │");
    println!("│  [c] Colle System  - d4, Nf3, e3 setup  │");
    println!("└─────────────────────────────────────────┘");

    loop {
        print!("  > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        match input.trim().to_lowercase().as_str() {
            "n" | "none" => {
                println!("  ✓ No opening book selected\n");
                return WhiteOpeningBook::None;
            }
            "l" | "london" => {
                println!("  ✓ London System opening book selected\n");
                return WhiteOpeningBook::LondonSystem;
            }
            "c" | "colle" => {
                println!("  ✓ Colle System opening book selected\n");
                return WhiteOpeningBook::ColleSystem;
            }
            _ => println!("  ✗ Invalid choice. Enter 'n', 'l', or 'c'."),
        }
    }
}

fn get_black_opening_book() -> BlackOpeningBook {
    println!("┌─────────────────────────────────────────┐");
    println!("│      AI Opening Book (Black)            │");
    println!("├─────────────────────────────────────────┤");
    println!("│  [n] None - No opening book             │");
    println!("│                                         │");
    println!("│  (More opening books coming soon!)      │");
    println!("└─────────────────────────────────────────┘");

    loop {
        print!("  > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        match input.trim().to_lowercase().as_str() {
            "n" | "none" => {
                println!("  ✓ No opening book selected\n");
                return BlackOpeningBook::None;
            }
            _ => println!("  ✗ Invalid choice. Enter 'n'."),
        }
    }
}

impl DisplaySettings {
    pub fn from_args() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut settings = Self::default();

        for arg in &args[1..] {
            match arg.as_str() {
                "--stats" | "-s" => settings.show_search_stats = true,
                "--tt" | "-t" => settings.show_tt_info = true,
                "--eval" | "-e" => settings.show_eval = true,
                "--analysis" | "-a" => settings.show_move_analysis = true,
                "--verbose" | "-v" => {
                    settings.show_search_stats = true;
                    settings.show_tt_info = true;
                    settings.show_eval = true;
                    settings.show_move_analysis = true;
                }
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                _ => {}
            }
        }

        settings
    }

    pub fn any_enabled(&self) -> bool {
        self.show_search_stats || self.show_tt_info || self.show_eval || self.show_move_analysis
    }
}

fn print_usage() {
    println!(
        r#"
Rusty Chess - A terminal chess engine written in Rust

USAGE:
    rusty-chess [OPTIONS]

OPTIONS:
    -s, --stats      Show search statistics (time, nodes, depth)
    -t, --tt         Show transposition table information
    -e, --eval       Show position evaluation before/after moves
    -a, --analysis   Show move analysis (position change, material delta)
    -v, --verbose    Enable all display options
    -h, --help       Print this help message

EXAMPLES:
    rusty-chess                  Run with minimal output (default)
    rusty-chess --verbose        Run with all performance insights enabled
    rusty-chess -s -e            Show search stats and evaluation only

IN-GAME COMMANDS:
    You can also toggle these options during gameplay using:
    stats, tt, eval, analysis, verbose
"#
    );
}
