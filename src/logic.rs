pub mod birdy;
pub mod rock;
pub mod coin;

pub const PLAYFIELD_BOUNCE_COEFFICIENT: f32 = -0.75; // portion of player's velocity to reflect when they collide with the bottom of the playfield.

const DESPAWN_DISTANCE: f32 = 2.0;

#[derive(Copy, Clone)]
pub struct PhysObj {
    pub x: f32,
    pub y: f32,
    pub x_velocity: f32,
    pub y_velocity: f32,
    pub size: f32,
}
impl PhysObj {
    fn position_delta(&mut self, time_delta: f32) {
        (*self).x += (*self).x_velocity * time_delta;
        (*self).y += (*self).y_velocity * time_delta;
    }
}

fn rand_range(min: f32, max: f32) -> f32 {
    rand::random::<f32>() * (max - min) + min
}

fn despawn_objs(phys_objs: &mut Vec<PhysObj>) {
    let mut i = 0;
    while i < phys_objs.len() {
        if f32::abs(phys_objs[i].y) > DESPAWN_DISTANCE {
            phys_objs.remove(i);
        } else {
            i += 1;
        }
    }
}

fn objs_overlap (a: PhysObj, b: PhysObj) -> bool {
    a.x - a.size < b.x + b.size
        && a.x + a.size > b.x - b.size
        && a.y - a.size < b.y + b.size
        && a.y + a.size > b.y - b.size
}

pub fn tick (game_state: &mut super::GameState, now: std::time::Instant, time_delta: f32) {
    // update positions
    (*game_state).birdy.position_delta(time_delta);
    for rock in (*game_state).rocks.iter_mut() {
        rock.position_delta(time_delta);
    }
    for coin in (*game_state).coins.iter_mut() {
        coin.position_delta(time_delta);
    }

    // update player velocity for next frame
    (*game_state).birdy.x_velocity = if (*game_state).birdy.x_velocity.is_sign_positive() {
	f32::max((*game_state).birdy.x_velocity + (birdy::DECCEL_MOVE * time_delta), 0.0)
    }
    else {
	f32::min((*game_state).birdy.x_velocity - (birdy::DECCEL_MOVE * time_delta), 0.0)
    };
    (*game_state).birdy.y_velocity = f32::max(
	(*game_state).birdy.y_velocity + birdy::ACCEL_GRAV * time_delta,
	birdy::TERMINAL_VELOCITY
    );
    let mut i = 0;
    while i < game_state.keys.len() {
	match game_state.keys[i] {
            glium::glutin::event::VirtualKeyCode::Space => {
                if match game_state.last_jump_time {
                    None => std::time::Duration::MAX,
                    Some(time) => now.duration_since(time),
                } > birdy::JUMP_COOLDOWN
                {
                    game_state.last_jump_time = Some(now);
                    game_state.birdy.y_velocity = birdy::ACCEL_JUMP;
                }
            }
            glium::glutin::event::VirtualKeyCode::Left => {
                game_state.birdy.x_velocity = birdy::ACCEL_MOVE * -1.0
            }
            glium::glutin::event::VirtualKeyCode::Right => {
                game_state.birdy.x_velocity = birdy::ACCEL_MOVE
            }
            _ => (),
        }
	i += 1;
    }

    // rock spawning
    if match (*game_state).last_rock_spawn_time {
        None => std::time::Duration::MAX,
        Some(time) => now.duration_since(time),
    } > rock::COOLDOWN
    {
        (*game_state).last_rock_spawn_time = Some(now);
        let size = rand_range(rock::MIN_SIZE, rock::MAX_SIZE);
        let mut x = rand::random::<f32>() * (1.0 - size);
        if rand::random() {
            x *= -1.0;
        }
        (*game_state).rocks.push(PhysObj {
            x,
	    y: rock::SPAWN_DIST * (*game_state).rock_fall_direction,
	    x_velocity: 0.0,
	    y_velocity: (*game_state).rock_fall_direction
		* -1.0
		* rand_range(rock::MIN_VELOCITY, rock::MAX_VELOCITY),
	    size: size,
        });
        (*game_state).rock_fall_direction *= -1.0;
    }

    // coin spawning
    if match (*game_state).last_coin_spawn_time {
        None => std::time::Duration::MAX,
        Some(time) => now.duration_since(time),
    } > coin::COOLDOWN
    {
        (*game_state).last_coin_spawn_time = Some(now);
        let size = rand_range(coin::MIN_SIZE, coin::MAX_SIZE);
        let mut x = rand::random::<f32>() * (1.0 - size);
        if rand::random() {
            x *= -1.0;
        }
        let coin_fall_direction = if rand::random() { 1.0 } else { -1.0 };
        (*game_state).coins.push(PhysObj {
            x,
	    y: coin::SPAWN_DIST * coin_fall_direction,
            x_velocity: 0.0,
            y_velocity: coin_fall_direction * -1.0 * rand_range(coin::MIN_VELOCITY, coin::MAX_VELOCITY),
            size: size,
        });
    }

    // despawning
    despawn_objs(&mut(*game_state).rocks);
    despawn_objs(&mut(*game_state).coins);

    // birdy-playfield edge collision
     if (*game_state).birdy.x - (*game_state).birdy.size < -1.0 {
        (*game_state).birdy.x = -1.0 + (*game_state).birdy.size;
    }
    if (*game_state).birdy.x + (*game_state).birdy.size > 1.0 {
        (*game_state).birdy.x = 1.0 - (*game_state).birdy.size;
    }
    if (*game_state).birdy.y - (*game_state).birdy.size < -1.0 {
        (*game_state).birdy.y = -1.0 + (*game_state).birdy.size;
        (*game_state).birdy.y_velocity *= PLAYFIELD_BOUNCE_COEFFICIENT; // bounce
    }
    if (*game_state).birdy.y + (*game_state).birdy.size > 1.0 {
        (*game_state).birdy.y = 1.0 - (*game_state).birdy.size;
    }

    // birdy-rock collision
    let mut i = 0;
    while i < (*game_state).rocks.len() {
        let rock = (*game_state).rocks[i];
        if objs_overlap((*game_state).birdy, rock) {
	    (*game_state).dead = true;
	    return; // this game is over, no point in doing anything else
        }
        i += 1;
    }

    // birdy-coin collision
    let mut i = 0;
    while i < (*game_state).coins.len() {
        let coin = (*game_state).coins[i];
        if objs_overlap((*game_state).birdy, coin) {
            (*game_state).score += 1;
            (*game_state).coins.remove(i);
        } else {
            i += 1;
        }
    }
}
