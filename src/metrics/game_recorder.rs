use crate::board::Color;
use crate::metrics::output::{AiMoveMetrics, GameMetadata, GameRecording, GameResult, MoveRecord};
use chrono::Local;
use std::fs;
use std::io::Write;
use std::time::Instant;

pub struct GameRecorder {
    recording: GameRecording,
    start_instant: Instant,
    output_dir: String,
}

impl GameRecorder {
    pub fn new(player_color: Color, ai_depth: u8) -> Self {
        let now = Local::now();
        let date = now.format("%Y-%m-%d").to_string();
        let start_time = now.format("%H:%M:%S").to_string();

        let metadata = GameMetadata::new(player_color, ai_depth, start_time, date);
        let recording = GameRecording::new(metadata);

        Self {
            recording,
            start_instant: Instant::now(),
            output_dir: "game_recordings".to_string(),
        }
    }

    pub fn record_player_move(&mut self, move_number: u16, color: Color, move_notation: String) {
        let move_record = MoveRecord::new_player_move(move_number, color, move_notation);
        self.recording.add_move(move_record);
    }

    pub fn record_ai_move(
        &mut self,
        move_number: u16,
        color: Color,
        move_notation: String,
        metrics: AiMoveMetrics,
    ) {
        let move_record = MoveRecord::new_ai_move(move_number, color, move_notation, metrics);
        self.recording.add_move(move_record);
    }

    pub fn finalize_and_save(&mut self, result: GameResult) -> Result<String, String> {
        let now = Local::now();
        let end_time = now.format("%H:%M:%S").to_string();
        let duration = self.start_instant.elapsed();

        self.recording
            .finalize(end_time, duration.as_secs(), result);

        // Create output directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&self.output_dir) {
            return Err(format!("Failed to create output directory: {}", e));
        }

        // Generate filename with timestamp
        let filename = format!(
            "{}/game_{}.json",
            self.output_dir,
            now.format("%Y-%m-%d_%H-%M-%S")
        );

        // Serialize to JSON
        let json = match serde_json::to_string_pretty(&self.recording) {
            Ok(j) => j,
            Err(e) => return Err(format!("Failed to serialize game data: {}", e)),
        };

        // Write to file
        match fs::File::create(&filename) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(json.as_bytes()) {
                    return Err(format!("Failed to write to file: {}", e));
                }
            }
            Err(e) => return Err(format!("Failed to create file: {}", e)),
        }

        Ok(filename)
    }
}
