use board::MyWorld;
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
    pub fn game_is_over(
        &self,
        world: &board::MyWorld,
        _team: ActiveTeam,
        history: &MoveHistory,
    ) -> Option<GameOver> {
        let (a, b) = match &history.inner[..] {
            [.., a, b] => (a, b),
            _ => return None,
        };

        if a.0.moveto != moves::PASS_MOVE_INDEX || b.0.moveto != moves::PASS_MOVE_INDEX {
            return None;
        }

        let game = self;
        let mut score = 0;
        let mut stack_count = 0;
        let mut territory_count = 0;
        let mut strength = 0;
        let mut contested = 0;
        let mut unseen = 0;
        for index in world.get_game_cells().inner.iter_ones() {
            let mut num_white = 0;
            let mut num_black = 0;
            for (_, rest) in game.factions.iter_end_points(world, index) {
                if let Some((_, team)) = rest {
                    match team {
                        ActiveTeam::White => num_white += 1,
                        ActiveTeam::Black => num_black += 1,
                        ActiveTeam::Neutral => {}
                    }
                }
            }

            if let Some((height, tt)) = game.factions.get_cell_inner(index) {
                let height = height as i64;

                let curr_strength = match tt {
                    ActiveTeam::White => height.max(num_white - 1),
                    ActiveTeam::Black => -height.max(num_black - 1),
                    ActiveTeam::Neutral => 0,
                };

                strength += curr_strength;

                stack_count += 1;

                match tt {
                    ActiveTeam::White => {
                        if num_black > height {
                            score -= 1
                        } else {
                            score += 1
                        }
                    }
                    ActiveTeam::Black => {
                        if num_white > height {
                            score += 1
                        } else {
                            score -= 1
                        }
                    }
                    ActiveTeam::Neutral => {}
                }
            } else {
                let ownership = num_white - num_black;

                if ownership > 0 {
                    score += ownership;
                    territory_count += 1;
                } else if ownership < 0 {
                    score += ownership;
                    territory_count += 1;
                } else {
                    //The diff is zero, so if num_white is positive, so too must be black indicating they are contesting.
                    if num_white > 0 {
                        contested += 1
                    } else {
                        unseen += 1;
                    }
                }
            };
        }

        Some(if score > 0 {
            GameOver::WhiteWon
        } else if score < 0 {
            GameOver::BlackWon
        } else {
            GameOver::Tie
        })
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
    pub ice: SmallMesh,
    //This just signifies if there is a number in cells.
    //This way you can just check one mesh to see if a piece is there or not
    //instead of checking 3
    pub piece: SmallMesh,
}

impl Tribe {
    pub fn doop(&self, index: usize, world: &MyWorld) -> SmallMesh {
        let mut s = SmallMesh::new();

        for i in 0..6 {
            let dd = hex::HDir::from(i as u8);

            let stride = board::STRIDES[i] as isize;
            let dis = board::dis_to_hex_of_hexagon(
                mesh::small_mesh::inverse(index),
                dd,
                world.radius as i8,
            );
            let mut index2 = index as isize;

            for d in 0..dis {
                index2 += stride;

                s.inner.set(index2 as usize, true);

                if let Some(pp) = self.get_cell_inner(index2 as usize) {
                    break;
                }
            }
        }

        s
    }
    pub fn iter_end_points(
        &self,
        world: &board::MyWorld,
        index: usize,
    ) -> [(i8, Option<(u8, ActiveTeam)>); 6] {
        core::array::from_fn(|i| {
            let dd = hex::HDir::from(i as u8);

            let stride = board::STRIDES[i] as isize;
            let dis = board::dis_to_hex_of_hexagon(
                mesh::small_mesh::inverse(index),
                dd,
                world.radius as i8,
            );
            let mut index2 = index as isize;

            for d in 0..dis {
                index2 += stride;

                if let Some(pp) = self.get_cell_inner(index2 as usize) {
                    return (d + 1, Some(pp));
                }
            }

            (dis, None)
        })
    }

