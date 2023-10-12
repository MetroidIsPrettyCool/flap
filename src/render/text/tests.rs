use super::*;

#[test]
fn super_string_constructor() {
    let font_data: Vec<u8> = std::fs::read("./res/Octoville.otf").unwrap();
    let font: rusttype::Font<'static> = rusttype::Font::try_from_vec(font_data).unwrap();
    let a = SuperString::new(
        "ABCD".to_string(),
        &font,
        vec![ColorFmt::new(2, (1.0, 0.0, 0.0, 1.0))],
        1.0,
    );

    let b = SuperString {
        text: "ABCD".to_string(),
        font: &font,
        color_fmts: vec![ColorFmt {
            glyph_index: 2,
            color: (1.0, 0.0, 0.0, 1.0),
        }],
        normalized_height: 1.0,
    };

    assert_eq!(a.text, b.text);
    assert_eq!(a.color_fmts, b.color_fmts);
    assert_eq!(a.normalized_height, b.normalized_height);
}

#[test]
fn super_string_concatonation() {
    let font_data: Vec<u8> = std::fs::read("./res/Octoville.otf").unwrap();
    let font: rusttype::Font<'static> = rusttype::Font::try_from_vec(font_data).unwrap();

    let mut a = SuperString {
        text: "ABCD".to_string(),
        font: &font,
        color_fmts: vec![ColorFmt {
            glyph_index: 2,
            color: (1.0, 0.0, 0.0, 1.0),
        }],
        normalized_height: 1.0,
    };

    let b = SuperString {
        text: "EFGH".to_string(),
        font: &font,
        color_fmts: vec![ColorFmt {
            glyph_index: 2,
            color: (0.0, 1.0, 1.0, 1.0),
        }],
        normalized_height: 1.0,
    };

    a.cat(b);
    assert_eq!("ABCDEFGH", a.text);
    assert_eq!(
        vec![
            ColorFmt {
                glyph_index: 2,
                color: (1.0, 0.0, 0.0, 1.0),
            },
            ColorFmt {
                glyph_index: 6,
                color: (0.0, 1.0, 1.0, 1.0),
            }
        ],
        a.color_fmts
    );
}
