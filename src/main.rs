use glium::Surface;

const JUMP_COOLDOWN: std::time::Duration = std::time::Duration::from_millis(250);
const ACCEL_GRAV: f32 = -0.009;
const TERM_VEL: f32 = -0.01;

#[derive(Copy, Clone)]
struct Vert {
    pos: (f32, f32),
    color: (f32, f32, f32),
}
glium::implement_vertex!(Vert, pos, color);

#[derive(Clone)]
struct Bird {
    pos: (f32, f32),
    velocity: (f32, f32),
    model: Vec<Vert>,
    hit_box: ((f32, f32), (f32, f32)),
}
impl Bird {
    fn new () -> Bird {
	Bird {
	    pos: (0.0, 0.0),
	    velocity: (0.0, 0.0),
	    model: vec![
		Vert {
		    pos: (0.1, 0.0),
		    color: (1.0, 0.9, 0.0),
		},
		Vert {
		    pos: (0.0, 0.1),
		    color: (1.0, 0.9, 0.0),
		},
		Vert {
		    pos: (-0.1, 0.0),
		    color: (1.0, 0.9, 0.0),
		},
		Vert {
		    pos: (0.1, 0.0),
		    color: (1.0, 0.9, 0.0),
		},
		Vert {
		    pos: (-0.1, 0.0),
		    color: (1.0, 0.9, 0.0),
		},
		Vert {
		    pos: (0.0, -0.1),
		    color: (1.0, 0.9, 0.0),
		},
	    ],
	    hit_box: ((-0.1, -0.1), (0.1, 0.1)),
	}
    }
    // obviouly need to repalce with a better, translation matrix-based approach
    fn draw (&self) -> Vec<Vert> {
	let mut model = self.model.clone();
	for mut vrtx in model.iter_mut() {
	    (*vrtx).pos.0 += self.pos.0; // x
	    (*vrtx).pos.1 += self.pos.1; // y
	}
	model
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // renderer variables
    let mut next_frame_time = std::time::Instant::now();
    let mut then = std::time::Instant::now();
    let mut _win_size_px = (1024, 768);

    // initial setup
    let eve_lp = glium::glutin::event_loop::EventLoop::new();
    let win_b = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(_win_size_px.0, _win_size_px.1))
        .with_title("flap");
    let cntxt_b = glium::glutin::ContextBuilder::new();
    let disp = glium::Display::new(win_b, cntxt_b, &eve_lp)?;

    // shaders
    let vert_shdr = std::fs::read_to_string("./vert.glsl")?;
    let frag_shdr = std::fs::read_to_string("./frag.glsl")?;
    let shdr = glium::program::Program::from_source(&disp, &vert_shdr, &frag_shdr, None)?;

    // gameplay variables
    let mut last_jump_time = None;
    let mut birdy = Bird::new();

    // main loop
    eve_lp.run(move |eve, _, ctrl_flow| {
	// time-wimey stuff
	let now = std::time::Instant::now();
	let time_delta = now.duration_since(then);
	next_frame_time += std::time::Duration::from_nanos(16_666_667); // next frame should render 1/60 sec later
	*ctrl_flow = glium::glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

	// handle events
	match eve {
	    glium::glutin::event::Event::WindowEvent {
		event: glium::glutin::event::WindowEvent::CloseRequested,
		..
	    } => *ctrl_flow = glium::glutin::event_loop::ControlFlow::ExitWithCode(0),
	    glium::glutin::event::Event::WindowEvent {
		event: glium::glutin::event::WindowEvent::Resized(win_size),
		..
	    } => glium::glutin::dpi::PhysicalSize{width: _win_size_px.0, height: _win_size_px.1} = win_size,
	    glium::glutin::event::Event::WindowEvent {
		event: glium::glutin::event::WindowEvent::KeyboardInput {
		    input: glium::glutin::event::KeyboardInput {
			state: glium::glutin::event::ElementState::Pressed,
			virtual_keycode: Some(glium::glutin::event::VirtualKeyCode::Space),
			..
		    },
		    is_synthetic: false, // true indicates key was already pressed when window gainend focus
		    ..
		},
		..
	    } => if match last_jump_time {
		None => std::time::Duration::MAX,
		Some(time) => now.duration_since(time)
	    } > JUMP_COOLDOWN {
		last_jump_time = Some(now);
		birdy.velocity.1 = 0.004;
	    },
	    _ => (),
	}

	// logic
	birdy.velocity.1 += ACCEL_GRAV * time_delta.as_secs_f32();
	if birdy.velocity.1 < TERM_VEL {
	    birdy.velocity.1 = TERM_VEL;
	}
	birdy.pos.0 += birdy.velocity.0;
	if birdy.pos.0 + birdy.hit_box.0.0 < -1.0 {
	    birdy.pos.0 = -1.0 - birdy.hit_box.0.0;
	}
	if birdy.pos.0 + birdy.hit_box.1.0 > 1.0 {
	    birdy.pos.0 = 1.0 - birdy.hit_box.1.0;
	}
	birdy.pos.1 += birdy.velocity.1;
	if birdy.pos.1 + birdy.hit_box.0.1< -1.0 {
	    birdy.pos.1 = -1.0 - birdy.hit_box.0.1;
	    birdy.velocity.1 *= -0.5 // bounce
	}
	if birdy.pos.1 + birdy.hit_box.1.1 > 1.0 {
	    birdy.pos.1 = 1.0 - birdy.hit_box.1.1;
	}

	// render to framebuffer
	let mut f_buff = disp.draw(); // next framebuffer
	f_buff.clear( // clear the framebuffer
	    None, // rect
	    Some((0.0, 1.0, 0.5, 1.0)), // color
	    true, // color_srgb
	    None, // depth
	    None // stencil
	);
	f_buff.draw( // draw birdy to framebuffer
	    &glium::VertexBuffer::new( // vertices
		&disp,
		&birdy.draw()
	    ).unwrap(),
	    &glium::index::NoIndices( // not using indexed rendering
		glium::index::PrimitiveType::TrianglesList // polygon type
	    ),
	    &shdr, // shader program
	    &glium::uniforms::EmptyUniforms, // no uniforms
	    &Default::default() // default draw parameters
	).unwrap();
	f_buff.finish().unwrap(); // swap framebuffers

	// more timey-wimey
	then = now;
    });
}
