use crate::board::Color;
use serde::{Deserialize, Serialize};

/// Top-level structure containing all game data
#[derive(Debug, Serialize, Deserialize)]
pub struct GameRecording {
    pub metadata: GameMetadata,
    pub moves: Vec<MoveRecord>,
}

/// Game metadata and overall statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct GameMetadata {
    pub date: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_seconds: Option<u64>,
    pub player_color: String,
    pub ai_depth: u8,
    pub result: GameResult,
    pub final_position_fen: Option<String>,
}

/// Game result enum
#[derive(Debug, Serialize, Deserialize)]
pub enum GameResult {
    PlayerWin,
    AIWin,
    Draw,
    InProgress,
}

/// Per-move record containing move information and optional AI metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct MoveRecord {
    pub move_number: u16,
    pub color: String,
    pub move_notation: String,
    pub ai_metrics: Option<AiMoveMetrics>,
}

/// AI search metrics for a single move
#[derive(Debug, Serialize, Deserialize)]
pub struct AiMoveMetrics {
    pub search_time_ms: u128,
    pub nodes_explored: u64,
    pub nodes_per_second: u64,
    pub beta_cutoffs: u64,
    pub beta_cutoff_percentage: f64,
    pub max_depth_reached: u8,
    pub tt_size_bytes: usize,
    pub tt_num_entries: usize,
    pub tt_hits: usize,
    pub tt_misses: usize,
    pub tt_hit_rate_percentage: f64,
}

impl GameRecording {
    pub fn new(metadata: GameMetadata) -> Self {
        Self {
            metadata,
            moves: Vec::new(),
        }
    }

    pub fn add_move(&mut self, move_record: MoveRecord) {
        self.moves.push(move_record);
    }

    pub fn finalize(&mut self, end_time: String, duration_seconds: u64, result: GameResult) {
        self.metadata.end_time = Some(end_time);
        self.metadata.duration_seconds = Some(duration_seconds);
        self.metadata.result = result;
    }
}

impl GameMetadata {
    pub fn new(player_color: Color, ai_depth: u8, start_time: String, date: String) -> Self {
        let player_color_str = match player_color {
            Color::White => "White".to_string(),
            Color::Black => "Black".to_string(),
        };

        Self {
            date,
            start_time,
            end_time: None,
            duration_seconds: None,
            player_color: player_color_str,
            ai_depth,
            result: GameResult::InProgress,
            final_position_fen: None,
        }
    }
}

impl MoveRecord {
    pub fn new_player_move(move_number: u16, color: Color, move_notation: String) -> Self {
        let color_str = match color {
            Color::White => "White".to_string(),
            Color::Black => "Black".to_string(),
        };

        Self {
            move_number,
            color: color_str,
            move_notation,
            ai_metrics: None,
        }
    }

    pub fn new_ai_move(
        move_number: u16,
        color: Color,
        move_notation: String,
        metrics: AiMoveMetrics,
    ) -> Self {
        let color_str = match color {
            Color::White => "White".to_string(),
            Color::Black => "Black".to_string(),
        };

        Self {
            move_number,
            color: color_str,
            move_notation,
            ai_metrics: Some(metrics),
        }
    }
}
