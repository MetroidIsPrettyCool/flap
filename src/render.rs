use glium::Surface;

const BIRDY_DEPTH: f32 = 0.0;
const ROCK_DEPTH: f32 = 0.1;
const COIN_DEPTH: f32 = 0.2;

const PLAYFIELD_MODEL: Quad = square_from_edge_positions(
    -1.0,
    1.0,
    1.0,
    -1.0,
    0.6,
    ((0.0, 0.0), (8.0 / 64.0, 8.0 / 64.0)),
);

#[cfg(test)]
mod tests;

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

pub fn draw (
    game_state: &super::GameState,
    disp: &glium::Display,
    shdr: &glium::program::Program,
    texture_atlas: &glium::texture::srgb_texture2d::SrgbTexture2d,
    window_aspect_ratio: f32) {
    // get all our vertices together
    let mut vertices = Vec::new();
    vertices.extend_from_slice(&square_from_dims(
        game_state.birdy.size,
        game_state.birdy.size,
        BIRDY_DEPTH,
        (game_state.birdy.x, game_state.birdy.y),
        ((8.0 / 64.0, 0.0 / 64.0), (16.0 / 64.0, 8.0 / 64.0)),
    ));
    for rock in game_state.rocks.iter() {
        // rocks, duh
        if rock.y_velocity.is_sign_positive() {
            vertices.extend_from_slice(&square_from_dims(
                rock.size,
                rock.size,
                ROCK_DEPTH,
                (rock.x, rock.y),
                ((16.0 / 64.0, 32.0 / 64.0), (32.0 / 64.0, 48.0 / 64.0)),
            ));
        } else {
            vertices.extend_from_slice(&square_from_dims(
                rock.size,
                rock.size,
                ROCK_DEPTH,
                (rock.x, rock.y),
                ((0.0 / 64.0, 32.0 / 64.0), (16.0 / 64.0, 48.0 / 64.0)),
            ));
        }
    }
    for coin in game_state.coins.iter() {
        // coins, duh
        vertices.extend_from_slice(&square_from_dims(
            coin.size,
            coin.size,
            COIN_DEPTH,
            (coin.x, coin.y),
            ((32.0 / 64.0, 32.0 / 64.0), (48.0 / 64.0, 48.0 / 64.0)),
        ));
    }
    vertices.extend_from_slice(&PLAYFIELD_MODEL); // playfield

    // render to framebuffer
    let mut f_buff = disp.draw(); // next framebuffer
    f_buff.clear(
        // clear the framebuffer
        None,                       // rect
        Some((0.0, 0.0, 0.0, 0.0)), // color
        true,                       // color_srgb
        Some(f32::MAX),             // depth
        None,                       // stencil
    );
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
                ..Default::default()
            },
        )
        .unwrap();
    f_buff.finish().unwrap(); // swap framebuffers
}
