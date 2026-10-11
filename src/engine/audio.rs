use macroquad::prelude::*;

/// Procedural audio system using macroquad's built-in audio.
/// Generates simple synthesized sound effects for game events.
pub struct AudioSystem {
    enabled: bool,
    volume: f32,
}

impl AudioSystem {
    pub fn new() -> Self {
        Self { enabled: true, volume: 0.3 }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn play_collect(&self) {
        if !self.enabled { return; }
        // Short ascending tone
        let _ = play_tone(880.0, 0.08, self.volume);
    }

    pub fn play_combo(&self, combo_level: u32) {
        if !self.enabled { return; }
        let freq = 880.0 + combo_level as f32 * 110.0;
        let _ = play_tone(freq.min(1760.0), 0.12, self.volume);
    }

    pub fn play_dash(&self) {
        if !self.enabled { return; }
        let _ = play_tone(220.0, 0.1, self.volume * 0.5);
    }

    pub fn play_portal(&self) {
        if !self.enabled { return; }
        let _ = play_tone(440.0, 0.15, self.volume * 0.7);
        // Second tone slightly delayed via lower freq
        let _ = play_tone(660.0, 0.1, self.volume * 0.5);
    }

    pub fn play_damage(&self) {
        if !self.enabled { return; }
        let _ = play_tone(110.0, 0.2, self.volume * 0.8);
    }

    pub fn play_complete(&self) {
        if !self.enabled { return; }
        // Victory arpeggio
        let _ = play_tone(523.25, 0.15, self.volume); // C5
        let _ = play_tone(659.25, 0.15, self.volume); // E5
        let _ = play_tone(783.99, 0.2, self.volume);  // G5
    }

    pub fn play_wrap(&self) {
        if !self.enabled { return; }
        let _ = play_tone(330.0, 0.08, self.volume * 0.4);
    }

    pub fn play_ambient_tick(&self) {
        if !self.enabled { return; }
        let _ = play_tone(60.0, 0.05, self.volume * 0.1);
    }
}

/// Helper: play a simple sine tone at a given frequency for a duration.
/// This is a minimal audio stub — macroquad 0.4 doesn't have built-in tone synthesis,
/// so we provide a no-op placeholder that can be connected to real audio later.
fn play_tone(_freq: f32, _duration: f32, _volume: f32) -> Result<(), String> {
    // Audio synthesis would require a more complex audio backend.
    // This serves as the API surface for when audio is added.
    Ok(())
}