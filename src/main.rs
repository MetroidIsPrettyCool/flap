mod logic;
mod render;

use crate::logic::PhysObj;

use std::time::{Duration, Instant};

use glium::glutin::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};

// use rusttype::{Font};

const WINDOW_INITIAL_WIDTH: u32 = 1024;
const WINDOW_INITIAL_HEIGHT: u32 = 768;

#[derive(Clone)]
pub struct GameState {
    pub last_jump_time: Option<Instant>,
    pub last_rock_spawn_time: Option<Instant>,
    pub rock_fall_direction: f32,
    pub next_rock: PhysObj,
    pub last_coin_spawn_time: Option<Instant>,
    pub next_coin: PhysObj,
    pub birdy: PhysObj,
    pub rocks: Vec<PhysObj>,
    pub coins: Vec<PhysObj>,
    pub score: u32,
    pub keys: Vec<VirtualKeyCode>,
    pub dead: bool,
}
impl GameState {
    pub fn new() -> GameState {
        GameState {
            last_jump_time: None,
            last_rock_spawn_time: None,
            rock_fall_direction: 1.0,
            next_rock: logic::rock::new_rock(1.0),
            last_coin_spawn_time: None,
            next_coin: logic::coin::new_coin(),
            birdy: logic::birdy::new_birdy(),
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
    let mut next_frame_time = Instant::now();
    let mut then = Instant::now();
    let mut window_aspect_ratio = WINDOW_INITIAL_WIDTH as f32 / WINDOW_INITIAL_HEIGHT as f32;

    // initial setup
    let eve_lp = EventLoop::new();
    let win_b = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(
            WINDOW_INITIAL_WIDTH,
            WINDOW_INITIAL_HEIGHT,
        ))
        .with_title("flap");
    let cntxt_b = glium::glutin::ContextBuilder::new().with_double_buffer(Some(true)).with_depth_buffer(24);
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

    // Load font data from file
    let font_data: Vec<u8> = std::fs::read("./res/Octoville.otf")?;
    let font: rusttype::Font<'static> = rusttype::Font::try_from_vec(font_data).unwrap();

    let mut game_state = GameState::new();

    let mut avg_fps = 0.0;
    let mut last_frametime = std::time::Duration::ZERO;

    let mut last_frametime_avg = std::time::Instant::now();
    let mut frame_counter = 0;

    // main loop
    eve_lp.run(move |eve, _, ctrl_flow| {
        // time-wimey stuff
        let now = Instant::now();
        let time_delta = now.duration_since(then).as_secs_f32();
        next_frame_time += Duration::from_nanos(16_666_667); // next frame should render at most 1/60 sec later
        *ctrl_flow = ControlFlow::WaitUntil(next_frame_time);

        // upkeep
        if game_state.dead {
            game_state = GameState::new();
        }

        // handle events
        match eve {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *ctrl_flow = ControlFlow::ExitWithCode(0);
                return;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                window_aspect_ratio = size.width as f32 / size.height as f32;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => match state {
                ElementState::Pressed => game_state.keys.push(keycode),
                ElementState::Released => game_state.keys.retain(|&key| key != keycode),
            },
            _ => (),
        }

        logic::tick(&mut game_state, now, time_delta);

        render::draw(
            &game_state,
            &disp,
            &shdr,
            &texture_atlas,
            &font,
            window_aspect_ratio,
            last_frametime,
            avg_fps,
        );

        // more timey-wimey
	last_frametime = now.elapsed();
        frame_counter += 1;
        if last_frametime_avg.elapsed() >= std::time::Duration::from_secs(1) {
	    avg_fps = frame_counter as f32 / last_frametime_avg.elapsed().as_secs_f32();
            frame_counter = 0;
	    last_frametime_avg = std::time::Instant::now();
        }
	then = now;
    });
}
