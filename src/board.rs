use crate::movement::FilterRes;

use super::*;

pub struct GridFilter {}
impl movement::Filter for GridFilter {
    fn filter(&self, a: &GridCoord) -> FilterRes {
        FilterRes::from_bool(world().find(|b| b.to_axial() == *a).is_some())

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

    hex::Cube::new(0, 0).range(4)
}

#[derive(Debug)]
pub struct World {}
impl World {
    pub fn new() -> Self {
        World {}
    }
    pub fn filter(&self) -> GridFilter {
        GridFilter {}
    }
    //This is world
    pub fn iter_cells(&self) -> impl Iterator<Item = hex::Cube> {
        world()
    }
}
