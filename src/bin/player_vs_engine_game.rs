// Chess Engine - MIT License, 2026 James Wallace

use rusty_chess::board::Color;
use rusty_chess::engine::AiGame;
use rusty_chess::terminal::{
    ChessEngineSettings, DisplaySettings, get_chess_engine_settings, print_instructions,
    print_introduction,
};

fn main() {
    // Parse command-line arguments for display settings
    let display_settings: DisplaySettings = DisplaySettings::from_args();

    print_introduction();
    let settings: ChessEngineSettings = get_chess_engine_settings();
    print_instructions(&settings, &display_settings);

    let mut game: AiGame = AiGame::new(
        settings.player_color,
        settings.search_depth,
        settings.starting_position,
        display_settings,
        settings.white_opening_book,
        settings.black_opening_book,
    );

    // If player chose black and it's white's turn, AI makes the first move
    if settings.player_color == Color::Black && game.side_to_move() == Color::White {
        println!("\nAI will make the first move as White.\n");
    }

    game.run();
}
