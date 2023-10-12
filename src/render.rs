use glium::Surface;

#[cfg(test)]
mod tests;

mod text;

const BIRDY_DEPTH: f32 = 0.1;
const ROCK_DEPTH: f32 = 0.2;
const COIN_DEPTH: f32 = 0.3;

const PLAYFIELD_MODEL: Quad = square_from_edge_positions(
    -1.0,
    1.0,
    1.0,
    -1.0,
    0.9,
    ((0.0, 0.0), (8.0 / 64.0, 8.0 / 64.0)),
);

#[derive(Copy, Clone, Debug, PartialEq)]
struct Vert {
    position: (f32, f32, f32),
    texture_coordinates: (f32, f32),
}
glium::implement_vertex!(Vert, position, texture_coordinates);

type Quad = [Vert; 6];
const fn quad_from_verts(lt: Vert, rt: Vert, rb: Vert, lb: Vert) -> Quad {
    [rt, lt, lb, rt, lb, rb]
}

fn square_from_dims(
    width: f32,
    height: f32,
    depth: f32,
    center: (f32, f32),
    texture_coordinates: ((f32, f32), (f32, f32)),
) -> Quad {
    let center_x = center.0;
    let center_y = center.1;
    quad_from_verts(
        Vert {
            position: (center_x - width, center_y + height, depth),
            texture_coordinates: (texture_coordinates.0 .0, texture_coordinates.0 .1),
        },
        Vert {
            position: (center_x + width, center_y + height, depth),
            texture_coordinates: (texture_coordinates.1 .0, texture_coordinates.0 .1),
        },
        Vert {
            position: (center_x + width, center_y - height, depth),
            texture_coordinates: (texture_coordinates.1 .0, texture_coordinates.1 .1),
        },
        Vert {
            position: (center_x - width, center_y - height, depth),
            texture_coordinates: (texture_coordinates.0 .0, texture_coordinates.1 .1),
        },
    )
}

const fn square_from_edge_positions(
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    depth: f32,
    texture_coordinates: ((f32, f32), (f32, f32)),
) -> Quad {
    quad_from_verts(
        Vert {
            position: (left, top, depth),
            texture_coordinates: (texture_coordinates.0 .0, texture_coordinates.0 .1),
        },
        Vert {
            position: (right, top, depth),
            texture_coordinates: (texture_coordinates.1 .0, texture_coordinates.0 .1),
        },
        Vert {
            position: (right, bottom, depth),
            texture_coordinates: (texture_coordinates.1 .0, texture_coordinates.1 .1),
        },
        Vert {
            position: (left, bottom, depth),
            texture_coordinates: (texture_coordinates.0 .0, texture_coordinates.1 .1),
        },
    )
}

