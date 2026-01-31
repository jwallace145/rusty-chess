use crate::{
    board::Color,
    terminal::{BlackOpeningBook, ChessEngineSettings, DisplaySettings, WhiteOpeningBook},
};

pub fn print_instructions(settings: &ChessEngineSettings, display: &DisplaySettings) {
    let color_str = match settings.player_color {
        Color::White => "White",
        Color::Black => "Black",
    };

    // Determine the AI's opening book based on player color
    let opening_book_str = match settings.player_color {
        Color::White => {
            // AI plays Black
            match settings.black_opening_book {
                BlackOpeningBook::None => "None",
            }
        }
        Color::Black => {
            // AI plays White
            match settings.white_opening_book {
                WhiteOpeningBook::None => "None",
                WhiteOpeningBook::LondonSystem => "London System",
            }
        }
    };

    println!("┌─────────────────────────────────────────┐");
    println!("│            Game Settings                │");
    println!("├─────────────────────────────────────────┤");
    println!("│  Player color:     {:>19}  │", color_str);
    println!("│  AI search depth:  {:>15} ply  │", settings.search_depth);
    println!("│  AI opening book:  {:>19}  │", opening_book_str);
    println!("├─────────────────────────────────────────┤");
    println!("│            Commands                     │");
    println!("├─────────────────────────────────────────┤");
    println!("│  e2,e4    - Make a move (from,to)       │");
    println!("│  moves    - Show all legal moves        │");
    println!("│  undo     - Undo last move pair         │");
    println!("│  fen      - Show current FEN            │");
    println!("│  eval     - Show position evaluation    │");
    println!("│  resign   - Resign the game             │");
    println!("│  quit     - Exit the game               │");
    println!("├─────────────────────────────────────────┤");
    println!("│       Display Toggles (for devs)        │");
    println!("├─────────────────────────────────────────┤");
    println!(
        "│  stats    - Search statistics     [{}] │",
        if display.show_search_stats {
            "ON "
        } else {
            "OFF"
        }
    );
    println!(
        "│  tt       - Transposition table   [{}] │",
        if display.show_tt_info { "ON " } else { "OFF" }
    );
    println!(
        "│  evalon   - Eval before/after     [{}] │",
        if display.show_eval { "ON " } else { "OFF" }
    );
    println!(
        "│  analysis - Move analysis         [{}] │",
        if display.show_move_analysis {
            "ON "
        } else {
            "OFF"
        }
    );
    println!("│  verbose  - Toggle all on/off           │");
    println!("│  display  - Show current settings       │");
    println!("└─────────────────────────────────────────┘");
    println!();
}
