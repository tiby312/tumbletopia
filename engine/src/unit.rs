use mesh::small_mesh::SmallMesh;

use super::*;

// #[derive(Serialize, Deserialize, Default, Clone, Debug, Hash, Eq, PartialEq)]
// pub struct Factions {
//     pub cells: Tribe,
// }
// impl Factions {
//     // pub fn has_a_set(&self, coord: Axial) -> bool {
//     //     self.black.is_set(coord) || self.white.is_set(coord)
//     // }

//     // pub fn has_a_set_type(&self, coord: Axial) -> Option<UnitType> {
//     //     if let Some(a) = self.black.try_get_type(coord) {
//     //         return Some(a);
//     //     }

//     //     self.white.try_get_type(coord)
//     // }

//     // pub fn get_unit_mut(&mut self, team: ActiveTeam, coord: Axial) -> &mut UnitData {
//     //     self.relative_mut(team)
//     //         .this_team
//     //         .find_slow_mut(&coord)
//     //         .unwrap()
//     // }
//     // pub fn get_unit(&self, team: ActiveTeam, coord: Axial) -> &UnitData {
//     //     self.relative(team).this_team.find_slow(&coord).unwrap()
//     // }
//     // pub fn relative_mut(&mut self, team: ActiveTeam) -> FactionRelative<&mut Tribe> {
//     //     match team {
//     //         ActiveTeam::White => FactionRelative {
//     //             this_team: &mut self.white,
//     //             that_team: &mut self.black,
//     //         },
//     //         ActiveTeam::Black => FactionRelative {
//     //             this_team: &mut self.black,
//     //             that_team: &mut self.white,
//     //         },
//     //     }
//     // }
//     // pub fn relative(&self, team: ActiveTeam) -> FactionRelative<&Tribe> {
//     //     match team {
//     //         ActiveTeam::White => FactionRelative {
//     //             this_team: &self.white,
//     //             that_team: &self.black,
//     //         },
//     //         ActiveTeam::Black => FactionRelative {
//     //             this_team: &self.black,
//     //             that_team: &self.white,
//     //         },
//     //     }
//     // }
// }

