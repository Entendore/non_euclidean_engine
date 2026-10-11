use macroquad::prelude::*;
use crate::geometry::GeoPoint;

/// A trigger zone that fires events when the player enters/exits/stays in a region.
pub enum TriggerEvent {
    OnEnter,
    OnExit,
    OnStay,
    OnProximity(f32),  // fires when player within given distance of center
}

/// What happens when a trigger fires.
pub enum TriggerAction {
    Message(String, f64),           // show message for duration
    ScreenFlash(Color, f32),        // flash screen
    ScreenShake(f32, f32),          // (intensity, duration)
    KillPlayer,                      // trigger level failure
    Teleport(f32, f32),             // teleport player to (x, y)
    CompleteLevel,                   // trigger level completion
    ParticleBurst(Color, usize),    // emit particles
    Custom,                          // level handles it via callback flag
}

/// A trigger zone in the level.
pub struct TriggerZone {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub event: TriggerEvent,
    pub action: TriggerAction,
    pub one_shot: bool,
    pub triggered: bool,
    pub stay_timer: f32,
    pub stay_threshold: f32,
    pub cooldown: f32,
    pub active: bool,
}

impl TriggerZone {
    pub fn new(x: f32, y: f32, radius: f32, event: TriggerEvent, action: TriggerAction) -> Self {
        Self {
            x, y, radius, event, action,
            one_shot: false, triggered: false,
            stay_timer: 0.0, stay_threshold: 0.5,
            cooldown: 0.0, active: true,
        }
    }

    pub fn one_shot(mut self) -> Self {
        self.one_shot = true;
        self
    }

    pub fn stay_threshold(mut self, t: f32) -> Self {
        self.stay_threshold = t;
        self
    }

    /// Check if this trigger should fire. Returns Some(&TriggerAction) if it fires.
    pub fn check(&mut self, player: GeoPoint, dt: f32) -> Option<&TriggerAction> {
        if !self.active { return None; }
        if self.one_shot && self.triggered { return None; }
        if self.cooldown > 0.0 { self.cooldown -= dt; return None; }

        let dx = player.x - self.x;
        let dy = player.y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();
        let inside = dist < self.radius;

        match &self.event {
            TriggerEvent::OnEnter => {
                if inside && !self.triggered {
                    self.triggered = true;
                    return Some(&self.action);
                }
                if !inside { self.triggered = false; }
            }
            TriggerEvent::OnExit => {
                if !inside && self.triggered {
                    self.triggered = false;
                    return Some(&self.action);
                }
                if inside { self.triggered = true; }
            }
            TriggerEvent::OnStay => {
                if inside {
                    self.stay_timer += dt;
                    if self.stay_timer >= self.stay_threshold {
                        self.stay_timer = 0.0;
                        return Some(&self.action);
                    }
                } else {
                    self.stay_timer = 0.0;
                }
            }
            TriggerEvent::OnProximity(prox_dist) => {
                if dist < *prox_dist {
                    self.stay_timer += dt;
                    if self.stay_timer >= 0.3 {
                        self.stay_timer = 0.0;
                        return Some(&self.action);
                    }
                } else {
                    self.stay_timer = 0.0;
                }
            }
        }
        None
    }
}

/// Manages multiple trigger zones.
pub struct TriggerSystem {
    pub zones: Vec<TriggerZone>,
    pub pending_actions: Vec<TriggerAction>,
}

impl TriggerSystem {
    pub fn new() -> Self {
        Self { zones: Vec::new(), pending_actions: Vec::new() }
    }

    pub fn add(&mut self, zone: TriggerZone) {
        self.zones.push(zone);
    }

    /// Check all triggers. Collects actions into pending_actions.
    pub fn update(&mut self, player: GeoPoint, dt: f32) {
        self.pending_actions.clear();
        for zone in &mut self.zones {
            if let Some(action) = zone.check(player, dt) {
                // Clone the action
                self.pending_actions.push(Self::clone_action(action));
            }
        }
    }

    fn clone_action(a: &TriggerAction) -> TriggerAction {
        match a {
            TriggerAction::Message(s, d) => TriggerAction::Message(s.clone(), *d),
            TriggerAction::ScreenFlash(c, d) => TriggerAction::ScreenFlash(*c, *d),
            TriggerAction::ScreenShake(i, d) => TriggerAction::ScreenShake(*i, *d),
            TriggerAction::KillPlayer => TriggerAction::KillPlayer,
            TriggerAction::Teleport(x, y) => TriggerAction::Teleport(*x, *y),
            TriggerAction::CompleteLevel => TriggerAction::CompleteLevel,
            TriggerAction::ParticleBurst(c, n) => TriggerAction::ParticleBurst(*c, *n),
            TriggerAction::Custom => TriggerAction::Custom,
        }
    }

    /// Draw trigger zone circles (debug/visual).
    pub fn draw(&self, cam_x: f32, cam_y: f32, zoom: f32, sw: f32, sh: f32) {
        use crate::engine::renderer;
        for zone in &self.zones {
            if !zone.active { continue; }
            let alpha = if zone.triggered { 0.15 } else { 0.05 };
            renderer::draw_circle_outline_world(
                zone.x, zone.y, zone.radius,
                cam_x, cam_y, zoom, sw, sh,
                Color::new(1.0, 1.0, 0.0, alpha), 1.0);
        }
    }
}