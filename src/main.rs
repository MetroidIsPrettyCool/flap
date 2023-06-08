use glium::Surface;

const BIRDY_ACCEL_GRAV: f32 = -0.9;
const BIRDY_ACCEL_JUMP: f32 = 0.8;
const BIRDY_ACCEL_MOVE: f32 = 0.6;
const BIRDY_DECCEL_MOVE: f32 = -0.4; // decceleration applied when player isn't holding a direction key
const BIRDY_JUMP_COOLDOWN: std::time::Duration = std::time::Duration::from_millis(250);
const BIRDY_TERMINAL_VELOCITY: f32 = -1.0;
const BIRDY_COLOR: (f32, f32, f32) = (1.0, 0.9, 0.0);
const BIRDY_DEPTH: f32 = 0.0;
const BIRDY_SIZE: f32 = 0.1;

const ROCK_COOLDOWN: std::time::Duration = std::time::Duration::from_millis(750);
const ROCK_DESPAWN_DIST: f32 = 2.0;
const ROCK_MIN_VELOCITY: f32 = 0.5;
const ROCK_MAX_VELOCITY: f32 = 1.0;
const ROCK_SPAWN_DIST: f32 = 1.5;
const ROCK_COLOR: (f32, f32, f32) = (0.2, 0.2, 0.3);
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
    (0.0, 1.0, 0.5)
);

const WINDOW_INITIAL_WIDTH: u32 = 1024;
const WINDOW_INITIAL_HEIGHT: u32 = 768;


