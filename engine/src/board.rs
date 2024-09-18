use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct MyWorld {
    pub seed: WorldSeed,
    pub land: mesh::small_mesh::SmallMesh,
    pub radius: u8,
}

pub const NUM_CELLS: usize = 128;

#[test]
fn foo() {
    //let i=234314;
    let mut white_long_buffer = 0;
    let mut black_long_buffer = 0;
    let mut black_long2_buffer = 0;
    let mut world_missing_index1_buffer = 0;
    let mut world_missing_index2_buffer = 0;

    //3*3*2*5*4 = 360 choices!!!

    for mut i in 0..360 {
        let white_long = i % 3;
        i /= 3;
        let black_long = i % 3;
        i /= 3;
        let black_long2 = i % 2;
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

        white_long_buffer |= white_long;
        black_long_buffer |= black_long;
        black_long2_buffer |= black_long2;
        world_missing_index1_buffer |= world_missing_index1;
        world_missing_index2_buffer |= world_missing_index2;
    }

    dbg!(
        white_long_buffer,
        black_long_buffer,
        black_long2_buffer,
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
        let num = rand::thread_rng().gen_range(0..180);
        //let num = 0;
        WorldSeed { foo: num }
    }
}

pub fn dis_to_hex_of_hexagon(a: Axial, dir: hex::HDir, radius: i8) -> i8 {
    let a = a.to_cube();
    match dir {
        hex::HDir::BottomRight => radius - a.q - a.r.max(0),
        hex::HDir::Bottom => radius - a.q - a.s.max(0),
        hex::HDir::BottomLeft => radius - a.s - a.q.max(0),
        hex::HDir::TopLeft => radius - a.s - a.r.max(0),
        hex::HDir::Top => radius - a.r - a.s.max(0),
        hex::HDir::TopRight => radius - a.r - a.q.max(0),
    }
}

pub const STRIDES: [i32; 6] = [16, 15, -1, -16, -15, 1];

pub fn determine_stride(a: hex::HDir) -> i32 {
    match a {
        hex::HDir::BottomRight => 16,
        hex::HDir::Bottom => 15,
        hex::HDir::BottomLeft => -1,
        hex::HDir::TopLeft => -16,
        hex::HDir::Top => -15,
        hex::HDir::TopRight => 1,
    }
}

#[test]
fn test_dummy() {
    let size = 5;

    let ll = hex::Cube::new(0, 0).range(size).map(|x| x.to_axial());
    let mesh = mesh::small_mesh::SmallMesh::from_iter(ll);
    let mut mesh2 = mesh::small_mesh::SmallMesh::new();

    let unit = Axial { q: 1, r: 2 };

    let computed_dis = dis_to_hex_of_hexagon(unit, hex::HDir::BottomRight, 5);

    // let top_right_stride = 1; //hex::Dir::TopRight
    // let bottom_left_stride = -1; //hex::DIR::BottomLeft
    // let top_stride = -15;
    // let bottom_stride = 15;
    // let top_left_stride = -16;
    let bottom_right_stride = 16;

    let stride = bottom_right_stride;
    let mut index = mesh::small_mesh::conv(unit) as isize;
    for _ in 0..computed_dis {
        index += stride;
        mesh2.inner.set(index as usize, true);
    }

    for q in -8..8 {
        for r in -8..8 {
            let unit = Axial { q, r };

            if mesh2.is_set(unit) {
                print!("x ");
            } else if mesh.is_set(unit) {
                print!("o ");
            } else {
                print!("- ");
            }
        }
        println!();
    }

    panic!("FIN");
}

#[test]
fn test_dis_to_hex_border() {
    let size = 3;

    let ll = hex::Cube::new(0, 0).range(size).map(|x| x.to_axial());
    let mesh = mesh::small_mesh::SmallMesh::from_iter(ll);

    for q in -8..8 {
        for r in -8..8 {
            let unit = Axial { q, r };
            if mesh.is_set(unit) {
                for i in 0..6 {
                    let true_dis = unit
                        .to_cube()
                        .ray_from_vector(hex::Cube::from_arr(hex::OFFSETS[i]))
                        .take_while(|x| mesh.is_set(**x))
                        .count() as i8;

                    let computed_dis = dis_to_hex_of_hexagon(unit, hex::HDir::from(i as u8), 3);

                    assert_eq!(true_dis, computed_dis);
                }
            } else {
                print!("- ");
            }
        }
        println!();
    }

    //panic!("FIN");
}

