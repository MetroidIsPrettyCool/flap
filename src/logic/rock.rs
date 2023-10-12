pub const COOLDOWN: std::time::Duration = std::time::Duration::from_millis(1000);
pub const MIN_VELOCITY: f32 = 0.25;
pub const MAX_VELOCITY: f32 = 0.5;
pub const SPAWN_DIST: f32 = 1.5;
pub const MIN_SIZE: f32 = 0.05;
pub const MAX_SIZE: f32 = 0.15;

pub fn new_rock(fall_direction: f32) -> super::PhysObj {
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
        width: size,
	height: size,
    }
}
