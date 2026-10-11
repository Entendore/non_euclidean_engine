use macroquad::prelude::*;

/// Unified input state collected each frame.
pub struct InputState {
    pub move_x: f32,
    pub move_y: f32,
    pub action_pressed: bool,
    pub restart_pressed: bool,
    pub next_level_pressed: bool,
    pub dash_pressed: bool,
    pub pause_pressed: bool,
    pub mouse_clicked: bool,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub fps_toggle: bool,
    pub tab_pressed: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            move_x: 0.0,
            move_y: 0.0,
            action_pressed: false,
            restart_pressed: false,
            next_level_pressed: false,
            dash_pressed: false,
            pause_pressed: false,
            mouse_clicked: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
            fps_toggle: false,
            tab_pressed: false,
        }
    }

    pub fn update(&mut self) {
        self.move_x = 0.0;
        self.move_y = 0.0;

        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            self.move_y = -1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            self.move_y = 1.0;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            self.move_x = -1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            self.move_x = 1.0;
        }

        // Normalize diagonal movement
        let mag = (self.move_x * self.move_x + self.move_y * self.move_y).sqrt();
        if mag > 1.0 {
            self.move_x /= mag;
            self.move_y /= mag;
        }

        self.action_pressed = is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::E);
        self.restart_pressed = is_key_pressed(KeyCode::R);
        self.next_level_pressed = is_key_pressed(KeyCode::N);
        self.dash_pressed = is_key_pressed(KeyCode::LeftShift) || is_key_pressed(KeyCode::RightShift) || is_key_pressed(KeyCode::F);
        self.pause_pressed = is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape);

        self.mouse_clicked = is_mouse_button_pressed(MouseButton::Left);
        self.mouse_x = mouse_position().0;
        self.mouse_y = mouse_position().1;
        self.fps_toggle = is_key_pressed(KeyCode::F3);
        self.tab_pressed = is_key_pressed(KeyCode::Tab);
    }
}