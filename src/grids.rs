use super::*;

///A way to map a grid to world coordinates and vice versa
#[derive(Debug, Clone)]
pub struct GridMatrix {
    spacing: f32,
}

const FOO: f32 = hex::SQRT_3;
const EPSILON: f32 = 1.01;

impl GridMatrix {
    pub fn hex_axial_to_square_matrix(&self) -> cgmath::Matrix2<f32> {
        let sc = EPSILON * self.spacing() / FOO;
        let scale = cgmath::Matrix2::new(sc, 0.0, 0.0, sc);
        scale * hex::HEX_PROJ_FLAT
    }

    pub fn world_to_hex(&self, pos: cgmath::Vector2<f32>) -> Axial {
        use cgmath::SquareMatrix;
        let k = self.hex_axial_to_square_matrix().invert().unwrap() * pos;
        let q = k.x;
        let r = k.y;
        let s = -k.x - k.y;
        hex::Cube::round([q, r, s]).to_axial()
    }
    pub fn center_world_to_hex(&self, mut pos: cgmath::Vector2<f32>) -> Axial {
        pos.x -= EPSILON * self.spacing() / FOO;
        pos.y -= EPSILON * self.spacing() / FOO;
        self.world_to_hex(pos)
    }

    pub fn hex_axial_to_world(&self, coord: &Axial) -> cgmath::Vector2<f32> {
        let v = cgmath::Vector2::new(coord.q as f32, coord.r as f32);
        self.hex_axial_to_square_matrix() * v
    }

    pub fn new() -> Self {
        let spacing = 30.0; //grid_dim[0] / (grid_width as f32);
        Self { spacing }
    }
    pub fn spacing(&self) -> f32 {
        self.spacing
    }
}
