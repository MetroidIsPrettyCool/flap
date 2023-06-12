mod logic;
mod render;

use crate::logic::PhysObj;

const WINDOW_INITIAL_WIDTH: u32 = 1024;
const WINDOW_INITIAL_HEIGHT: u32 = 768;

#[derive(Clone)]
pub struct GameState {
    pub last_jump_time: Option<std::time::Instant>,
    pub last_rock_spawn_time: Option<std::time::Instant>,
    pub rock_fall_direction: f32,
    pub last_coin_spawn_time: Option<std::time::Instant>,
    pub birdy: PhysObj,
    pub rocks: Vec<PhysObj>,
    pub coins: Vec<PhysObj>,
    pub score: u32,
    pub keys: Vec<glium::glutin::event::VirtualKeyCode>,
    pub dead: bool,
}
impl GameState {
    pub fn new() -> GameState {
        GameState {
            last_jump_time: None,
            last_rock_spawn_time: None,
            rock_fall_direction: 1.0,
            last_coin_spawn_time: None,
            birdy: PhysObj {
		x: 0.0,
		y: 0.0,
		x_velocity: 0.0,
		y_velocity: 0.0,
                size: logic::birdy::SIZE,
            },
            rocks: Vec::new(),
            coins: Vec::new(),
            score: 0,
	    keys: Vec::new(),
	    dead: false,
        }
    }
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
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(
            WINDOW_INITIAL_WIDTH,
            WINDOW_INITIAL_HEIGHT,
        ))
        .with_title("flap");
    let cntxt_b = glium::glutin::ContextBuilder::new()
        .with_depth_buffer(24);
    let disp = glium::Display::new(win_b, cntxt_b, &eve_lp)?;

    // compile and link shaders
    let shdr = glium::program::Program::from_source(
        &disp,
        &std::fs::read_to_string("./res/vert.glsl")?,
        &std::fs::read_to_string("./res/frag.glsl")?,
        None,
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
        },
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
            }
            glium::glutin::event::Event::WindowEvent {
                event: glium::glutin::event::WindowEvent::Resized(size),
                ..
            } => {
                window_aspect_ratio = size.width as f32 / size.height as f32;
            }
            glium::glutin::event::Event::WindowEvent {
                event:
                    glium::glutin::event::WindowEvent::KeyboardInput {
                        input:
                            glium::glutin::event::KeyboardInput {
                                state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => match state {
		glium::glutin::event::ElementState::Pressed => game_state.keys.push(keycode),
		glium::glutin::event::ElementState::Released => game_state.keys.retain(|&key| key != keycode),
	    },
            _ => (),
        }

	logic::tick(&mut game_state, now, time_delta);

        render::draw(&game_state, &disp, &shdr, &texture_atlas, window_aspect_ratio);

        // more timey-wimey
        then = now;
        frame_times.push(now.elapsed());
        if frame_times.len() == 60 {
            let mut total_frame_time = std::time::Duration::ZERO;
            for frame_time in frame_times.iter() {
                total_frame_time += *frame_time;
            }
            println!("total frametime: {:?}/1000ms", total_frame_time);
            println!("avg frametime: {:?}/16.7ms", total_frame_time / 60);
            println!("score: {}", game_state.score);
            println!("");
            frame_times = Vec::new();
        }
    });
}
