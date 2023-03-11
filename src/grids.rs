use super::*;

use axgeom::*;
///A way to map a grid to world coordinates and vice versa
#[derive(Debug)]
pub struct GridMatrix {
    grid_width: i16,
    grid_dim: [f32; 2],
    spacing: f32,
}

pub struct GridFilter {
    grid_width: i16,
}
impl movement::Filter for GridFilter {
    fn filter(&self, a: &GridCoord) -> bool {
        let x = a.0[0];
        let y = a.0[1];

        x >= 0 && y >= 0 && x < self.grid_width && y < self.grid_width
    }
}
pub fn hex_axial_to_square_matrix()->cgmath::Matrix2<f32>{
    let sc=19.0;
    let scale=cgmath::Matrix2::new(sc,0.0,0.0,sc);
    scale*hex::HEX_PROJ_FLAT
}

impl GridMatrix {
    pub fn world_to_hex(&self,pos:cgmath::Vector2<f32>)->GridCoord{
        use cgmath::SquareMatrix;
        let k=hex_axial_to_square_matrix().invert().unwrap()*pos;
        let k=[k.x.round() as i16,k.y.round() as i16];
        GridCoord(k)
    }
    pub fn center_world_to_hex(&self,mut pos:cgmath::Vector2<f32>)->GridCoord{
        pos.x-=19.0/2.0;
        pos.y-=19.0/2.0;

        use cgmath::SquareMatrix;
        let k=hex_axial_to_square_matrix().invert().unwrap()*pos;
        let k=[k.x.round() as i16,k.y.round() as i16];
        GridCoord(k)
    }
    
    pub fn hex_axial_to_world(&self,coord:&GridCoord)->cgmath::Vector2<f32>{
        let v=cgmath::Vector2::new(coord.0[0] as f32,coord.0[1] as f32);
        hex_axial_to_square_matrix()*v
    }

    pub fn filter(&self) -> GridFilter {
        GridFilter {
            grid_width: self.grid_width,
        }
    }
    pub fn new() -> Self {
        let grid_width = 32;

        let grid_dim = [1000.0f32, 1000.0];

        let spacing = grid_dim[0] / (grid_width as f32);

        Self {
            grid_width,
            grid_dim,
            spacing,
        }
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

    pub fn num_rows(&self) -> i16 {
        self.grid_width
    }
    pub fn dim(&self) -> &[f32; 2] {
        &self.grid_dim
    }
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