#[must_use]
#[derive(Serialize, Deserialize, Hash, Ord, PartialOrd, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ActiveTeam {
    White = 0,
    Black = 1,
    Neutral = 2,
}
impl ActiveTeam {
    pub fn is_white(&self) -> bool {
        if let ActiveTeam::White = self {
            true
        } else {
            false
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = Self> {
        [*self, self.not()].into_iter().cycle()
    }
    pub fn not(&self) -> Self {
        match self {
            ActiveTeam::White => ActiveTeam::Black,
            ActiveTeam::Black => ActiveTeam::White,
            ActiveTeam::Neutral => ActiveTeam::Neutral,
        }
    }
}

// pub struct FactionRelative<T> {
//     pub this_team: T,
//     pub that_team: T,
// }
// impl FactionRelative<&mut Tribe> {
//     pub fn has_a_set(&self, coord: Axial) -> bool {
//         self.this_team.is_set(coord) || self.that_team.is_set(coord)
//     }
// }
// impl FactionRelative<&Tribe> {
//     pub fn has_a_set(&self, coord: Axial) -> bool {
//         self.this_team.is_set(coord) || self.that_team.is_set(coord)
//     }
// }

// #[derive(Serialize, Deserialize, Default, Clone, Debug, Hash, Eq, PartialEq)]
// pub struct Terrain {
//     pub land: BitField,
//     pub forest: BitField,
//     pub mountain: BitField,
// }
// impl Terrain {
//     pub fn is_set(&self, a: Axial) -> bool {
//         self.land.is_set(a) || self.forest.is_set(a) || self.mountain.is_set(a)
//     }
//     pub fn gen_all_terrain(&self) -> BitField {
//         let mut k = BitField::new();
//         k.union_with(&self.land);
//         k.union_with(&self.forest);
//         k.union_with(&self.mountain);
//         k
//     }
// }
// #[derive(Serialize, Deserialize, Default, Clone, Debug, Hash, Eq, PartialEq)]
// pub struct Environment {
//     pub terrain: Terrain,
//     pub fog: BitField,
//     pub powerups: Vec<Axial>,
// }

//Additionally removes need to special case animation.
#[derive(Serialize, Deserialize, Default, Clone, Debug, Hash, Eq, PartialEq)]
pub struct GameState {
    pub factions: Tribe,
    //pub env: Environment,
}

#[must_use]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameOver {
    WhiteWon,
    BlackWon,
    Tie,
}

impl GameState {
    pub fn hash_me(&self) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::hash::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
    pub fn game_is_over(&self, _world: &board::MyWorld, _team: ActiveTeam) -> Option<GameOver> {
        //TODO update
        // let this_team_stuck = 'foo: {
        //     for unit in self.factions.relative(team).this_team.iter_mesh() {
        //         let mesh = self.generate_possible_moves_movement(world, &unit, team);
        //         if !mesh.is_empty() {
        //             break 'foo false;
        //         }
        //     }
        //     true
        // };

        // if this_team_stuck {
        //     match team {
        //         ActiveTeam::White => Some(GameOver::BlackWon),
        //         ActiveTeam::Black => Some(GameOver::WhiteWon),
        //     }
        // } else {
        //     None
        // }
        None
    }
}

// #[derive(Eq, PartialEq, Hash, Debug, Clone, Ord, PartialOrd)]
// pub struct UnitData {
//     pub position: Axial,
//     pub typ: Type,
// }

// #[derive(Default, Clone)]
// pub struct SpokeNode {
//     spokes: [u8; 6],
// }
// impl SpokeNode {
//     pub fn has_piece_at_end(&self, dir: usize) -> bool {
//         self.spokes[dir] & (1 << 7) != 0
//     }

//     pub fn distance(&self, dir: usize) -> u8 {
//         self.spokes[dir] & !(1 << 7)
//     }
// }

// pub struct Spokes {
//     inner: Vec<SpokeNode>,
// }
// impl Spokes {
//     pub fn update_to_added_unit(&mut self, factions: &Tribe, ax: Axial, world: &board::MyWorld) {
//         let s = self.get_spokes(ax).clone();

//         for i in 0..6 {
//             let dis = s.distance(i);

//             for k in 0..dis {}
//             let has_piece = s.has_piece_at_end(i);
//             let j = (i + 3) % 6;
//             // let end_point_ax=ax.add(hex::OFFSETS[i].mul(dis));

//             // self.get_spokes_mut(end_point_ax).spokes[j]=dis;
//         }
//     }

//     pub fn generate(factions: &Tribe, world: &board::MyWorld) -> Spokes {
//         let mut s = Spokes {
//             inner: vec![SpokeNode::default(); 256],
//         };

//         for unit in world.get_game_cells().iter_mesh() {
//             let res = factions.iter_end_points(world, unit);

//             let res = res.map(|(ax, foo)| {
//                 let mut val = ax.to_cube().dist(&unit.to_cube()) as u8;

//                 if foo.is_some() {
//                     val |= 1 << 7;
//                 }
//                 val
//             });

//             s.get_spokes_mut(unit).spokes = res;
//         }

//         s
//     }
//     pub fn get_spokes(&self, a: Axial) -> &SpokeNode {
//         let ind = mesh::small_mesh::conv(a);
//         &self.inner[ind]
//     }
//     pub fn get_spokes_mut(&mut self, a: Axial) -> &mut SpokeNode {
//         let ind = mesh::small_mesh::conv(a);
//         &mut self.inner[ind]
//     }
// }

#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq, Hash, Clone)]
pub struct Tribe {
    pub cells: [SmallMesh; 3],
    pub team: SmallMesh,

    //This just signifies if there is a number in cells.
    //This way you can just check one mesh to see if a piece is there or not
    //instead of checking 3
    pub piece: SmallMesh,
}

impl Tribe {
    pub fn iter_end_points(
        &self,
        world: &board::MyWorld,
        index: usize,
    ) -> [(i8, Option<(u8, ActiveTeam)>); 6] {
        core::array::from_fn(|i| {
            // let first:Vec<_>=for_ray(unit,i).collect();
            // let second:Vec<_>=for_ray2(unit,i).collect();
            // assert_eq!(first,second);

            let dd = hex::HDir::from(i as u8);
            let stride = board::determine_stride(dd) as isize;
            let dis = board::dis_to_hex_of_hexagon(
                mesh::small_mesh::inverse(index),
                dd,
                world.radius as i8,
            );
            let mut index2 = index as isize;

            for d in 0..dis {
                index2 += stride;

                if self.piece.inner[index2 as usize] {
                    if let Some(pp) = self.get_cell_inner(index2 as usize) {
                        return (d + 1, Some(pp));
                    }
                }
            }

            // for k in for_ray2(unit, i) {

            //     let index = mesh::small_mesh::conv(k);
            //     if self.has_a_piece(index) {
            //         if let Some((a, b)) = self.get_cell_inner(index) {
            //             return (dis, Some((a, b)));
            //         }
            //     }
            // }
            (dis, None)
        })
    }

    pub fn new() -> Tribe {
        Tribe {
            cells: [0; 3].map(|_| SmallMesh::new()),
            team: SmallMesh::new(),
            piece: SmallMesh::new(),
        }
    }

    pub fn remove(&mut self, a: Axial) {
        let a = mesh::small_mesh::conv(a);
        self.remove_inner(a);
    }
    pub fn remove_inner(&mut self, a: usize) {
        self.cells[0].inner.set(a, false);
        self.cells[1].inner.set(a, false);
        self.cells[2].inner.set(a, false);
        self.piece.inner.set(a, false);
        self.team.inner.set(a, false);
    }
    pub fn has_a_piece(&self, index: usize) -> bool {
        //TODO worth having a seperate piece bitfield????
        //Check smaller bits first. more likely to be set.
        //self.cells[0].is_set(a) || self.cells[1].is_set(a) || self.cells[2].is_set(a)
        self.piece.inner[index]
    }

