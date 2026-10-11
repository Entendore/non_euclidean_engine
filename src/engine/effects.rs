use macroquad::prelude::*;

/// Screen-wide visual effects system.
pub struct ScreenEffects {
    pub shake_intensity: f32,
    pub shake_duration: f32,
    pub shake_timer: f32,
    pub flash_color: Color,
    pub flash_duration: f32,
    pub flash_timer: f32,
    pub transition_alpha: f32,
    pub transition_duration: f32,
    pub transition_timer: f32,
    pub transition_color: Color,
    pub transition_active: bool,
    pub transition_phase: TransitionPhase,
    pub chromatic_aberration: f32,
    pub chromatic_timer: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TransitionPhase {
    FadeOut,
    FadeIn,
    None,
}

impl ScreenEffects {
    pub fn new() -> Self {
        Self {
            shake_intensity: 0.0,
            shake_duration: 0.0,
            shake_timer: 0.0,
            flash_color: WHITE,
            flash_duration: 0.0,
            flash_timer: 0.0,
            transition_alpha: 0.0,
            transition_duration: 0.8,
            transition_timer: 0.0,
            transition_color: BLACK,
            transition_active: false,
            transition_phase: TransitionPhase::None,
            chromatic_aberration: 0.0,
            chromatic_timer: 0.0,
        }
    }

    /// Trigger a screen shake.
    pub fn shake(&mut self, intensity: f32, duration: f32) {
        self.shake_intensity = intensity;
        self.shake_duration = duration;
        self.shake_timer = 0.0;
    }

    /// Trigger a screen flash.
    pub fn flash(&mut self, color: Color, duration: f32) {
        self.flash_color = color;
        self.flash_duration = duration;
        self.flash_timer = 0.0;
    }

    /// Start a fade-out/fade-in transition.
    pub fn start_transition(&mut self, color: Color, duration: f32) {
        self.transition_active = true;
        self.transition_color = color;
        self.transition_duration = duration;
        self.transition_timer = 0.0;
        self.transition_phase = TransitionPhase::FadeOut;
        self.transition_alpha = 0.0;
    }

    /// Trigger chromatic aberration effect.
    pub fn chromatic(&mut self, amount: f32, duration: f32) {
        self.chromatic_aberration = amount;
        self.chromatic_timer = 0.0;
        // reuse shake duration
        self.shake_duration = self.shake_duration.max(duration);
    }

    /// Returns (offset_x, offset_y) for screen shake.
    pub fn get_shake_offset(&self) -> (f32, f32) {
        if self.shake_timer < self.shake_duration {
            let decay = 1.0 - (self.shake_timer / self.shake_duration);
            let i = self.shake_intensity * decay;
            let t = self.shake_timer * 40.0;
            (i * t.sin(), i * t.cos() * 0.7)
        } else {
            (0.0, 0.0)
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Shake
        if self.shake_timer < self.shake_duration {
            self.shake_timer += dt;
        }
        // Flash
        if self.flash_timer < self.flash_duration {
            self.flash_timer += dt;
        }
        // Transition
        if self.transition_active {
            self.transition_timer += dt;
            let half = self.transition_duration / 2.0;
            match self.transition_phase {
                TransitionPhase::FadeOut => {
                    self.transition_alpha = (self.transition_timer / half).min(1.0);
                    if self.transition_timer >= half {
                        self.transition_phase = TransitionPhase::FadeIn;
                    }
                }
                TransitionPhase::FadeIn => {
                    self.transition_alpha = (1.0 - (self.transition_timer - half) / half).max(0.0);
                    if self.transition_timer >= self.transition_duration {
                        self.transition_active = false;
                        self.transition_phase = TransitionPhase::None;
                        self.transition_alpha = 0.0;
                    }
                }
                TransitionPhase::None => {}
            }
        }
        // Chromatic
        if self.chromatic_timer < self.shake_duration {
            self.chromatic_timer += dt;
            self.chromatic_aberration *= 0.95;
        } else {
            self.chromatic_aberration = 0.0;
        }
    }

    /// Draw flash overlay (call after all world drawing).
    pub fn draw_flash(&self) {
        if self.flash_timer < self.flash_duration {
            let t = self.flash_timer / self.flash_duration;
            let alpha = (1.0 - t) * 0.6;
            draw_rectangle(
                0.0, 0.0, screen_width(), screen_height(),
                Color::new(self.flash_color.r, self.flash_color.g, self.flash_color.b, alpha),
            );
        }
    }

    /// Draw transition overlay.
    pub fn draw_transition(&self) {
        if self.transition_active {
            draw_rectangle(
                0.0, 0.0, screen_width(), screen_height(),
                Color::new(self.transition_color.r, self.transition_color.g, self.transition_color.b, self.transition_alpha),
            );
        }
    }

    pub fn is_transitioning(&self) -> bool {
        self.transition_active
    }
}