impl MyWorld {
    pub fn new(seed: WorldSeed) -> MyWorld {
        //let size = 3;
        let size = 7;

        //let j = [[2, -4], [-2, -2], [-4, 2], [-2, 4], [2, 2], [4, -2]];

        //for size 3 use this
        //let j = [[-1, -2], [-3, 1], [-2, 3], [1, 2], [3, -1], [2, -3]];

        let land = mesh::small_mesh::SmallMesh::from_iter(
            hex::Cube::new(0, 0).range(size).map(|x| x.to_axial()),
        );
        //w.set_coord(Axial::zero(), false);
        //3*3*5*4 = 180 choices!!!

        // let mut i: usize = seed.foo.try_into().unwrap();

        // let white_long = i % 3;
        // i /= 3;
        // let black_long = i % 3;
        // i /= 3;
        // // let mut dog_long2 = i % 2;
        // // i /= 2;
        // let world_missing_index1 = i % 5;
        // i /= 5;
        // let mut world_missing_index2 = i % 4;
        // i /= 4;
        // assert_eq!(i, 0);

        // // if dog_long == dog_long2 {
        // //     dog_long2 = (dog_long2 + 1) % 3
        // // }

        // if world_missing_index1 == world_missing_index2 {
        //     world_missing_index2 = (world_missing_index2 + 1) % 5
        // }
        // //assert_ne!(dog_long, dog_long2);
        // assert_ne!(world_missing_index1, world_missing_index2);
        // assert!((0..3).contains(&white_long), "uhoh:{}", white_long);
        // assert!((0..3).contains(&black_long));
        // //assert!((0..3).contains(&dog_long2));
        // assert!((0..6).contains(&world_missing_index1));
        // assert!((0..6).contains(&world_missing_index2));

        // let d = 4;

        // let mut white_start: Vec<_> = [[-d, d], [0, -d], [d, 0]].map(Axial::from_arr).into();
        // let mut black_start: Vec<_> = [[d, -d], [-d, 0], [0, d]].map(Axial::from_arr).into();

        // let world_missing = j.map(Axial::from_arr);

        // for a in 0..3 {
        //     if a == white_long {
        //         continue;
        //     }
        //     let mut j = white_start[a];
        //     increase_mag(&mut j.q);
        //     increase_mag(&mut j.r);
        //     white_start.push(j);
        // }

        // for a in 0..3 {
        //     if a == black_long {
        //         continue;
        //     }
        //     let mut j = black_start[a];
        //     increase_mag(&mut j.q);
        //     increase_mag(&mut j.r);
        //     black_start.push(j);
        // }

        // // let mut j=dog_start[dog_long2];
        // // increase_mag(&mut j.q);
        // // increase_mag(&mut j.r);
        // // dog_start.push(j);

        // let mut land = BitField::new();
        // land.set_coord(world_missing[world_missing_index1], true);
        // land.set_coord(world_missing[world_missing_index2], true);

        // // let starting_land=[[0,-3],[-3,0],[-3,3],[0,3],[3,0],[3,-3]];

        // // for a in starting_land{
        // //     land.set_coord(Axial::from_arr(a), true);
        // // }

        // for &a in white_start.iter() {
        //     w.set_coord(a, true);
        // }

        // for &a in black_start.iter() {
        //     w.set_coord(a, true);
        // }

        MyWorld {
            seed,
            land,
            radius: size as u8,
        }
    }
    // pub fn white_start(&self) -> &[Axial] {
    //     &self.white_start
    // }
    // pub fn black_start(&self) -> &[Axial] {
    //     &self.black_start
    // }

    pub fn get_game_cells(&self) -> &mesh::small_mesh::SmallMesh {
        &self.land
    }
}
