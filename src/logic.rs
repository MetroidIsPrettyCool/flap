use super::GameState;
use glium::glutin::event::VirtualKeyCode;
use std::time::{Duration, Instant};

pub mod birdy;
pub mod coin;
pub mod rock;

pub const PLAYFIELD_BOUNCE_COEFFICIENT: f32 = -0.75; // portion of player's velocity to reflect when they collide with the bottom of the playfield.

const DESPAWN_DISTANCE: f32 = 2.5;

#[derive(Copy, Clone)]
pub struct PhysObj {
    pub x: f32,
    pub y: f32,
    pub x_velocity: f32,
    pub y_velocity: f32,
    pub width: f32,
    pub height: f32,
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

fn objs_overlap(a: PhysObj, b: PhysObj) -> bool {
    a.x - a.width < b.x + b.width
        && a.x + a.width > b.x - b.width
        && a.y - a.height < b.y + b.height
        && a.y + a.height > b.y - b.height
}

fn spawn_obj(
    last_spawn_time: &mut Option<Instant>,
    now: Instant,
    cooldown: Duration,
    next_obj: PhysObj,
    obj_list: &mut Vec<PhysObj>,
) -> bool {
    if match *last_spawn_time {
        None => Duration::MAX,
        Some(time) => now.duration_since(time),
    } > cooldown
    {
        *last_spawn_time = Some(now);
        (*obj_list).push(next_obj);
        return true;
    }
    return false;
}

pub fn tick(game_state: &mut GameState, now: Instant, time_delta: f32) {
    // update positions
    game_state.birdy.position_delta(time_delta);
    for rock in game_state.rocks.iter_mut() {
        rock.position_delta(time_delta);
    }
    for coin in game_state.coins.iter_mut() {
        coin.position_delta(time_delta);
    }

    // update player velocity for next frame
    game_state.birdy.x_velocity = if game_state.birdy.x_velocity.is_sign_positive() {
        f32::max(
            game_state.birdy.x_velocity + (birdy::DECCEL_MOVE * time_delta),
            0.0,
        )
    } else {
        f32::min(
            game_state.birdy.x_velocity - (birdy::DECCEL_MOVE * time_delta),
            0.0,
        )
    };
    game_state.birdy.y_velocity = f32::max(
        game_state.birdy.y_velocity + birdy::ACCEL_GRAV * time_delta,
        birdy::TERMINAL_VELOCITY,
    );
    for key in game_state.keys.iter() {
        match key {
            VirtualKeyCode::Space => {
                if match game_state.last_jump_time {
                    None => Duration::MAX,
                    Some(time) => now.duration_since(time),
                } > birdy::JUMP_COOLDOWN
                {
                    game_state.last_jump_time = Some(now);
                    game_state.birdy.y_velocity = birdy::ACCEL_JUMP;
                }
            }
            VirtualKeyCode::Left => game_state.birdy.x_velocity = birdy::ACCEL_MOVE * -1.0,
            VirtualKeyCode::Right => game_state.birdy.x_velocity = birdy::ACCEL_MOVE,
            _ => (),
        }
    }

    // rock spawning and despawning
    if spawn_obj(
        &mut game_state.last_rock_spawn_time,
        now,
        rock::COOLDOWN,
        game_state.next_rock,
        &mut game_state.rocks,
    ) {
        game_state.rock_fall_direction *= -1.0;
        game_state.next_rock = rock::new_rock(game_state.rock_fall_direction);
    }
    despawn_objs(&mut game_state.rocks);

    // coin spawning and despawning
    if spawn_obj(
        &mut game_state.last_coin_spawn_time,
        now,
        coin::COOLDOWN,
        game_state.next_coin,
        &mut game_state.coins,
    ) {
        game_state.next_coin = coin::new_coin();
    }
    despawn_objs(&mut game_state.coins);

    // birdy-playfield edge collision
    if game_state.birdy.x - game_state.birdy.width < -1.0 {
        game_state.birdy.x = -1.0 + game_state.birdy.width;
    }
    if game_state.birdy.x + game_state.birdy.width > 1.0 {
        game_state.birdy.x = 1.0 - game_state.birdy.width;
    }
    if game_state.birdy.y - game_state.birdy.height < -1.0 {
        game_state.birdy.y = -1.0 + game_state.birdy.height;
        game_state.birdy.y_velocity *= PLAYFIELD_BOUNCE_COEFFICIENT; // bounce
    }
    if game_state.birdy.y + game_state.birdy.height > 1.0 {
        game_state.birdy.y = 1.0 - game_state.birdy.height;
    }

    // birdy-rock collision
    let mut i = 0;
    while i < game_state.rocks.len() {
        let rock = game_state.rocks[i];
        if objs_overlap(game_state.birdy, rock) {
            game_state.dead = true;
            return; // this round is over, no point in doing anything else
        }
        i += 1;
    }

    // birdy-coin collision
    let mut i = 0;
    while i < game_state.coins.len() {
        let coin = game_state.coins[i];
        if objs_overlap(game_state.birdy, coin) {
            game_state.score += 1;
            game_state.coins.remove(i);
        } else {
            i += 1;
        }
    }
}
