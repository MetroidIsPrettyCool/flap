use glium::Surface;

const BIRDY_ACCEL_GRAV: f32 = -0.9;
const BIRDY_ACCEL_JUMP: f32 = 0.8;
const BIRDY_ACCEL_MOVE: f32 = 0.6;
const BIRDY_DECCEL_MOVE: f32 = -0.4; // decceleration applied when player isn't holding a direction key
const BIRDY_JUMP_COOLDOWN: std::time::Duration = std::time::Duration::from_millis(250);
const BIRDY_TERMINAL_VELOCITY: f32 = -1.0;
const BIRDY_DEPTH: f32 = 0.0;
const BIRDY_SIZE: f32 = 0.1;

const ROCK_COOLDOWN: std::time::Duration = std::time::Duration::from_millis(750);
const ROCK_DESPAWN_DIST: f32 = 2.0;
const ROCK_MIN_VELOCITY: f32 = 0.5;
const ROCK_MAX_VELOCITY: f32 = 1.0;
const ROCK_SPAWN_DIST: f32 = 1.5;
const ROCK_DEPTH: f32 = 0.1;
const ROCK_MIN_SIZE: f32 = 0.05;
const ROCK_MAX_SIZE: f32 = 0.2;

const PLAYFIELD_BOUNCE_COEFFICIENT: f32 = -0.5; // portion of player's velocity to reflect when they collide with the bottom of the playfield.
const PLAYFIELD_MODEL: Quad = square_from_edge_positions(
    -1.0,
    1.0,
    1.0,
    -1.0,
    0.6,
    ((0.0, 0.0), (8.0 / 64.0, 8.0 / 64.0))
);

