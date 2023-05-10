use super::*;

///A way to map a grid to world coordinates and vice versa
#[derive(Debug, Clone)]
pub struct GridMatrix {
    spacing: f32,
}

pub struct GridFilter {}
impl movement::Filter for GridFilter {
    fn filter(&self, a: &GridCoord) -> bool {
        world().find(|b| b.to_axial() == *a).is_some()
        // let x = a.0[0];
        // let y = a.0[1];

        // x >= 0 && y >= 0 && x < self.grid_width && y < self.grid_width
    }
}

fn world() -> impl Iterator<Item = hex::Cube> {
    // let dim=8;

    // let og = hex::Cube::new(dim-1, -(dim-1));
    // (0..dim)
    //     .flat_map(move |i| (0..dim).map(move |j| (i, j)))
    //     .map(move |(x, y)| {
    //         let a = hex::Cube::new(x, y);
    //         let a = a.0;
    //         og.add(hex::Cube([-a[0], -a[2], -a[1]]))
    //     })

    hex::Cube::new(0, 0).range(3)
}

const FOO: f32 = hex::SQRT_3;
const EPSILON: f32 = 1.01;

impl GridMatrix {
    pub fn hex_axial_to_square_matrix(&self) -> cgmath::Matrix2<f32> {
        let sc = EPSILON * self.spacing() / FOO;
        let scale = cgmath::Matrix2::new(sc, 0.0, 0.0, sc);
        scale * hex::HEX_PROJ_FLAT
    }

    //This is world
    pub fn world(&self) -> impl Iterator<Item = hex::Cube> {
        world()
    }
    pub fn world_to_hex(&self, pos: cgmath::Vector2<f32>) -> GridCoord {
        use cgmath::SquareMatrix;
        let k = self.hex_axial_to_square_matrix().invert().unwrap() * pos;
        let q = k.x;
        let r = k.y;
        let s = -k.x - k.y;
        hex::Cube::round([q, r, s]).to_axial()
    }
    pub fn center_world_to_hex(&self, mut pos: cgmath::Vector2<f32>) -> GridCoord {
        pos.x -= EPSILON * self.spacing() / FOO;
        pos.y -= EPSILON * self.spacing() / FOO;
        self.world_to_hex(pos)
    }

    pub fn hex_axial_to_world(&self, coord: &GridCoord) -> cgmath::Vector2<f32> {
        let v = cgmath::Vector2::new(coord.0[0] as f32, coord.0[1] as f32);
        self.hex_axial_to_square_matrix() * v
    }

    pub fn filter(&self) -> GridFilter {
        GridFilter {}
    }
    pub fn new() -> Self {
        let spacing = 30.0; //grid_dim[0] / (grid_width as f32);
        log!(format!("spacing{:?}", spacing));
        Self { spacing }
    }
    // pub fn to_world_topleft(&self, pos: Vec2<i16>) -> Vec2<f32> {
    //     pos.inner_as() * self.spacing
    // }

    // pub fn to_world_center(&self, pos: Vec2<i16>) -> Vec2<f32> {
    //     self.to_world_topleft(pos) + vec2same(self.spacing) / 2.0
    // }

    // pub fn to_grid_mod(&self, pos: Vec2<f32>) -> Vec2<f32> {
    //     let k = self.to_grid(pos);
    //     let k = k.inner_as() * self.spacing;
    //     pos - k
    // }
    // pub fn to_grid(&self, pos: Vec2<f32>) -> Vec2<i16> {
    //     let result = pos / self.spacing;
    //     result.inner_as()
    // }

    // pub fn num_rows(&self) -> i16 {
    //     self.grid_width
    // }
    // pub fn dim(&self) -> &[f32; 2] {
    //     &self.grid_dim
    // }
    pub fn spacing(&self) -> f32 {
        self.spacing
    }
}

// pub struct GridMatrix{
//     grid_width:usize, //Number of cells in a row
//     grid_dim:[f32;2]
// }

// impl GridMatrix{
//     pub fn new()->Self{

//         let grid_width = 32;

//         let grid_dim = [1000.0f32, 1000.0];

//         Self { grid_width,grid_dim }
//     }
//     pub fn world_to_grid(&self,a:[i16;2])->[f32;2]{
//         use matrix::*;
//         let gg=grid_to_world(self.grid_dim,self.grid_width).inverse().generate();
//         let pos:[f32;3]=gg.transform_vector([a[0] as f32,a[1] as f32,0.0].into()).into();
//         [pos[0],pos[1]]
//     }
//     pub fn grid_to_world(&self,a:[i16;2])->[f32;2]{
//         use matrix::*;
//         let gg=grid_to_world(self.grid_dim,self.grid_width).generate();
//         let pos:[f32;3]=gg.transform_vector([a[0] as f32,a[1] as f32,0.0].into()).into();
//         [pos[0],pos[1]]
//     }
// }

// fn grid_to_world(grid_dim:[f32;2],grid_width:usize)->impl matrix::MyMatrix+matrix::Inverse{
//     use matrix::*;

//     let spacing=grid_dim[0] / (grid_width as f32);

//     matrix::scale(spacing,spacing,1.0).chain(matrix::translation(spacing/2.0,spacing/2.0,0.0))
// }
