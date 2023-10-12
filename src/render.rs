use glium::Surface;

use rusttype::{point, Font, Scale};
use std::io::Write;

#[cfg(test)]
mod tests;

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

    // TODO: Clean up this boilerplate once it is understood

    let textbox_height_scaler = 20.0;

    // Desired font pixel height
    let height: f32 = f_buff.get_dimensions().1 as f32 / textbox_height_scaler;
    let pixel_height = height.ceil() as usize;

    // 2x scale in x direction to counter the aspect ratio of monospace characters.
    let scale = Scale {
        x: height * 2.0,
        y: height,
    };

    // The origin of a line of text is at the baseline (roughly where
    // non-descending letters sit). We don't want to clip the text, so we shift
    // it down with an offset when laying it out. v_metrics.ascent is the
    // distance between the baseline and the highest edge of any glyph in
    // the font. That's enough to guarantee that there's no clipping.
    let v_metrics = font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);

    let text_color = (1.0, 1.0, 1.0);

    // Glyphs to draw for "RustType". Feel free to try other strings.
    let score_text = format!(
        "Score: {} FPS: {:.1} {:?}",
        game_state.score, avg_fps, last_frametime
    );
    let glyphs: Vec<_> = font.layout(&score_text, scale, offset).collect();

    // Find the most visually pleasing width to display
    let width = glyphs
        .iter()
        .rev()
        .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
        .next()
        .unwrap_or(0.0)
        .ceil() as usize;

    let mut pixel_data = vec![0.0; width * pixel_height * 4]; // multiply by 4 because we're using rgba, thus 4 channels
    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                // ensure pixel value is within the range 0.0..=1.0
                let v = v.clamp(0.0, 1.0);
                // if v > 1.0 || v < 0.0 {
                //     eprintln!(
                //         "Glyph pixel value was out of range! {v} at {x} {y} for {g:#?} in {{glyphs:#?}}, clamped to {value}"
                //     );
                // }
                let color = (text_color.0, text_color.1, text_color.2, v);
                let x = x as i32 + bb.min.x;
                let y = y as i32 + bb.min.y;

                // clip our writes to the texture to stay within its bounds
                if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
                    let x = x as usize;
                    let y = y as usize;
                    let index = (x + y * width) * 4;
                    // write each color channel
                    pixel_data[index + 0] = color.0; // R
                    pixel_data[index + 1] = color.1; // G
                    pixel_data[index + 2] = color.2; // B
                    pixel_data[index + 3] = color.3; // A
                }
            })
        }
    }

    // build texture from raw color data
    let text_texture = glium::texture::RawImage2d::from_raw_rgba(
        pixel_data,
        (width.try_into().unwrap(), pixel_height.try_into().unwrap()),
    );
    let text_texture =
        glium::texture::srgb_texture2d::SrgbTexture2d::new(disp, text_texture).unwrap();

    // simple quad model, should be located on the upper left corner of the screen
    let text_model = square_from_edge_positions(
        -1.0,
        -1.0 + ((2.0 / textbox_height_scaler) * (width as f32 / pixel_height as f32)),
        1.0,
        1.0 - (2.0 / textbox_height_scaler),
        0.0,
        ((0.0, 0.0), (1.0, 1.0)),
    );

    // draw the text text... ure
    f_buff
        .draw(
            // draw vertices to framebuffer
            &glium::VertexBuffer::new(disp, &text_model).unwrap(),
            &glium::index::NoIndices(
                // not using indexed rendering
                glium::index::PrimitiveType::TrianglesList, // polygon type
            ),
            shdr, // shader program
            &glium::uniform! {
                window_aspect_ratio: window_aspect_ratio,
                texture_atlas: text_texture.sampled()
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

    // END BOILERPLATE

    f_buff.finish().unwrap(); // swap framebuffers
}
