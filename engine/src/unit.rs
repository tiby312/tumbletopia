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
pub enum Team {
    White = 0,
    Black = 1,
    Neutral = 2,
}

impl std::ops::Not for Team {
    type Output = Team;

    fn not(self) -> Self::Output {
        Team::not(&self)
    }
}

impl std::ops::Neg for Team {
    type Output = Team;

    fn neg(self) -> Self::Output {
        self.not()
    }
}

impl std::ops::SubAssign<unit::Team> for i64 {
    fn sub_assign(&mut self, rhs: unit::Team) {
        *self -= rhs.value();
    }
}
impl std::ops::AddAssign<unit::Team> for i64 {
    fn add_assign(&mut self, rhs: unit::Team) {
        *self += rhs.value();
    }
}
impl std::ops::Add<unit::Team> for i64 {
    type Output = i64;

    fn add(self, rhs: unit::Team) -> Self::Output {
        self + rhs.value()
    }
}

impl std::ops::Sub<unit::Team> for i64 {
    type Output = i64;

    fn sub(self, rhs: unit::Team) -> Self::Output {
        self - rhs.value()
    }
}

impl<T> std::ops::IndexMut<Team> for [T] {
    fn index_mut(&mut self, index: Team) -> &mut Self::Output {
        match index {
            Team::White => &mut self[0],
            Team::Black => &mut self[1],
            _ => {
                unreachable!()
            }
        }
    }
}
impl<T> std::ops::Index<Team> for [T] {
    type Output = T;
    fn index(&self, idx: Team) -> &Self::Output {
        match idx {
            Team::White => &self[0],
            Team::Black => &self[1],
            _ => {
                unreachable!()
            }
        }
    }
}

