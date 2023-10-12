use glium::Surface;

use rusttype::{point, Scale};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct ColorFmt {
    glyph_index: usize,
    color: (f32, f32, f32, f32),
}
impl ColorFmt {
    pub fn new(glyph_index: usize, color: (f32, f32, f32, f32)) -> Self {
        ColorFmt {
            glyph_index: glyph_index,
            color: color,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SuperString<'a> {
    text: String,
    font: &'a rusttype::Font<'a>,
    color_fmts: Vec<ColorFmt>,
    normalized_height: f32,
}
impl<'a> SuperString<'a> {
    pub fn new(
        text: String,
        font: &'a rusttype::Font<'a>,
        color_fmts: Vec<ColorFmt>,
        normalized_height: f32,
    ) -> Self {
        SuperString {
            text: text,
            font: font,
            color_fmts: color_fmts,
            normalized_height: normalized_height,
        }
    }

    pub fn cat(&mut self, other: Self) {
        let self_glyph_count = {
            let mut count = 0;
            for character in self.text.chars() {
                if self.font.glyph(character).id().0 != 0 {
                    count += 1;
                }
            }
            count
        };
        self.text += &other.text;
        for mut color_fmt in other.color_fmts {
            color_fmt.glyph_index += self_glyph_count;
            self.color_fmts.push(color_fmt);
        }
    }
}

pub fn render_text_to_texture(
    f_buff: &mut glium::Frame,
    disp: &glium::Display,
    text: &SuperString,
) -> glium::texture::srgb_texture2d::SrgbTexture2d {
    // determine text height in screen pixels
    let height = f_buff.get_dimensions().1 as f32 * text.normalized_height / 2.0;
    let pixel_height = height.ceil() as usize;

    // we'll be rendering glyphs at a 1:1 scale
    let scale = Scale {
        x: height,
        y: height,
    };

    // Determine font ascender (distance from the top of the text to the baseline) and offest baseline down
    let v_metrics = text.font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);

    // Glyphs to draw for the provided text string.
    let glyphs: Vec<_> = text.font.layout(&text.text, scale, offset).collect();

    // Determine textbox width by summing character widths and advance lengths
    let width = glyphs
        .iter()
        .rev()
        .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
        .next()
        .unwrap_or(0.0)
        .ceil() as usize;

    let mut pixel_data = vec![0.0; width * pixel_height * 4]; // multiply by 4 because we're using rgba, thus 4 channels

    let mut current_color = (1.0, 1.0, 1.0, 1.0);
    let mut colors = text.color_fmts.iter();
    let mut next_color = colors.next();

    for (index, g) in glyphs.into_iter().enumerate() {
	if let Some(color_fmt) = next_color {
            if color_fmt.glyph_index == index {
                current_color = color_fmt.color;
                next_color = colors.next();
            }
        }
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                // ensure pixel value is within the range 0.0..=1.0
                let v = v.clamp(0.0, 1.0);

                let x = x as i32 + bb.min.x;
                let y = y as i32 + bb.min.y;

                // clip our writes to the texture to stay within its bounds
                if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
                    let x = x as usize;
                    let y = y as usize;
                    let index = (x + y * width) * 4;
                    // write each color channel
                    pixel_data[index + 0] = current_color.0; // R
                    pixel_data[index + 1] = current_color.1; // G
                    pixel_data[index + 2] = current_color.2; // B
                    pixel_data[index + 3] = current_color.3 * v; // A
                }
            })
        }
    }

    // build texture from raw color data
    let raw_texture = glium::texture::RawImage2d::from_raw_rgba(
        pixel_data,
        (width.try_into().unwrap(), pixel_height.try_into().unwrap()),
    );
    glium::texture::srgb_texture2d::SrgbTexture2d::new(disp, raw_texture).unwrap()
}

pub fn render_text(
    f_buff: &mut glium::Frame,
    disp: &glium::Display,
    shdr: &glium::program::Program,
    window_aspect_ratio: f32,
    text: SuperString,
    position: (f32, f32),
) {
    let text_texture = render_text_to_texture(f_buff, disp, &text);

    // simple quad model, should be located on the upper left corner of the screen
    let text_model = super::square_from_edge_positions(
        position.0,
        position.0
            + (text.normalized_height * (text_texture.width() as f32 / text_texture.height() as f32)),
        position.1,
        position.1 - text.normalized_height,
        0.0,
        ((0.0, 0.0), (1.0, 1.0)),
    );

    // draw the text text... ure
    (*f_buff)
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
}