    pub fn get_cell_inner(&self, index: usize) -> Option<(u8, ActiveTeam)> {
        let bit0 = self.cells[0].inner[index] as usize;
        let bit1 = self.cells[1].inner[index] as usize;
        let bit2 = self.cells[2].inner[index] as usize;

        let val = bit0 | bit1 << 1 | bit2 << 2;

        if val == 7 {
            return Some((2, ActiveTeam::Neutral));
        }
        if val == 0 {
            return None;
        }
        let team = if self.team.inner[index] {
            ActiveTeam::White
        } else {
            ActiveTeam::Black
        };
        Some((val as u8, team))
    }
    pub fn get_cell(&self, a: Axial) -> Option<(u8, ActiveTeam)> {
        self.get_cell_inner(mesh::small_mesh::conv(a))
    }

    fn set_coord(&mut self, index: usize, stack: u8) {
        assert!(stack <= 7);
        let bit2 = ((stack >> 2) & 1) != 0;
        let bit1 = ((stack >> 1) & 1) != 0;
        let bit0 = ((stack >> 0) & 1) != 0;

        self.cells[0].inner.set(index, bit0);
        self.cells[1].inner.set(index, bit1);
        self.cells[2].inner.set(index, bit2);

        if stack != 0 {
            self.piece.inner.set(index, true);
        }
    }

    pub fn add_cell_inner(&mut self, a: usize, stack: u8, team: ActiveTeam) {
        match team {
            ActiveTeam::White => self.team.inner.set(a, true),
            ActiveTeam::Black => self.team.inner.set(a, false),
            ActiveTeam::Neutral => {
                let val = if stack == 2 { 7 } else { panic!("impossible") };

                self.set_coord(a, val);
                self.team.inner.set(a, false);
                return;
            }
        }
        self.set_coord(a, stack);
    }
    pub fn add_cell(&mut self, a: Axial, stack: u8, team: ActiveTeam) {
        let a = mesh::small_mesh::conv(a);
        self.add_cell_inner(a, stack, team);
    }
}

#[derive(PartialOrd, Ord, Eq, PartialEq, Copy, Clone, Debug)]
pub enum UnitType {
    Mouse,
    Rabbit,
}

// impl std::fmt::Debug for Tribe {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", "tribe:[")?;
//         for pos in self.warrior.iter_mesh() {
//             write!(f, "{:?},", pos)?;
//         }
//         write!(f, "{}", "]")
//     }
// }

pub fn game_init(_world: &board::MyWorld) -> GameState {
    //let a = 3; //world.white_start().len();

    // let white_mouse = BitField::from_iter(&world.white_start()[0..a]);

    // let black_mouse = BitField::from_iter(&world.black_start()[0..a]);

    // let white_rabbit = BitField::from_iter(&world.white_start()[a..]);

    // let black_rabbit = BitField::from_iter(&world.black_start()[a..]);

    //let powerups = vec![]; //vec![[1, 1], [1, -2], [-2, 1]];

    // let mut fog = BitField::from_iter(Axial::zero().to_cube().range(4).map(|x| x.ax));
    // fog.intersect_with(&world.get_game_cells());
    //let fog=BitField::new();

    let mut cells = Tribe::new();
    cells.add_cell(Axial::from_arr([-1, 2]), 1, ActiveTeam::White);
    cells.add_cell(Axial::from_arr([0, -5]), 1, ActiveTeam::Black);
    cells.add_cell(Axial::from_arr([0, 0]), 2, ActiveTeam::Neutral);

    // use primitive_types::U256;

    // cells.cells[0].inner <<= U256::one();
    // cells.cells[1].inner <<= U256::one();
    // cells.cells[2].inner <<= U256::one();
    // cells.team.inner <<= U256::one();

    let game = GameState {
        factions: cells,
        // env: Environment {
        //     terrain: Terrain {
        //         land: world.land.clone(),
        //         forest: BitField::from_iter([] as [Axial; 0]),
        //         mountain: BitField::from_iter([] as [Axial; 0]),
        //     },
        //     fog,
        //     powerups: powerups.into_iter().map(Axial::from_arr).collect(),
        // },
    };

    // let str="{\"factions\":{\"cells\":{\"cells\":[{\"inner\":[0,180143985094819840,50332928,0]},{\"inner\":[0,0,0,0]},{\"inner\":[0,0,0,0]}],\"team\":{\"inner\":[0,0,50332672,0]}}}}";
    // let game: GameState = serde_json::from_str(str).unwrap();

    // let k = Evaluator::default().absolute_evaluate(&game, world, false);
    // console_dbg!("Current eval=", k);

    game
}