impl Team {
    pub fn value(&self) -> i64 {
        match self {
            Team::White => 1,
            Team::Black => -1,
            Team::Neutral => 0,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Team::White => 0,
            Team::Black => 1,
            Team::Neutral => unreachable!(),
        }
    }
    pub fn is_white(&self) -> bool {
        if let Team::White = self {
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
            Team::White => Team::Black,
            Team::Black => Team::White,
            Team::Neutral => Team::Neutral,
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

#[derive(Serialize, Deserialize, Default, Clone, Debug, Hash, Eq, PartialEq)]

pub struct GameStateTotal {
    //0 is white fog. 1 is black fog
    pub fog: [SmallMesh; 2],
    pub tactical: GameState,
}

impl GameStateTotal {}
//Additionally removes need to special case animation.
#[derive(Serialize, Deserialize, Default, Clone, Debug, Hash, Eq, PartialEq)]
pub struct GameState {
    pub factions: Tribe,
}

#[must_use]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameOver {
    WhiteWon,
    BlackWon,
    Tie,
}

impl GameState {
    pub fn bake_fog(&self, fog: &SmallMesh) -> GameState {
        let mut gg = self.clone();
        // let fog = match team {
        //     ActiveTeam::White => &self.fog[0],
        //     ActiveTeam::Black => &self.fog[1],
        //     ActiveTeam::Neutral => unreachable!(),
        // };

        //TODO use bit and/oring
        for a in fog.iter_mesh(Axial::zero()) {
            gg.factions.remove(a);
            gg.factions.add_cell(a, 6, Team::Neutral);
        }

        gg
    }
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
        _team: Team,
        history: &MoveHistory,
    ) -> Option<GameOver> {
        let score_data = self.score(world);

        let (a, b) = match &history.inner[..] {
            [.., a, b] => (a, b),
            _ => return None,
        };

        if a.0.moveto == hex::PASS_MOVE_INDEX && b.0.moveto == hex::PASS_MOVE_INDEX {
            //return None;
            if score_data.white > score_data.black {
                return Some(GameOver::WhiteWon);
            } else if score_data.white < score_data.black {
                return Some(GameOver::BlackWon);
            } else {
                return Some(GameOver::Tie);
            }
        }

        None
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ScoreData {
    pub white: usize,
    pub black: usize,
    pub neutral: usize,
}
impl GameState {
    pub fn score(&self, world: &MyWorld) -> ScoreData {
        //let total_num = world.get_game_cells().inner.count_ones();
        let mut neutral = 0;
        let game = self;
        let mut white_score = 0;
        let mut black_score = 0;
        for index in world.get_game_cells().inner.iter_ones() {
            let mut num_white = 0;
            let mut num_black = 0;
            for (_, rest) in game.factions.iter_end_points(world, index) {
                if let Some(EndPoint { team, .. }) = rest {
                    match team {
                        Team::White => num_white += 1,
                        Team::Black => num_black += 1,
                        Team::Neutral => {}
                    }
                }
            }

            if let Some((_height, tt)) = game.factions.get_cell_inner(index) {
                //let height = height as i8;
                match tt {
                    Team::White => {
                        white_score += 1;
                        // if num_black >= height {
                        //     black_score += 1
                        // }
                    }
                    Team::Black => {
                        black_score += 1;
                        // if num_white >= height {
                        //     white_score += 1;
                        // }
                    }
                    Team::Neutral => {
                        neutral += 1;
                    }
                }
            } else {
                let ownership = num_white - num_black;

                if ownership > 0 {
                    white_score += 1;
                } else if ownership < 0 {
                    black_score += 1;
                } else {
                    neutral += 1;
                }
            };
        }
        ScoreData {
            white: white_score,
            black: black_score,
            neutral,
        }
    }

    // pub fn threat_score(&self, world: &MyWorld) -> (usize, usize) {
    //     let total_num = world.get_game_cells().inner.count_ones();

    //     let game = self;
    //     let mut white_score = 0;
    //     let mut black_score = 0;
    //     for index in world.get_game_cells().inner.iter_ones() {
    //         let mut num_white = 0;
    //         let mut num_black = 0;
    //         for (_, rest) in game.factions.iter_end_points(world, index) {
    //             if let Some((_, team)) = rest {
    //                 match team {
    //                     ActiveTeam::White => num_white += 1,
    //                     ActiveTeam::Black => num_black += 1,
    //                     ActiveTeam::Neutral => {}
    //                 }
    //             }
    //         }

    //         if let Some((height, tt)) = game.factions.get_cell_inner(index) {
    //             let height = height as i8;
    //             match tt {
    //                 ActiveTeam::White => {
    //                     white_score += 1;
    //                     if num_black >= height {
    //                         black_score += 1000
    //                     }
    //                 }
    //                 ActiveTeam::Black => {
    //                     black_score += 1;
    //                     if num_white >= height {
    //                         white_score += 1000;
    //                     }
    //                 }
    //                 ActiveTeam::Neutral => {}
    //             }
    //         } else {
    //             let ownership = num_white - num_black;

    //             if ownership > 0 {
    //                 white_score += 1;
    //             } else if ownership < 0 {
    //                 black_score += 1;
    //             }
    //         };
    //     }
    //     (white_score, black_score)
    // }
}

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

pub fn ray(
    start: Axial,
    dd: hex::HDir,
    world: &board::MyWorld,
) -> (i8, impl Iterator<Item = isize> + use<'_>) {
    let stride = board::STRIDES[dd as usize] as isize;
    let dis = board::dis_to_hex_of_hexagon(start, dd, world.radius as i8);
    let mut index2 = start.to_index() as isize;

    // assert!(
    //     world.get_game_cells().inner[index2 as usize],
    //     "uhoh {:?}",
    //     world.format(&start)
    // );
    (
        dis,
        (1..dis).map(move |_d| {
            index2 += stride;
            //assert!(index2 > 0);

            assert!(
                world.get_game_cells().inner[index2 as usize],
                // "fail {}:{}:{:?}:{:?}",
                // d,
                // dis,
                // dd,
                // start.to_letter_coord(world.radius as i8)
            );
            index2
        }),
    )
}

pub struct EndPoint {
    pub index: usize,
    pub height: i8,
    pub team: Team,
}

impl Tribe {
    //TODO rename
    pub fn doop(&self, index: usize, world: &MyWorld) -> SmallMesh {
        let mut s = SmallMesh::new();

        for i in 0..6 {
            let dd = hex::HDir::from(i as u8);

            let stride = board::STRIDES[i] as isize;
            let dis =
                board::dis_to_hex_of_hexagon(Axial::from_index(index), dd, world.radius as i8);
            let mut index2 = index as isize;

            for _ in 0..dis {
                index2 += stride;

                s.inner.set(index2 as usize, true);

                if let Some(_) = self.get_cell_inner(index2 as usize) {
                    break;
                }
            }
        }

        s
    }
    pub fn iter_end_points<'a>(
        &'a self,
        world: &'a board::MyWorld,
        index: usize,
    ) -> impl Iterator<Item = (i8, Option<EndPoint>)> + use<'a> {
        hex::HDir::all().map(move |dd| {
            let (dis, it) = ray(Axial::from_index(index), dd, world);
            for (d, index2) in it.enumerate() {
                if let Some(pp) = self.get_cell_inner(index2 as usize) {
                    return (
                        d as i8 + 1,
                        Some(EndPoint {
                            index: index2 as usize,
                            height: pp.0 as i8,
                            team: pp.1,
                        }),
                    );
                }
            }

            (dis, None)
        })

        // core::array::from_fn(|i| {

        // })
    }

    pub fn new() -> Tribe {
        Tribe {
            cells: std::array::from_fn(|_| SmallMesh::new()),
            team: SmallMesh::new(),
            ice: SmallMesh::new(),
            piece: SmallMesh::new(),
        }
    }

    pub fn remove(&mut self, a: Axial) {
        let a = a.to_index();
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

    pub fn get_cell_inner(&self, index: usize) -> Option<(u8, Team)> {
        if !self.piece.inner[index as usize] {
            return None;
        }

        let bit0 = self.cells[0].inner[index] as usize;
        let bit1 = self.cells[1].inner[index] as usize;
        let bit2 = self.cells[2].inner[index] as usize;

        let val = bit0 | bit1 << 1 | bit2 << 2;

        if val == 7 {
            return Some((2, Team::Neutral));
        }
        if val == 0 {
            return Some((6, Team::Neutral));
        }

        let team = if self.team.inner[index] {
            Team::White
        } else {
            Team::Black
        };
        Some((val as u8, team))
    }
    pub fn get_cell(&self, a: Axial) -> Option<(u8, Team)> {
        self.get_cell_inner(a.to_index())
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

    pub fn add_cell_inner(&mut self, a: usize, stack: u8, team: Team) {
        match team {
            Team::White => self.team.inner.set(a, true),
            Team::Black => self.team.inner.set(a, false),
            Team::Neutral => {
                let val = if stack == 2 {
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
    pub fn add_cell(&mut self, a: Axial, stack: u8, team: Team) {
        let a = a.to_index();
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

// pub fn parse_replay_string(s: &str, world: &MyWorld) -> Option<(Map, MoveHistory)> {
//     let mut s = s.split(":");

//     let map = s.next()?;

//     let Ok(map) = Map::load(&map, world) else {
//         return None;
//     };

//     let moves = s.next()?;

//     let (mut g, start_team) = GameStateTotal::new(world, &map);
//     let mut mh = MoveHistory::new();
//     for (f, team) in moves.split_terminator(',').zip(start_team.iter()) {
//         let m = ActualMove::from_str(f)?;

//         let effect = m.apply(team, &mut g.tactical, world);
//         g.update_fog(world, team);
//         mh.inner.push((m, effect));
//     }

//     Some((map, mh))
// }

pub fn replay_string(_moves: &MoveHistory, _world: &MyWorld) -> Result<String, std::fmt::Error> {
    let s = String::new();

    //let map_str = map.save(world).unwrap();
    //TODO update this!!!
    //todo!();
    // write!(&mut s, "{}:", map_str)?;

    // for m in moves.inner.iter() {
    //     let mut kk = String::new();
    //     m.0.as_text(&mut kk).unwrap();
    //     write!(&mut s, "{},", kk)?;
    // }

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
                    Team::White => {
                        white.inner.set(a, true);
                    }
                    Team::Black => {
                        black.inner.set(a, true);
                    }
                    Team::Neutral => {
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

    //Map::load("wwwwwwwww-if--wwwwiiffw-wwwww----iw-www-w----i--2ww-2w--iiw-w-ww------f----wwwwff--f-----wwwwfiiw-ffw-w-www-ii--w---w1ww----i---w-ww1-wwi----www--iii-wwww---fiwwwwwwwwww",world).unwrap()
    //Map::load("wwwwwwwww-i---iiwwii--i-iiwwi----ii-iww-i----i--2ww-2i--iii-i-ww-----------iwwi-----f----iiww-iii---i-i-iww-ii--i---i1ww----i---i-ww1-iii----wwi--iii--wwi----i-wwwwwwwww",world).unwrap()
    Map::load("----------i1--ii--ii--i-ii--i--2-ii-i", world).unwrap()
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

impl GameStateTotal {
    //TODO make part of GameState
    pub fn new(world: &board::MyWorld, map: &unit::Map) -> (GameStateTotal, Team) {
        //let map = &world.map;

        let mut cells = Tribe::new();

        for f in map.white.iter_mesh(Axial::zero()) {
            cells.add_cell(f, 1, Team::White);
        }

        for f in map.black.iter_mesh(Axial::zero()) {
            cells.add_cell(f, 1, Team::Black);
        }

        for f in map.forests.iter_mesh(Axial::zero()) {
            cells.add_cell(f, 1, Team::Neutral);
        }

        for m in map.water.iter_mesh(Axial::zero()) {
            cells.add_cell(m, 6, Team::Neutral);
        }

        for w in map.ice.iter_mesh(Axial::zero()) {
            cells.ice.add(w);
        }

        let game = GameState { factions: cells };

        let mut game_total = GameStateTotal {
            tactical: game,
            fog: std::array::from_fn(|_| SmallMesh::new()),
        };

        //Fill everything with fog.
        //game_total.fog[0].inner |= world.get_game_cells().inner;
        //game_total.fog[1].inner |= world.get_game_cells().inner;

        game_total.update_fog(&world, Team::White);
        game_total.update_fog(&world, Team::Black);

        (game_total, Team::White)
    }
}

// mod test {

//     fn doop() {
//         let foo = "
// -----s--s-eb-ev-b--
// -c-c-s-tct---cs--c-
// tc-s-d-re-srces-s--
// --brc----dc--r-sr-r
// -r---s--rtd-bbb-c--
// cs--cs----s---csc--
// bs----s--d--c--s--c
// ducd-uc-d-ub-dubd-u
// test-est--erte-rte-
// b-rbbr---k---ds-tds
// c---rc-b-uc-r-s--sc
// s--cbs---ds---d--bs
// c--sc---s-e--ses-s-
// rr-dr----d----rr-dr
// ssetseteeessssettse
// d--sd-d-sdd--s---s-
// bcs-d-ss-e--sudtc--
// bb---cs--d----s---r
// -sr-se--se--se----r
// sccrbs--ses--ses-sc
// c-b--s-r-ts-ccd----
// rc-rr----d----rc-rr
// -r-e--rte--r-e--te-
// rdsr-ds--dd-r-ds-ds
// -bbrr----c-bs-rb-r-
// s-s-ddssd-ds-dd-s-s
// c-rse-rr-e-ss-e-s-c
// t---tt-d---d--dttd-
// d--d--t--d-t---dtt-
// -t--d--t---t-d-dd-t
// --t-td-t-t-dd--d---
// -rrs-r---cb---bb---
// s--c--s--t---cdbc--
// r--c----cs-r--d--b-
// s-cr-d---er-rtdt-c-
// -se-se--ser-se-t---
// --es-e--see--s-e-ss
// c--ct-------c-t--cc
// ccc-e-ss----s-----d
// --ttd-tt-d---dtdd--
// ---d-c-s--d-rtst-dc
// c--ctc-c-d---ssss--
// tddt-t--dt---t-d-d-
// t-dt---t-t-d--dtdd-
// -tbc--b-s--c----b-t
// bbbr-rc--dt----r---
// --d---ss---s----bcb
// bb-t-bbsrd-s----s--
// dcc-surr-f-s--sd-sd
// s---s-bdd-tct---b-b
// -s--ds---cs-s-ds---
// --ddttt------d-e---
// ----sc-s-----ccssc-
// duc----tt---e-b-td-
// -----ds--tdsc--er-t
// sdt--ect-----e-sc-s
// -tccd--t-t-ct--rdd-
// --s---s-cttc-d-cr-c
// cs--c----r--b----dr
// -r--r-rbbcr---dr-bs
// -r--r-r-bct---c--b-
// -r--r-r-d-t-c-d--b-
// "
//         .trim()
//         .split('\n');

//         //TODO make sure first player wins in all these cases using the AI.
//     }
// }
