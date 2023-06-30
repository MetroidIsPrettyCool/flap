use super::*;

#[test]
fn square_from_edge_positions() {
    assert_eq!(
        [
            Vert {
                // rt
                position: (1.0, 1.0, 0.6),
                texture_coordinates: (0.125, 0.0),
            },
            Vert {
                // lt
                position: (-1.0, 1.0, 0.6),
                texture_coordinates: (0.0, 0.0),
            },
            Vert {
                // lb
                position: (-1.0, -1.0, 0.6),
                texture_coordinates: (0.0, 0.125),
            },
            Vert {
                // rt
                position: (1.0, 1.0, 0.6),
                texture_coordinates: (0.125, 0.0),
            },
            Vert {
                // lb
                position: (-1.0, -1.0, 0.6),
                texture_coordinates: (0.0, 0.125),
            },
            Vert {
                // rb
                position: (1.0, -1.0, 0.6),
                texture_coordinates: (0.125, 0.125),
            },
        ],
        PLAYFIELD_MODEL
    );
}

#[test]
fn square_from_dims() {
    assert_eq!(
        [
            Vert {
                // rt
                position: (1.0, 1.0, 0.6),
                texture_coordinates: (1.0, 0.0),
            },
            Vert {
                // lt
                position: (-1.0, 1.0, 0.6),
                texture_coordinates: (0.0, 0.0),
            },
            Vert {
                // lb
                position: (-1.0, -1.0, 0.6),
                texture_coordinates: (0.0, 1.0),
            },
            Vert {
                // rt
                position: (1.0, 1.0, 0.6),
                texture_coordinates: (1.0, 0.0),
            },
            Vert {
                // lb
                position: (-1.0, -1.0, 0.6),
                texture_coordinates: (0.0, 1.0),
            },
            Vert {
                // rb
                position: (1.0, -1.0, 0.6),
                texture_coordinates: (1.0, 1.0),
            },
        ],
        super::square_from_dims(1.0, 1.0, 0.6, (0.0, 0.0), ((0.0, 0.0), (1.0, 1.0)))
    );
}