    pub fn new() -> Tribe {
        Tribe {
            cells: [0; 3].map(|_| SmallMesh::new()),
            team: SmallMesh::new(),
            ice: SmallMesh::new(),
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
        if !self.piece.inner[index as usize] {
            return None;
        }

        let bit0 = self.cells[0].inner[index] as usize;
        let bit1 = self.cells[1].inner[index] as usize;
        let bit2 = self.cells[2].inner[index] as usize;

        let val = bit0 | bit1 << 1 | bit2 << 2;

        if val == 7 {
            return Some((1, ActiveTeam::Neutral));
        }
        if val == 0 {
            return Some((6, ActiveTeam::Neutral));
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

        //if stack != 0 {
        self.piece.inner.set(index, true);
        //}
    }

    pub fn add_cell_inner(&mut self, a: usize, stack: u8, team: ActiveTeam) {
        match team {
            ActiveTeam::White => self.team.inner.set(a, true),
            ActiveTeam::Black => self.team.inner.set(a, false),
            ActiveTeam::Neutral => {
                let val = if stack == 1 {
                    7
                } else if stack == 6 {
                    0
                } else {
                    panic!("impossible")
                };

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

//need 8 layers of map
//each map needs

pub fn parse_replay_string(s: &str, world: &MyWorld) -> Option<(Map, MoveHistory)> {
    let mut s = s.split(":");

    let map = s.next()?;

    let Ok(map) = Map::load(&map, world) else {
        return None;
    };

    let moves = s.next()?;

    let (mut g, start_team) = GameState::new(world, &map);
    let mut mh = MoveHistory::new();
    for (f, team) in moves.split_terminator(',').zip(start_team.iter()) {
        let m = ActualMove::from_str(f)?;
        let effect = m.apply(team, &mut g, world);
        mh.inner.push((m, effect));
    }

    Some((map, mh))
}

pub fn replay_string(
    map: &Map,
    moves: &MoveHistory,
    world: &MyWorld,
) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;
    let mut s = String::new();

    let map_str = map.save(world).unwrap();

    write!(&mut s, "{}:", map_str)?;

    for m in moves.inner.iter() {
        let mut kk = String::new();
        m.0.as_text(&mut kk).unwrap();
        write!(&mut s, "{},", kk)?;
    }

    Ok(s)
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Map {
    pub ice: SmallMesh,
    pub water: SmallMesh,
    pub forests: SmallMesh,
    pub white: SmallMesh,
    pub black: SmallMesh,
}

impl unit::Map {
    pub fn load(s: &str, world: &MyWorld) -> Result<unit::Map, ()> {
        // use base64::prelude::*;
        // let k = BASE64_STANDARD_NO_PAD.decode(s).map_err(|_| LoadError)?;
        // let k = miniz_oxide::inflate::decompress_to_vec(&k).map_err(|_| LoadError)?;
        // Ok(postcard::from_bytes(&k).map_err(|_| LoadError)?)

        let mut ice = SmallMesh::new();
        let mut forests = SmallMesh::new();
        let mut mountains = SmallMesh::new();
        let mut white = SmallMesh::new();
        let mut black = SmallMesh::new();

        let mut s = s.chars();

        for a in world.get_game_cells().inner.iter_ones() {
            let Some(c) = s.next() else {
                return Err(());
            };

            match c {
                'i' => ice.inner.set(a, true),
                'f' => forests.inner.set(a, true),
                'w' => mountains.inner.set(a, true),
                '1' => white.inner.set(a, true),
                '2' => black.inner.set(a, true),
                '-' => continue,
                _ => return Err(()),
            }
        }

        Ok(unit::Map {
            ice,
            water: mountains,
            forests,
            white,
            black,
        })
    }
    pub fn save(&self, world: &MyWorld) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;
        let mut s = String::new();

        //write!(&mut s,"m{}=[",world.radius)?;
        for a in world.get_game_cells().inner.iter_ones() {
            if self.ice.inner[a] {
                write!(&mut s, "i")?;
            } else if self.forests.inner[a] {
                write!(&mut s, "f")?;
            } else if self.water.inner[a] {
                write!(&mut s, "w")?;
            } else if self.white.inner[a] {
                write!(&mut s, "1")?;
            } else if self.black.inner[a] {
                write!(&mut s, "2")?;
            } else {
                write!(&mut s, "-")?;
            }
        }
        //write!(&mut s,"]")?;
        Ok(s)

        // use base64::prelude::*;

        // let k = postcard::to_allocvec(self).unwrap();

        // let k = miniz_oxide::deflate::compress_to_vec(&k, 10);
        // BASE64_STANDARD_NO_PAD.encode(k)
    }
}

impl Map {
    pub fn from_game_state(game: &GameState, world: &board::MyWorld) -> Option<Map> {
        let water = game.factions.ice.clone();

        let mut white = SmallMesh::new();
        let mut black = SmallMesh::new();
        let mut mountains = SmallMesh::new();
        let mut forests = SmallMesh::new();

        for a in world.get_game_cells().inner.iter_ones() {
            if let Some((height, team)) = game.factions.get_cell_inner(a) {
                match team {
                    ActiveTeam::White => {
                        white.inner.set(a, true);
                    }
                    ActiveTeam::Black => {
                        black.inner.set(a, true);
                    }
                    ActiveTeam::Neutral => {
                        if height == 6 {
                            mountains.inner.set(a, true);
                        } else if height == 1 {
                            forests.inner.set(a, true);
                        }
                    }
                }
            }
        }

        Some(Map {
            ice: water,
            water: mountains,
            forests,
            white,
            black,
        })
    }
}

pub fn default_map(world: &board::MyWorld) -> Map {
    //Map::load("----------w-------ww--m----------w------m----m------2m--ww--m-w------------ww-------f-----ww--wwm---m-m-w---ww--m---m1------w---m-----mmw------m--www-m-mm----w----------",world).unwrap()

    //Map::load("----------if------iiffw----------i------w----w----i-2w--ii--w-i-------f----ii------f------ii--iiw-ffw-w-i---ii--w---w1------i---w-----wwi-----1w--iii-w-ww---fi-2-----ff-",world).unwrap()

    Map::load("wwwwwwwww-if--wwwwiiffw-wwwww----iw-www-w----i--2ww-2w--iiw-w-ww------f----wwwwff--f-----wwwwfiiw-ffw-w-www-ii--w---w1ww----i---w-ww1-wwi----www--iii-wwww---fiwwwwwwwwww",world).unwrap()
    // let mut mountains = SmallMesh::new();
    // let mut water = SmallMesh::new();
    // let mut forests = SmallMesh::new();

    // let mountains2 = [
    //     [1, -3],
    //     [1, 1],
    //     [-5, 3],
    //     [2, -1],
    //     [-3, 3],
    //     [-4, -2],
    //     [-3, -2],
    //     [-2, -2],
    // ];

    // for a in mountains2 {
    //     mountains.add(Axial::from_arr(a));
    // }

    // let water2 = [[-2, 2], [-2, 1], [-4, 3], [3, -2], [4, -2], [5, -3]];

    // for a in water2 {
    //     water.add(Axial::from_arr(a));
    // }

    // forests.add(Axial::from_arr([0, 0]));

    // let start1 = Axial { q: -1, r: 2 };
    // let start2 = Axial { q: 0, r: -5 };

    // Map {
    //     water,
    //     mountains,
    //     forests,
    //     start1,
    //     start2,
    // }
}

impl GameState {
    //TODO make part of GameState
    pub fn new(world: &board::MyWorld, map: &unit::Map) -> (GameState, ActiveTeam) {
        //let map = &world.map;

        let mut cells = Tribe::new();

        for f in map.white.iter_mesh(Axial::zero()) {
            cells.add_cell(f, 1, ActiveTeam::White);
        }

        for f in map.black.iter_mesh(Axial::zero()) {
            cells.add_cell(f, 1, ActiveTeam::Black);
        }

        for f in map.forests.iter_mesh(Axial::zero()) {
            cells.add_cell(f, 1, ActiveTeam::Neutral);
        }

        for m in map.water.iter_mesh(Axial::zero()) {
            cells.add_cell(m, 6, ActiveTeam::Neutral);
        }

        for w in map.ice.iter_mesh(Axial::zero()) {
            cells.ice.add(w);
        }

        let game = GameState { factions: cells };

        (game, ActiveTeam::White)
    }
}
