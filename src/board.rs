use crate::mesh::bitfield::BitField;

use super::*;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct MyWorld {
    w: BitField,
    dog_start: [Axial; 3],
    cat_start: [Axial; 3],
}

impl Default for MyWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl MyWorld {
    pub fn new() -> MyWorld {
        let w = BitField::from_iter(world().map(|a| a.to_axial()));
        let d = 5;

        let cat_start = [[-d, d], [0, -d], [d, 0]].map(Axial::from_arr);
        let dog_start = [[d, -d], [-d, 0], [0, d]].map(Axial::from_arr);

        MyWorld {
            w,
            dog_start,
            cat_start,
        }
    }
    pub fn cat_start(&self) -> &[Axial] {
        &self.cat_start
    }
    pub fn dog_start(&self) -> &[Axial] {
        &self.dog_start
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
        .chain(k.into_iter().map(Axial::from_arr).map(|x| x.to_cube()))
}
