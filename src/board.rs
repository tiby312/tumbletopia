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

fn increase_mag(a: &mut i16) {
    if *a == 0 {
        return;
    }

    if *a > 0 {
        *a += 1
    } else {
        *a -= 1
    }
}

impl MyWorld {
    pub fn new() -> MyWorld {
        let mut w = BitField::from_iter(hex::Cube::new(0, 0).range(4).map(|x| x.to_axial()));

        let cat_long = 1; // 0,1,2
        let dog_long = 2; // 0,1,2
        let dog_long2 = 1; // 0,1,2
        let world_missing_index1 = 5; //0,1,2,3,4,5
        let world_missing_index2 = 2; //0,1,2,3,4,5

        let d = 5;

        let mut cat_start = [[-d, d], [0, -d], [d, 0]].map(Axial::from_arr);
        let mut dog_start = [[d, -d], [-d, 0], [0, d]].map(Axial::from_arr);
        let world_missing =
            [[2, -4], [-2, -2], [-4, 2], [-2, 4], [2, 2], [4, -2]].map(Axial::from_arr);

        w.set_coord(cat_start[cat_long], true);
        increase_mag(&mut cat_start[cat_long].q);
        increase_mag(&mut cat_start[cat_long].r);

        w.set_coord(dog_start[dog_long], true);
        increase_mag(&mut dog_start[dog_long].q);
        increase_mag(&mut dog_start[dog_long].r);

        w.set_coord(dog_start[dog_long2], true);
        increase_mag(&mut dog_start[dog_long2].q);
        increase_mag(&mut dog_start[dog_long2].r);

        w.set_coord(world_missing[world_missing_index1], false);
        w.set_coord(world_missing[world_missing_index2], false);

        for a in cat_start {
            w.set_coord(a, true);
        }

        for a in dog_start {
            w.set_coord(a, true);
        }

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
