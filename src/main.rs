use glium::Surface;

#[derive(Copy, Clone)]
struct Vert {
    pos: (f32, f32),
    color: (f32, f32, f32),
}
glium::implement_vertex!(Vert, pos, color);

#[derive(Clone)]
struct Bird {
    pos: f32,
    model: Vec<Vert>,
}
impl Bird {
    fn new () -> Bird {
	Bird {
	    pos: 0.0,
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
	}
    }
    // obviouly need to repalce with a better, translation matrix-based approach
    fn draw (&self) -> Vec<Vert> {
	let mut model = self.model.clone();
	for mut vrtx in model.iter_mut() {
	    (*vrtx).pos.0 += self.pos;
	}
	model
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // default window size
    let mut _win_w = 1024;
    let mut _win_h = 768;

    // initial setup
    let eve_lp = glium::glutin::event_loop::EventLoop::new();
    let win_b = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(_win_w, _win_h))
        .with_title("flap");
    let cntxt_b = glium::glutin::ContextBuilder::new();
    let disp = glium::Display::new(win_b, cntxt_b, &eve_lp)?;

    // shaders
    let vert_shdr = std::fs::read_to_string("./vert.glsl")?;
    let frag_shdr = std::fs::read_to_string("./frag.glsl")?;
    let shdr = glium::program::Program::from_source(&disp, &vert_shdr, &frag_shdr, None)?;

    // gameplay variables
    let mut birdy = Bird::new();

    // main loop
    eve_lp.run(move |eve, _, ctrl_flow| {
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

	// handle events
	match eve {
	    glium::glutin::event::Event::WindowEvent{event: glium::glutin::event::WindowEvent::CloseRequested, ..} => *ctrl_flow = glium::glutin::event_loop::ControlFlow::ExitWithCode(0),
	    glium::glutin::event::Event::WindowEvent{event: glium::glutin::event::WindowEvent::Resized(win_size), ..} => glium::glutin::dpi::PhysicalSize{width: _win_w, height: _win_h} = win_size,
	    _ => (),
	}
    });
}
