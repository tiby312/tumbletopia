use crate::movement::bitfield::BitField;

use super::*;

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
    let d = 5;
    let k = [[-d, d], [0, -d], [d, 0], [d, -d], [-d, 0], [0, d]];

    hex::Cube::new(0, 0)
        .range(4)
        .chain(k.into_iter().map(GridCoord).map(|x| x.to_cube()))
}
