#[derive(Debug, Clone)]
pub struct EvaluationScores {
    pub material: i32,
    pub position: i32,
    pub pawn_structure: i32,
    pub mobility: i32,
    pub king_safety: i32,
    pub tempo: i32,
    pub bishop_pair: i32,
    pub knight_outpost: i32,
    pub rook_file: i32,
    pub central_control: i32,
    pub threat: i32,
    pub line_pressure: i32,
    pub fork: i32,
    pub total: i32,
}

impl std::fmt::Display for EvaluationScores {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  Material:        {:+5} cp", self.material)?;
        writeln!(f, "  Position:        {:+5} cp", self.position)?;
        writeln!(f, "  Pawn Structure:  {:+5} cp", self.pawn_structure)?;
        writeln!(f, "  Mobility:        {:+5} cp", self.mobility)?;
        writeln!(f, "  King Safety:     {:+5} cp", self.king_safety)?;
        writeln!(f, "  Tempo:           {:+5} cp", self.tempo)?;
        writeln!(f, "  Bishop Pair:     {:+5} cp", self.bishop_pair)?;
        writeln!(f, "  Knight Outpost:  {:+5} cp", self.knight_outpost)?;
        writeln!(f, "  Rook on File:    {:+5} cp", self.rook_file)?;
        writeln!(f, "  Central Control: {:+5} cp", self.central_control)?;
        writeln!(f, "  Threats:         {:+5} cp", self.threat)?;
        writeln!(f, "  Line Pressure:   {:+5} cp", self.line_pressure)?;
        writeln!(f, "  Fork Potential:  {:+5} cp", self.fork)?;
        writeln!(f, "  ─────────────────────────")?;
        write!(f, "  TOTAL:           {:+5} cp", self.total)
    }
}
