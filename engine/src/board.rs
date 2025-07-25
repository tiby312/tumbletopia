use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct MyWorld {
    pub land: mesh::small_mesh::SmallMesh,
    pub radius: u8,
    //pub starting_team: Team,
    pub starting_state: unit::GameState, //pub map: unit::Map,
    pub land_as_vec: Vec<usize>,
}

//pub const NUM_CELLS: usize = 128;

pub const TABLE_SIZE: usize = 512;

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
    let mut index = unit.to_index() as isize;
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

impl GameState {
    //TODO replace with HexFmt
    pub fn into_string(&self, world: &MyWorld) -> String {
        let mut ret = String::new();
        for index in world.get_game_cells().inner.iter_ones() {
            let foo = match self.factions.get_cell_inner(index) {
                unit::GameCell::Piece(unit::Piece {
                    height: stack_height,
                    team,
                    ..
                }) => match (stack_height.to_num(), team) {
                    (1, Team::Neutral) => 'j',
                    (2, Team::Neutral) => 'k',
                    (3, Team::Neutral) => 'l',
                    (4, Team::Neutral) => 'm',
                    (5, Team::Neutral) => 'n',
                    (6, Team::Neutral) => 'o',
                    (1, Team::White) => 'r',
                    (2, Team::White) => 's',
                    (3, Team::White) => 't',
                    (4, Team::White) => 'u',
                    (5, Team::White) => 'v',
                    (6, Team::White) => 'w',
                    (1, Team::Black) => 'b',
                    (2, Team::Black) => 'c',
                    (3, Team::Black) => 'd',
                    (4, Team::Black) => 'e',
                    (5, Team::Black) => 'f',
                    (6, Team::Black) => 'g',
                    _ => continue,
                },
                unit::GameCell::Empty => '_',
            };

            ret.push(foo);
        }
        ret
    }
}

impl MyWorld {
    pub fn format<'a, H: hex::HexDraw>(&self, foo: &'a H) -> hex::Displayer<'a, H> {
        hex::disp(foo, self.radius as i8)
    }

    // pub fn with_size(s:i8,starting_team:ActiveTeam) -> MyWorld {
    //     let size=s;

    //     let land = mesh::small_mesh::SmallMesh::from_iter(
    //         hex::Cube::new(0, 0).range(size).map(|x| x.to_axial()),
    //     );

    //     MyWorld {
    //         land,
    //         radius: size as u8,
    //         starting_team
    //     }
    // }

    pub fn load_from_string(s: &str) -> Option<MyWorld> {
        // Area = (3√3 / 2) x (Side Length)^2
        //
        let size = ((3. + (12. * s.len() as f64 - 3.)).sqrt() / 6.).ceil() as i8;
        log!("SIZE OF HEX={}", size);

        if size > 8 {
            return None;
        }

        //let world=MyWorld::with_size(size,ActiveTeam::White);
        let land = mesh::small_mesh::SmallMesh::from_iter(
            hex::Cube::new(0, 0).range(size - 1).map(|x| x.to_axial()),
        );

        let land_as_vec: Vec<_> = land.inner.iter_ones().collect();

        if land_as_vec.len() != s.len() {
            return None;
        }

        let mut g = GameState::new();
        for (a, i) in s.chars().zip(land.inner.iter_ones()) {
            use StackHeight::*;
            let (stack, team) = match a {
                'j' => (Stack1, Team::Neutral),
                'k' => (Stack2, Team::Neutral),
                'l' => (Stack3, Team::Neutral),
                'm' => (Stack4, Team::Neutral),
                'n' => (Stack5, Team::Neutral),
                'o' => (Stack6, Team::Neutral),
                'r' => (Stack1, Team::White),
                's' => (Stack2, Team::White),
                't' => (Stack3, Team::White),
                'u' => (Stack4, Team::White),
                'v' => (Stack5, Team::White),
                'w' => (Stack6, Team::White),
                'b' => (Stack1, Team::Black),
                'c' => (Stack2, Team::Black),
                'd' => (Stack3, Team::Black),
                'e' => (Stack4, Team::Black),
                'f' => (Stack5, Team::Black),
                'g' => (Stack6, Team::Black),
                'x' => (Stack0, Team::White),
                'y' => (Stack0, Team::Black),
                '-' => {
                    continue;
                }
                _ => {
                    return None;
                }
            };

            //x and y are lighthouses.
            //lighthouses behave as zero stacks that can't attack.
            let lighthouse = if stack == Stack0 { true } else { false };

            log!("Adding {} {:?} {:?}", i, stack, team);
            g.factions.add_cell_inner(i, stack, team, lighthouse);
        }

        // let g = unit::GameStateTotal {
        //     tactical: g,
        //     fog: std::array::from_fn(|_| mesh::small_mesh::SmallMesh::new()),
        // };

        Some(MyWorld {
            land,
            radius: size as u8,
            //starting_team: Team::White,
            starting_state: g,
            land_as_vec,
        })
    }

    #[deprecated]
    pub fn new2() -> MyWorld {
        //let size = 3;
        let size = 3;

        //let j = [[2, -4], [-2, -2], [-4, 2], [-2, 4], [2, 2], [4, -2]];

        //for size 3 use this
        //let j = [[-1, -2], [-3, 1], [-2, 3], [1, 2], [3, -1], [2, -3]];

        let _land = mesh::small_mesh::SmallMesh::from_iter(
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
        todo!();

        // MyWorld {
        //     land,
        //     radius: size as u8,
        //     starting_team:ActiveTeam::White
        //     //map,
        // }
    }

    //TODO get rid of this???? use bounds checking instead???
    pub fn get_game_cells(&self) -> &mesh::small_mesh::SmallMesh {
        &self.land
    }
}