pub fn draw(
    game_state: &super::GameState,
    disp: &glium::Display,
    shdr: &glium::program::Program,
    texture_atlas: &glium::texture::srgb_texture2d::SrgbTexture2d,
    font: &rusttype::Font,
    window_aspect_ratio: f32,
    last_frametime: std::time::Duration,
    avg_fps: f32,
) {
    // get all our vertices together
    let mut vertices = Vec::new();
    vertices.extend_from_slice(&square_from_dims(
        game_state.birdy.width,
        game_state.birdy.height,
        BIRDY_DEPTH,
        (game_state.birdy.x, game_state.birdy.y),
        ((8.0 / 64.0, 0.0 / 64.0), (16.0 / 64.0, 8.0 / 64.0)),
    ));
    for rock in game_state.rocks.iter() {
        // rocks, duh
        if rock.y_velocity.is_sign_positive() {
            vertices.extend_from_slice(&square_from_dims(
                rock.width,
                rock.height,
                ROCK_DEPTH,
                (rock.x, rock.y),
                ((16.0 / 64.0, 32.0 / 64.0), (32.0 / 64.0, 48.0 / 64.0)),
            ));
        } else {
            vertices.extend_from_slice(&square_from_dims(
                rock.width,
                rock.height,
                ROCK_DEPTH,
                (rock.x, rock.y),
                ((0.0 / 64.0, 32.0 / 64.0), (16.0 / 64.0, 48.0 / 64.0)),
            ));
        }
    }
    for coin in game_state.coins.iter() {
        // coins, duh
        vertices.extend_from_slice(&square_from_dims(
            coin.width,
            coin.height,
            COIN_DEPTH,
            (coin.x, coin.y),
            ((32.0 / 64.0, 32.0 / 64.0), (48.0 / 64.0, 48.0 / 64.0)),
        ));
    }
    vertices.extend_from_slice(&PLAYFIELD_MODEL); // playfield

    // render game elements to framebuffer
    let mut f_buff = disp.draw(); // next framebuffer
    f_buff.clear(
        // clear the framebuffer
        None,                       // rect
        Some((0.0, 0.0, 0.0, 0.0)), // color
        true,                       // color_srgb
        Some(f32::MAX),             // depth
        None,                       // stencil
    );

    // primary draw, command, render playfield, coins, rocks and player
    f_buff
        .draw(
            // draw vertices to framebuffer
            &glium::VertexBuffer::new(disp, &vertices).unwrap(),
            &glium::index::NoIndices(
                // not using indexed rendering
                glium::index::PrimitiveType::TrianglesList, // polygon type
            ),
            shdr, // shader program
            &glium::uniform! {
                window_aspect_ratio: window_aspect_ratio,
                texture_atlas: texture_atlas.sampled()
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
            },
            &glium::DrawParameters {
                // draw parameters
                depth: glium::Depth {
                    test: glium::draw_parameters::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                blend: glium::draw_parameters::Blend::alpha_blending(),
                ..Default::default()
            },
        )
        .unwrap();

    // create and render our text strings

    // FPS: {avg_fps} {last_framtime}
    let fps_text = {
        let mut prefix = text::SuperString::new("FPS: ".to_string(), font, Vec::new(), 1.0 / 15.0);
        let avg_fps = if avg_fps != 0.0 {
            format!("{:.1} ", avg_fps)
        } else {
            "... ".to_string()
        };
        let avg_fps = text::SuperString::new(
            avg_fps,
            font,
            vec![text::ColorFmt::new(0, (0.0, 1.0, 0.0, 1.0))],
            1.0 / 15.0,
        );
        let last_frametime = text::SuperString::new(
            format!("{:?}", last_frametime),
            font,
            vec![text::ColorFmt::new(0, (1.0, 1.0, 0.0, 1.0))],
            1.0 / 15.0,
        );
        prefix.cat(avg_fps);
        prefix.cat(last_frametime);
        prefix
    };

    // render the FPS info to the framebuffer
    text::render_text(
        &mut f_buff,
        disp,
        shdr,
        window_aspect_ratio,
        fps_text,
        (-1.0, 1.0),
    );

    // Score: {score}
    // let score_text = {
    //     let mut prefix =
    //         text::SuperString::new("Score: ".to_string(), font, Vec::new(), 1.0 / 10.0);
    //     let score = text::SuperString::new(
    //         format!("{}", game_state.score),
    //         font,
    //         vec![text::ColorFmt::new(0, (0.0, 1.0, 0.0, 1.0))],
    //         1.0 / 10.0,
    //     );
    //     prefix.cat(score);
    //     prefix
    // };
    let score_text = text::SuperString::new(
        format!("Score: {}", game_state.score),
        font,
        vec![text::ColorFmt::new(6, (0.0, 1.0, 0.0, 1.0))],
        1.0 / 10.0,
    );

    // render the score info to the framebuffer
    text::render_text(
        &mut f_buff,
        disp,
        shdr,
        window_aspect_ratio,
        score_text,
        (-1.0, 1.0 - (1.0 / 15.0)),
    );

    f_buff.finish().unwrap(); // swap framebuffers
}