const WINDOW_INITIAL_WIDTH: u32 = 1024;
const WINDOW_INITIAL_HEIGHT: u32 = 768;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_from_edge_positions() {
        assert_eq!(
	    [
		Vert { // rt
		    position: (1.0, 1.0, 0.6),
		    texture_coordinates: (0.125, 0.0),
		},
		Vert { // lt
		    position: (-1.0, 1.0, 0.6),
		    texture_coordinates: (0.0, 0.0),
		},
		Vert { // lb
		    position: (-1.0, -1.0, 0.6),
		    texture_coordinates: (0.0, 0.125),
		},
		Vert { // rt
		    position: (1.0, 1.0, 0.6),
		    texture_coordinates: (0.125, 0.0),
		},
		Vert { // lb
		    position: (-1.0, -1.0, 0.6),
		    texture_coordinates: (0.0, 0.125),
		},
		Vert { // rb
		    position: (1.0, -1.0, 0.6),
		    texture_coordinates: (0.125, 0.125),
		},
	    ],
	    PLAYFIELD_MODEL
	);
    }

    #[test]
    fn square_from_dims() {
        assert_eq!(
	    [
		Vert { // rt
		    position: (1.0, 1.0, 0.6),
		    texture_coordinates: (1.0, 0.0),
		},
		Vert { // lt
		    position: (-1.0, 1.0, 0.6),
		    texture_coordinates: (0.0, 0.0),
		},
		Vert { // lb
		    position: (-1.0, -1.0, 0.6),
		    texture_coordinates: (0.0, 1.0),
		},
		Vert { // rt
		    position: (1.0, 1.0, 0.6),
		    texture_coordinates: (1.0, 0.0),
		},
		Vert { // lb
		    position: (-1.0, -1.0, 0.6),
		    texture_coordinates: (0.0, 1.0),
		},
		Vert { // rb
		    position: (1.0, -1.0, 0.6),
		    texture_coordinates: (1.0, 1.0),
		},
	    ],
	    super::square_from_dims(
		1.0,
		1.0,
		0.6,
		(0.0, 0.0),
		((0.0, 0.0), (1.0, 1.0))
	    )
	);
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Vert {
    position: (f32, f32, f32),
    texture_coordinates: (f32, f32),
}
glium::implement_vertex!(Vert, position, texture_coordinates);

#[derive(Copy, Clone)]
struct PhysObj {
    position: (f32, f32),
    velocity: (f32, f32),
    size: f32,
}
impl PhysObj {
    fn position_delta(&mut self, time_delta: f32) {
	(*self).position.0 += (*self).velocity.0 * time_delta;
	(*self).position.1 += (*self).velocity.1 * time_delta;
    }
}

#[derive(Clone)]
struct GameState {
    last_jump_time: Option<std::time::Instant>,
    last_rock_spawn_time: Option<std::time::Instant>,
    rock_fall_direction: f32,
    birdy: PhysObj,
    rocks: Vec<PhysObj>,
    dead: bool,
}
impl GameState {
    fn new () -> GameState {
	GameState {
	    last_jump_time: None,
	    last_rock_spawn_time: None,
	    rock_fall_direction: 1.0,
	    birdy: PhysObj {
		position: (0.0, 0.0),
		velocity: (0.0, 0.0),
		size: BIRDY_SIZE,
	    },
	    rocks: Vec::new(),
	    dead: false,
	}
    }
}

fn rand_range(min: f32, max: f32) -> f32 {
    rand::random::<f32>() * (max - min) + min
}

type Quad = [Vert; 6];
const fn quad_from_verts(lt: Vert, rt: Vert, rb: Vert, lb: Vert) -> Quad {
    [
	rt, lt, lb,
	rt, lb, rb
    ]
}
fn square_from_dims(width: f32, height: f32, depth: f32, center: (f32, f32), texture_coordinates: ((f32, f32), (f32, f32))) -> Quad {
    let center_x = center.0;
    let center_y = center.1;
    quad_from_verts(
	Vert {
	    position: (center_x - width, center_y + height, depth),
	    texture_coordinates: (texture_coordinates.0.0, texture_coordinates.0.1),
	},
	Vert {
	    position: (center_x + width, center_y + height, depth),
	    texture_coordinates: (texture_coordinates.1.0, texture_coordinates.0.1),
	},
	Vert {
	    position: (center_x + width, center_y - height, depth),
	    texture_coordinates: (texture_coordinates.1.0, texture_coordinates.1.1),
	},
	Vert {
	    position: (center_x - width, center_y - height, depth),
	    texture_coordinates: (texture_coordinates.0.0, texture_coordinates.1.1),
	}
    )
}
const fn square_from_edge_positions(left: f32, right: f32, top: f32, bottom: f32, depth: f32, texture_coordinates: ((f32, f32), (f32, f32))) -> Quad {
    quad_from_verts(
	Vert {
	    position: (left,  top,    depth),
	    texture_coordinates: (texture_coordinates.0.0, texture_coordinates.0.1),
	},
	Vert {
	    position: (right, top,    depth),
	    texture_coordinates: (texture_coordinates.1.0, texture_coordinates.0.1),
	},
	Vert {
	    position: (right, bottom, depth),
	    texture_coordinates: (texture_coordinates.1.0, texture_coordinates.1.1),
	},
	Vert {
	    position: (left,  bottom, depth),
	    texture_coordinates: (texture_coordinates.0.0, texture_coordinates.1.1),
	}
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // renderer variables
    let mut next_frame_time = std::time::Instant::now();
    let mut then = std::time::Instant::now();
    let mut window_aspect_ratio = WINDOW_INITIAL_WIDTH as f32 / WINDOW_INITIAL_HEIGHT as f32;
    let mut frame_times = Vec::new();

    // initial setup
    let eve_lp = glium::glutin::event_loop::EventLoop::new();
    let win_b = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(WINDOW_INITIAL_WIDTH, WINDOW_INITIAL_HEIGHT))
        .with_title("flap");
    let cntxt_b = glium::glutin::ContextBuilder::new()
	.with_depth_buffer(24)
	.with_pixel_format(8, 8)
	.with_multisampling(8);
    // .with_srgb(false);
    let disp = glium::Display::new(win_b, cntxt_b, &eve_lp)?;

    // compile and link shaders
    let shdr = glium::program::Program::from_source(
	&disp,
	&std::fs::read_to_string("./res/vert.glsl")?,
	&std::fs::read_to_string("./res/frag.glsl")?,
	None
    )?;

    // Load texture atlas
    let decoder = png::Decoder::new(std::fs::File::open("./res/atlas.png")?);
    let mut reader = decoder.read_info()?;
    let mut texture_atlas = vec![0; reader.output_buffer_size()];
    let image_info = reader.next_frame(&mut texture_atlas)?;
    let texture_atlas = glium::texture::srgb_texture2d::SrgbTexture2d::new(
	&disp,
	glium::texture::RawImage2d {
	    data: std::borrow::Cow::from(&texture_atlas),
	    width: image_info.width,
	    height: image_info.height,
	    format: glium::texture::ClientFormat::U8U8U8U8,
	}
    )?;

    let mut game_state = GameState::new();

    // main loop
    eve_lp.run(move |eve, _, ctrl_flow| {
	// time-wimey stuff
	let now = std::time::Instant::now();
	let time_delta = now.duration_since(then).as_secs_f32();
	next_frame_time += std::time::Duration::from_nanos(16_666_667); // next frame should render 1/60 sec later
	*ctrl_flow = glium::glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

	// upkeep
	if game_state.dead {
	    game_state = GameState::new();
	}

	// handle events
	match eve {
	    glium::glutin::event::Event::WindowEvent {
		event: glium::glutin::event::WindowEvent::CloseRequested,
		..
	    } => {
		*ctrl_flow = glium::glutin::event_loop::ControlFlow::ExitWithCode(0);
		return;
	    },
	    glium::glutin::event::Event::WindowEvent {
		event: glium::glutin::event::WindowEvent::Resized(size),
		..
	    } => {
		window_aspect_ratio = size.width as f32 / size.height as f32;
	    },
	    glium::glutin::event::Event::WindowEvent {
		event: glium::glutin::event::WindowEvent::KeyboardInput {
		    input: glium::glutin::event::KeyboardInput {
			state: glium::glutin::event::ElementState::Pressed,
			virtual_keycode: Some(keycode),
			..
		    },
		    ..
		},
		..
	    } => match keycode {
		glium::glutin::event::VirtualKeyCode::Space => if match game_state.last_jump_time {
		    None => std::time::Duration::MAX,
		    Some(time) => now.duration_since(time)
		} > BIRDY_JUMP_COOLDOWN {
		    game_state.last_jump_time = Some(now);
		    game_state.birdy.velocity.1 = BIRDY_ACCEL_JUMP;
		},
		glium::glutin::event::VirtualKeyCode::Left => game_state.birdy.velocity.0 = BIRDY_ACCEL_MOVE * -1.0,
		glium::glutin::event::VirtualKeyCode::Right => game_state.birdy.velocity.0 = BIRDY_ACCEL_MOVE,
		_ => (),
	    },
	    _ => (),
	}

	// logic

	// player stuff
	game_state.birdy.position_delta(time_delta);
	if game_state.birdy.position.0 - game_state.birdy.size < -1.0 {
	    game_state.birdy.position.0 = -1.0 + game_state.birdy.size;
	}
	if game_state.birdy.position.0 + game_state.birdy.size > 1.0 {
	    game_state.birdy.position.0 = 1.0 - game_state.birdy.size;
	}
	if game_state.birdy.position.1 - game_state.birdy.size < -1.0 {
	    game_state.birdy.position.1 = -1.0 + game_state.birdy.size;
	    game_state.birdy.velocity.1 *= PLAYFIELD_BOUNCE_COEFFICIENT; // bounce
	}
	if game_state.birdy.position.1 + game_state.birdy.size > 1.0 {
	    game_state.birdy.position.1 = 1.0 - game_state.birdy.size;
	}
	game_state.birdy.velocity.1 += BIRDY_ACCEL_GRAV * time_delta;
	if game_state.birdy.velocity.1 < BIRDY_TERMINAL_VELOCITY {
	    game_state.birdy.velocity.1 = BIRDY_TERMINAL_VELOCITY;
	}
	let tmp = game_state.birdy.velocity.0;
	game_state.birdy.velocity.0 += (BIRDY_DECCEL_MOVE * time_delta) * f32::signum(game_state.birdy.velocity.0);
	if tmp.is_sign_positive() != game_state.birdy.velocity.0.is_sign_positive() {
	    game_state.birdy.velocity.0 = 0.0;
	}

	// rock stuff
	if match game_state.last_rock_spawn_time {
	    None => std::time::Duration::MAX,
	    Some(time) => now.duration_since(time)
	} > ROCK_COOLDOWN {
	    game_state.last_rock_spawn_time = Some(now);
	    let size = rand_range(ROCK_MIN_SIZE, ROCK_MAX_SIZE);
	    let mut x = rand::random::<f32>() * (1.0 - size);
	    if rand::random() {
		x *= -1.0;
	    }
	    game_state.rocks.push(
		PhysObj {
		    position: (x, ROCK_SPAWN_DIST * game_state.rock_fall_direction),
		    velocity: (0.0, game_state.rock_fall_direction * -1.0 * rand_range(ROCK_MIN_VELOCITY, ROCK_MAX_VELOCITY)),
		    size: size,
		}
	    );
	    game_state.rock_fall_direction *= -1.0;
	}
	for rock in game_state.rocks.iter_mut() {
	    rock.position_delta(time_delta);
	}
	let mut i = 0;
	while i < game_state.rocks.len() {
	    if f32::abs(game_state.rocks[i].position.1) > ROCK_DESPAWN_DIST {
		game_state.rocks.remove(i);
	    }
	    else {
		i += 1;
	    }
	}

	// birdy-rock collision
	for rock in game_state.rocks.iter() {
	    if f32::abs(rock.position.1) <= 1.0 + rock.size &&
		rock.position.0 - rock.size < game_state.birdy.position.0 + game_state.birdy.size &&
		rock.position.0 + rock.size > game_state.birdy.position.0 - game_state.birdy.size &&
		rock.position.1 - rock.size < game_state.birdy.position.1 + game_state.birdy.size &&
		rock.position.1 + rock.size > game_state.birdy.position.1 - game_state.birdy.size {
		    game_state.dead = true;
		    break;
		}
	}

	// get all our vertices together
	let mut vertices = Vec::new();
	vertices.extend_from_slice(
	    &square_from_dims(game_state.birdy.size,
			      game_state.birdy.size,
			      BIRDY_DEPTH,
			      game_state.birdy.position,
			      ((8.0 / 64.0, 0.0 / 64.0), (16.0 / 64.0, 8.0 / 64.0)))
	);
	for rock in game_state.rocks.iter() { // rocks, duh
	    if rock.velocity.1.is_sign_positive() {
		vertices.extend_from_slice(
		    &square_from_dims(rock.size, rock.size, ROCK_DEPTH, rock.position, ((16.0 / 64.0, 32.0 / 64.0), (32.0 / 64.0, 48.0 / 64.0)))
		);
	    }
	    else {
		vertices.extend_from_slice(
		    &square_from_dims(rock.size, rock.size, ROCK_DEPTH, rock.position, ((0.0 / 64.0, 32.0 / 64.0), (16.0 / 64.0, 48.0 / 64.0)))
		);
	    }
	}
	vertices.extend_from_slice(&PLAYFIELD_MODEL); // playfield

	// render to framebuffer
	let mut f_buff = disp.draw(); // next framebuffer
	f_buff.clear( // clear the framebuffer
	    None, // rect
	    Some((0.0, 0.0, 0.0, 0.0)), // color
	    true, // color_srgb
	    Some(f32::MAX), // depth
	    None // stencil
	);
	f_buff.draw( // draw vertices to framebuffer
	    &glium::VertexBuffer::new(
		&disp,
		&vertices,
	    ).unwrap(),
	    &glium::index::NoIndices( // not using indexed rendering
		glium::index::PrimitiveType::TrianglesList // polygon type
	    ),
	    &shdr, // shader program
	    &glium::uniform! {
		window_aspect_ratio: window_aspect_ratio,
		texture_atlas: texture_atlas.sampled()
		    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
	    },
	    &glium::DrawParameters { // draw parameters
		depth: glium::Depth {
		    test: glium::draw_parameters::DepthTest::IfLess,
		    write: true,
		    .. Default::default()
		},
		.. Default::default()
	    }
	).unwrap();
	f_buff.finish().unwrap(); // swap framebuffers

	// more timey-wimey
	then = now;
	frame_times.push(now.elapsed());
	if frame_times.len() == 60 {
	    let mut total_frame_time = std::time::Duration::ZERO;
	    for frame_time in frame_times.iter() {
		total_frame_time += *frame_time;
	    }
	    println!("total: {:?}", total_frame_time);
	    println!("avg: {:?}", total_frame_time / 60);
	    println!("");
	    frame_times = Vec::new();
	}
    });
}
