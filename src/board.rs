use crate::movement::{bitfield::BitField, movement_mesh::Mesh, FilterRes};

use super::*;

pub struct GridFilter {}
impl movement::Filter for GridFilter {
    fn filter(&self, a: &GridCoord) -> FilterRes {
        //TODO inefficient. look at the hex coord website
        FilterRes::from_bool(world().find(|b| b.to_axial() == *a).is_some())

        // let x = a.0[0];
        // let y = a.0[1];

        // x >= 0 && y >= 0 && x < self.grid_width && y < self.grid_width
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct MyWorld {
    w: BitField,
}
impl MyWorld {
    pub fn new() -> MyWorld {
        let w = BitField::from_iter(world().map(|a| a.to_axial()));
        MyWorld { w }
    }
    pub fn get_game_cells(&self) -> &BitField {
        &self.w
    }
}

fn world() -> impl Iterator<Item = hex::Cube> {
    hex::Cube::new(0, 0).range(3)
}