#[derive(Copy, Clone)]
struct Vert {
    position: (f32, f32, f32),
    color: (f32, f32, f32),
}
glium::implement_vertex!(Vert, position, color);

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
fn diamond_from_dims(width: f32, height: f32, depth: f32, center: (f32, f32), color: (f32, f32, f32)) -> Quad {
    let center_x = center.0;
    let center_y = center.1;
    quad_from_verts(
	Vert {
	    position: (center_x,         center_y + height, depth),
	    color: color,
	},
	Vert {
	    position: (center_x + width, center_y,          depth),
	    color: color,
	},
	Vert {
	    position: (center_x,         center_y - height, depth),
	    color: color,
	},
	Vert {
	    position: (center_x - width, center_y,          depth),
	    color: color,
	}
    )
}
fn _square_from_dims(width: f32, height: f32, depth: f32, center: (f32, f32), color: (f32, f32, f32)) -> Quad {
    let center_x = center.0;
    let center_y = center.1;
    quad_from_verts(
	Vert {
	    position: (center_x - width, center_y + height, depth),
	    color: color,
	},
	Vert {
	    position: (center_x + width, center_y + height, depth),
	    color: color,
	},
	Vert {
	    position: (center_x + width, center_y - height, depth),
	    color: color,
	},
	Vert {
	    position: (center_x - width, center_y - height, depth),
	    color: color,
	}
    )
}
const fn square_from_edge_positions(left: f32, right: f32, top: f32, bottom: f32, depth: f32, color: (f32, f32, f32)) -> Quad {
    quad_from_verts(
	Vert {
	    position: (left,  top,    depth),
	    color: color,
	},
	Vert {
	    position: (right, top,    depth),
	    color: color,
	},
	Vert {
	    position: (right, bottom, depth),
	    color: color,
	},
	Vert {
	    position: (left,  bottom, depth),
	    color: color,
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
	.with_depth_buffer(24);
    let disp = glium::Display::new(win_b, cntxt_b, &eve_lp)?;

    // compile and link shaders
    let shdr = glium::program::Program::from_source(
	&disp,
	&std::fs::read_to_string("./src/vert.glsl")?,
	&std::fs::read_to_string("./src/frag.glsl")?,
	None
    )?;

    // gameplay variables
    let mut last_jump_time = None;
    let mut last_rock_spawn_time = None;
    let mut rock_fall_dir = 1.0;
    let mut birdy = PhysObj {
	position: (0.0, 0.0),
	velocity: (0.0, 0.0),
	size: BIRDY_SIZE,
    };

    let mut rocks = Vec::new();

    // main loop
    eve_lp.run(move |eve, _, ctrl_flow| {
	// time-wimey stuff
	let now = std::time::Instant::now();
	let time_delta = now.duration_since(then).as_secs_f32();
	next_frame_time += std::time::Duration::from_nanos(16_666_667); // next frame should render 1/60 sec later
	*ctrl_flow = glium::glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

	// handle events
	match eve {
	    glium::glutin::event::Event::WindowEvent {
		event: glium::glutin::event::WindowEvent::CloseRequested,
		..
	    } => *ctrl_flow = glium::glutin::event_loop::ControlFlow::ExitWithCode(0),
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
		    is_synthetic: false, // true indicates key was already pressed when window gained focus
		    ..
		},
		..
	    } => match keycode {
		glium::glutin::event::VirtualKeyCode::Space => if match last_jump_time {
		    None => std::time::Duration::MAX,
		    Some(time) => now.duration_since(time)
		} > BIRDY_JUMP_COOLDOWN {
		    last_jump_time = Some(now);
		    birdy.velocity.1 = BIRDY_ACCEL_JUMP;
		},
		glium::glutin::event::VirtualKeyCode::Left => birdy.velocity.0 = BIRDY_ACCEL_MOVE * -1.0,
		glium::glutin::event::VirtualKeyCode::Right => birdy.velocity.0 = BIRDY_ACCEL_MOVE,
		_ => (),
	    },
	    _ => (),
	}

	// logic

	// player stuff
	birdy.position_delta(time_delta);
	if birdy.position.0 - birdy.size < -1.0 {
	    birdy.position.0 = -1.0 + birdy.size;
	}
	if birdy.position.0 + birdy.size > 1.0 {
	    birdy.position.0 = 1.0 - birdy.size;
	}
	if birdy.position.1 - birdy.size < -1.0 {
	    birdy.position.1 = -1.0 + birdy.size;
	    birdy.velocity.1 *= PLAYFIELD_BOUNCE_COEFFICIENT; // bounce
	}
	if birdy.position.1 + birdy.size > 1.0 {
	    birdy.position.1 = 1.0 - birdy.size;
	}
	birdy.velocity.1 += BIRDY_ACCEL_GRAV * time_delta;
	if birdy.velocity.1 < BIRDY_TERMINAL_VELOCITY {
	    birdy.velocity.1 = BIRDY_TERMINAL_VELOCITY;
	}
	let tmp = birdy.velocity.0;
	birdy.velocity.0 += (BIRDY_DECCEL_MOVE * time_delta) * f32::signum(birdy.velocity.0);
	if tmp.is_sign_positive() != birdy.velocity.0.is_sign_positive() {
	    birdy.velocity.0 = 0.0;
	}

	// rock stuff
	if match last_rock_spawn_time {
	    None => std::time::Duration::MAX,
	    Some(time) => now.duration_since(time)
	} > ROCK_COOLDOWN {
	    last_rock_spawn_time = Some(now);
	    let size = rand_range(ROCK_MIN_SIZE, ROCK_MAX_SIZE);
	    let mut x = rand::random::<f32>() * (1.0 - size);
	    if rand::random() {
		x *= -1.0;
	    }
	    rocks.push(
		PhysObj {
		    position: (x, ROCK_SPAWN_DIST * rock_fall_dir),
		    velocity: (0.0, rock_fall_dir * -1.0 * rand_range(ROCK_MIN_VELOCITY, ROCK_MAX_VELOCITY)),
		    size: size,
		}
	    );
	    rock_fall_dir *= -1.0;
	}
	for rock in rocks.iter_mut() {
	    rock.position_delta(time_delta);
	}
	let mut i = 0;
	while i < rocks.len() {
	    if f32::abs(rocks[i].position.1) > ROCK_DESPAWN_DIST {
		rocks.remove(i);
	    }
	    else {
		i += 1;
	    }
	}

	// get all our vertices together
	let mut vertices = Vec::new();
	vertices.extend_from_slice(
	    &diamond_from_dims(birdy.size, birdy.size, BIRDY_DEPTH, birdy.position, BIRDY_COLOR)
	);
	for rock in rocks.iter() { // rocks, duh
	    vertices.extend_from_slice(
		&diamond_from_dims(rock.size, rock.size, ROCK_DEPTH, rock.position, ROCK_COLOR)
	    );
	}

	vertices.extend_from_slice(&PLAYFIELD_MODEL); // playfield

	// render to framebuffer
	let mut f_buff = disp.draw(); // next framebuffer
	f_buff.clear( // clear the framebuffer
	    None, // rect
	    Some((0.0, 0.0, 0.0, 1.0)), // color
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
