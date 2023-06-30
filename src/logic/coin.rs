pub const COOLDOWN: std::time::Duration = std::time::Duration::from_millis(3000);
pub const MIN_VELOCITY: f32 = 0.75;
pub const MAX_VELOCITY: f32 = 1.0;
pub const SPAWN_DIST: f32 = 1.5;
pub const MIN_SIZE: f32 = 0.05;
pub const MAX_SIZE: f32 = 0.1;

pub fn new_coin() -> super::PhysObj {
    let fall_direction = if rand::random() { 1.0 } else { -1.0 };
    let size = super::rand_range(MIN_SIZE, MAX_SIZE);
    let mut x = rand::random::<f32>() * (1.0 - size);
    if rand::random() {
        x *= -1.0;
    }
    super::PhysObj {
        x,
        y: SPAWN_DIST * fall_direction,
        x_velocity: 0.0,
        y_velocity: fall_direction * -1.0 * super::rand_range(MIN_VELOCITY, MAX_VELOCITY),
        size: size,
    }
}
