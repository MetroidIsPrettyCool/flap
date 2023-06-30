pub const ACCEL_JUMP: f32 = 0.6;
pub const ACCEL_MOVE: f32 = 0.4;
pub const JUMP_COOLDOWN: std::time::Duration = std::time::Duration::from_millis(250);
pub const ACCEL_GRAV: f32 = -0.9;
pub const DECCEL_MOVE: f32 = -0.6; // decceleration applied when player isn't holding a direction key
pub const TERMINAL_VELOCITY: f32 = -0.6;

pub fn new_birdy() -> super::PhysObj {
    super::PhysObj {
        x: 0.0,
        y: 0.0,
        x_velocity: 0.0,
        y_velocity: 0.0,
        size: 0.05,
    }
}
