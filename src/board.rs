use crate::mesh::bitfield::BitField;

use super::*;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct MyWorld {
    pub seed: WorldSeed,
    w: BitField,
    dog_start: [Axial; 3],
    cat_start: [Axial; 3],
}

// impl Default for MyWorld {
//     fn default() -> Self {
//         Self::new()
//     }
// }

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

#[test]
fn foo() {
    //let i=234314;
    let mut cat_long_buffer = 0;
    let mut dog_long_buffer = 0;
    let mut dog_long2_buffer = 0;
    let mut world_missing_index1_buffer = 0;
    let mut world_missing_index2_buffer = 0;

    //3*3*2*5*4 = 360 choices!!!

    for mut i in 0..360 {
        let cat_long = i % 3;
        i /= 3;
        let dog_long = i % 3;
        i /= 3;
        let dog_long2 = i % 2;
        i /= 2;
        let world_missing_index1 = i % 5;
        i /= 5;
        let world_missing_index2 = i % 4;
        i /= 4;
        assert_eq!(i, 0);

        //12 bits total
        // let cat_long = i%3; // 0,1,2   //2 bits
        // let dog_long = (i/3)%3; // 0,1,2   // 2 bits
        // let dog_long2 = (i/(3*3))%3; // 0,1,2   // 2 bits
        // let world_missing_index1 = (i/(3*3*3))%5; //0,1,2,3,4,5 //3 bits
        // let world_missing_index2 = (i/(3*3*3*5))%5; //0,1,2,3,4,5 //3 bits

        cat_long_buffer |= cat_long;
        dog_long_buffer |= dog_long;
        dog_long2_buffer |= dog_long2;
        world_missing_index1_buffer |= world_missing_index1;
        world_missing_index2_buffer |= world_missing_index2;
    }

    dbg!(
        cat_long_buffer,
        dog_long_buffer,
        dog_long2_buffer,
        world_missing_index1_buffer,
        world_missing_index2_buffer
    );
    assert!(false);
}

#[derive(Deserialize, Serialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct WorldSeed {
    foo: u64,
}

impl Default for WorldSeed {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldSeed {
    pub fn new() -> Self {
        use rand::Rng;
        let num = rand::thread_rng().gen_range(0..360);
        WorldSeed { foo: num }
    }
}

impl MyWorld {
    pub fn new(seed: WorldSeed) -> MyWorld {
        let mut w = BitField::from_iter(hex::Cube::new(0, 0).range(4).map(|x| x.to_axial()));

        //3*3*2*5*4 = 360 choices!!!

        let mut i: usize = seed.foo.try_into().unwrap();

        let cat_long = i % 3;
        i /= 3;
        let dog_long = i % 3;
        i /= 3;
        let mut dog_long2 = i % 2;
        i /= 2;
        let world_missing_index1 = i % 5;
        i /= 5;
        let mut world_missing_index2 = i % 4;
        i /= 4;
        assert_eq!(i, 0);

        if dog_long == dog_long2 {
            dog_long2 = (dog_long2 + 1) % 3
        }

        if world_missing_index1 == world_missing_index2 {
            world_missing_index2 = (world_missing_index2 + 1) % 5
        }
        assert_ne!(dog_long, dog_long2);
        assert_ne!(world_missing_index1, world_missing_index2);
        assert!((0..3).contains(&cat_long), "uhoh:{}", cat_long);
        assert!((0..3).contains(&dog_long));
        assert!((0..3).contains(&dog_long2));
        assert!((0..6).contains(&world_missing_index1));
        assert!((0..6).contains(&world_missing_index2));

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
            seed,
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
