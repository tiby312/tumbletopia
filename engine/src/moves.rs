use hex::HDir;

use super::*;

use crate::{mesh::small_mesh::SmallMesh, unit::PieceType};

pub struct EndPoints<T> {
    inner: [T; 6],
    num_first: usize,
    second_start_index: usize,
}
impl<T> EndPoints<T> {
    pub fn new() -> EndPoints<T>
    where
        T: Default,
    {
        EndPoints {
            inner: [0; 6].map(|_| std::default::Default::default()),
            num_first: 0,
            second_start_index: 6,
        }
    }
    pub fn add_first(&mut self, a: T) {
        self.inner[self.num_first] = a;
        self.num_first += 1;
    }
    pub fn add_second(&mut self, a: T) {
        self.second_start_index -= 1;
        self.inner[self.second_start_index] = a;
    }
    pub fn first_len(&self) -> usize {
        self.num_first
    }
    pub fn second_len(&self) -> usize {
        6 - self.second_start_index
    }
    pub fn iter_first(&self) -> impl Iterator<Item = &T> {
        self.inner[..self.num_first].iter()
    }
    pub fn iter_second(&self) -> impl Iterator<Item = &T> {
        self.inner[self.second_start_index..].iter()
    }
}

impl crate::unit::GameStateTotal {
    pub fn update_fog(&mut self, world: &board::MyWorld, team: Team) {
        let fog = match team {
            Team::White => &mut self.fog[0],
            Team::Black => &mut self.fog[1],
            Team::Neutral => unreachable!(),
        };

        // let pieces = match team {
        //     ActiveTeam::White => {
        //         self.tactical.factions.piece.inner & self.tactical.factions.team.inner
        //     }
        //     ActiveTeam::Black => {
        //         return;
        //         self.tactical.factions.piece.inner & !self.tactical.factions.team.inner
        //     }
        //     ActiveTeam::Neutral => unreachable!(),
        // };

        for a in world.get_game_cells().inner.iter_ones() {
            let fa = Axial::from_index(&a);

            match self.tactical.factions.get_cell_inner(a) {
                &unit::GameCell::Piece(unit::Piece {
                    height: stack_height,
                    team: tt,
                    ..
                }) => {
                    if tt == team {
                        for b in fa
                            .to_cube()
                            .range(stack_height.to_num().try_into().unwrap())
                        {
                            if !world.get_game_cells().is_set(*b) {
                                continue;
                            }

                            fog.set_coord(*b, false);
                        }
                    }
                }
                unit::GameCell::Empty => {}
            }
        }
    }

    pub fn update_fog_spokes(
        &mut self,
        _world: &board::MyWorld,
        _team: Team,
        _spoke_info: &moves::SpokeInfo,
    ) {
        return;

        //TODO also need to convert ice blacks to grass blocks to emulate visition mode???
        //TODO also replace enemy units with mountains to allow suicidal moves
        // let res = self
        //     .tactical
        //     .bake_fog(&self.fog[team])
        //     .generate_possible_moves_movement(world, team, spoke_info);

        // let fog = match team {
        //     Team::White => &mut self.fog[0],
        //     Team::Black => &mut self.fog[1],
        //     Team::Neutral => unreachable!(),
        // };

        // fog.inner &= !res.0.inner;

        // let pieces = match team {
        //     Team::White => self.tactical.factions.piece.inner & self.tactical.factions.team.inner,
        //     Team::Black => self.tactical.factions.piece.inner & !self.tactical.factions.team.inner,
        //     Team::Neutral => unreachable!(),
        // };
        // fog.inner &= !pieces;

        // for a in pieces.iter_ones() {
        //     let fa = Axial::from_index(a);

        //     for a in hex::HDir::all() {
        //         let mut pos = fa;
        //         loop {
        //             pos = pos.advance(a);

        //             if !world.get_game_cells().is_set(pos) {
        //                 break;
        //             }

        //             if !res.0.is_set(pos) {
        //                 let np = pos; //pos.advance(a);
        //                 if fog.is_set(np) {
        //                     fog.set_coord(np, false);
        //                 }

        //                 break;
        //             }
        //         }
        //     }
        // }
    }
}

// pub enum LoudMove {
//     Capture(usize),
//     Reinforcement(usize),
// }

#[derive(Debug, Copy, Clone)]
pub enum MoveType {
    Capture,
    Reinforce,
    Fresh,
    Suicidal,
}

impl MoveType {
    pub fn is_suicidal(&self) -> bool {
        if let MoveType::Suicidal = self {
            true
        } else {
            false
        }
    }
}

pub use spoke::SpokeInfo;
mod spoke {
    use super::*;
    #[derive(PartialEq, Eq, Copy, Clone, Debug)]
    pub struct SpokeInfo {
        //pub data: [bitvec::BitArr!(for 256*6); 2],
        pub data: [SpokeCell; 256],
    }

    // 3 bits for num white
    // 3 bits for num black
    // 2 bits left over

    //0
    //1
    //2
    //3
    //4
    //5
    //6

    //   5   5   5   5   5   5
    // |---|---|---|---|---|---|
    //
    //
    //
    //
    // impl std::cmp::PartialEq for SpokeInfo {
    //     fn eq(&self, other: &Self) -> bool {
    //         self.data == other.data
    //     }
    // }

    // #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    // pub enum Thing {
    //     None,
    //     White,
    //     Black,
    //     Neutral,
    // }

    // impl Thing {
    //     pub fn value(&self) -> i64 {
    //         match self {
    //             Thing::None => 0,
    //             Thing::White => 1,
    //             Thing::Black => -1,
    //             Thing::Neutral => 0,
    //         }
    //     }
    // }
    #[derive(PartialEq, Eq, Copy, Clone, Debug)]
    pub struct SpokeCell {
        raw: [Team; 6],
        pub num_attack: [i64; 2],
    }

    pub struct SpokeTempInfo {
        data: [(i8, Option<unit::EndPoint>); 6],
    }

    impl SpokeInfo {
        pub fn new(game: &GameState, world: &MyWorld) -> Self {
            //tddt-t--dt---t-d-d-
            let mut spoke_info = SpokeInfo {
                data: [SpokeCell {
                    raw: [Team::Neutral; 6],
                    num_attack: [0; 2],
                }; 256],
            };

            //Update spoke info
            for index in world.get_game_cells().inner.iter_ones() {
                for (i, (_, rest)) in game.factions.iter_end_points(world, index).enumerate() {
                    let v = if let Some(e) = rest {
                        e.piece.team
                    } else {
                        Team::Neutral
                    };
                    spoke_info.set(index, HDir::from(i as u8), v);
                    debug_assert_eq!(v, spoke_info.get(index, HDir::from(i as u8)));
                }
            }

            spoke_info
        }

        pub fn process_move_better(
            &mut self,
            a: NormalMove,
            team: Team,
            world: &board::MyWorld,
            game: &GameState,
        ) -> SpokeTempInfo {
            if a.coord.0 == hex::PASS_MOVE_INDEX {
                return SpokeTempInfo {
                    data: std::array::from_fn(|_| (0, None)),
                };
            }
            let index = a.coord.0;
            debug_assert!(
                world.get_game_cells().inner[index as usize],
                "uhoh {:?}",
                world.format(&a.coord)
            );
            let mut it = hex::HDir::all().map(move |dd| {
                let (dis, it) = unit::ray(Axial::from_index(&index), dd, world);
                //let mut it=it.peekable();

                // if let Some(&index2)=it.peek(){
                //     if game.factions.get_cell_inner(index2 as usize).is_none() {
                //         if let Some(foo)=self.get(index2 as usize,dd.rotate_180()){
                //             match foo{
                //                 Team::White | Team::Black=> {
                //                     if foo==team{
                //                         // don't need to do anything for empty cells
                //                     }else{
                //                         //add one to this team
                //                         //subtract one from that team
                //                     }
                //                 },
                //                 Team::Neutral => {
                //                     //just add one for this team to all empty cells
                //                 },
                //             }
                //         }else{
                //             //just add one for this team to all the empty cells
                //         }
                //     }
                // }

                for (d, index2) in it.enumerate() {
                    debug_assert!(index != index2 as usize);
                    self.set(index2 as usize, dd.rotate_180(), team);

                    match game.factions.get_cell_inner(index2 as usize) {
                        &unit::GameCell::Piece(piece) => {
                            self.set(index, dd, piece.team);

                            return (
                                d as i8 + 1,
                                Some(unit::EndPoint {
                                    index: index2 as usize,
                                    piece,
                                }),
                            );
                        }
                        unit::GameCell::Empty => {}
                    }
                }
                self.set(index, dd, Team::Neutral);
                (dis, None)
            });

            SpokeTempInfo {
                data: std::array::from_fn(|_| it.next().unwrap()),
            }
        }

        pub fn undo_move(
            &mut self,
            a: NormalMove,
            effect: &move_build::MoveEffect,
            _team: Team,
            _world: &board::MyWorld,
            _game: &GameState,
            spoke_temp: SpokeTempInfo,
        ) {
            if a.coord.0 == hex::PASS_MOVE_INDEX {
                return;
            }
            let index = a.coord.0;

            let arr = &spoke_temp.data;

            for (hexdir, (dis, rest)) in HDir::all().zip(arr.iter()) {
                let st = if let &Some(unit::EndPoint { .. }) = rest {
                    1
                } else {
                    0
                };

                let stride = board::STRIDES[hexdir as usize] as isize;

                let mut index2: isize = index as isize;

                let oppt = if let Some((_, t2)) = effect.destroyed_unit {
                    t2
                } else {
                    if let (_, Some(end)) = &arr[hexdir.rotate_180() as usize] {
                        end.piece.team
                    } else {
                        Team::Neutral
                    }
                };

                for _ in 0..*dis - 1 + st {
                    index2 += stride;
                    self.set(index2 as usize, hexdir.rotate_180(), oppt);
                }
            }
        }

        fn set(&mut self, index: usize, dir: HDir, new_team: Team) {
            let cc = &mut self.data[index];

            let curr_team = cc.raw[dir as usize];

            if new_team == curr_team {
                return;
            }

            if new_team != Team::Neutral {
                cc.num_attack[new_team] += 1;
            }

            if curr_team != Team::Neutral {
                cc.num_attack[curr_team] -= 1;
            }
            cc.raw[dir as usize] = new_team;

            // let tt = match val {
            //     None => Thing::None,
            //     Some(Team::White) => Thing::White,
            //     Some(Team::Black) => Thing::Black,
            //     Some(Team::Neutral) => Thing::Neutral,
            // };

            // let new_value = tt.value();
            // let old_value = self.data[index].raw[dir as usize].value();

            // match (old_value, new_value) {
            //     (-1, -1) => {}
            //     (-1, 0) => {
            //         self.data[index].num_attack[1] -= 1;
            //     }
            //     (0, -1) => {
            //         self.data[index].num_attack[1] += 1;
            //     }
            //     (-1, 1) => {
            //         self.data[index].num_attack[0] += 1;
            //         self.data[index].num_attack[1] -= 1;
            //     }
            //     (1, 1) => {}
            //     (1, 0) => {
            //         self.data[index].num_attack[0] -= 1;
            //     }
            //     (0, 1) => {
            //         self.data[index].num_attack[0] += 1;
            //     }
            //     (1, -1) => {
            //         self.data[index].num_attack[0] -= 1;
            //         self.data[index].num_attack[1] += 1;
            //     }
            //     (0, 0) => {}
            //     _ => unreachable!("{:?} {:?}", old_value, new_value),
            // }

            // self.data[index].raw[dir as usize] = tt;
        }
        pub fn get(&self, index: usize, dir: HDir) -> Team {
            self.data[index].raw[dir as usize]
            // match self.data[index].raw[dir as usize] {
            //     Thing::None => None,
            //     Thing::White => Some(Team::White),
            //     Thing::Black => Some(Team::Black),
            //     Thing::Neutral => Some(Team::Neutral),
            // }
        }

        pub fn get_num_attack(&self, index: usize) -> &[i64; 2] {
            let foo = &self.data[index];
            &foo.num_attack
        }
    }
}

impl GameState {
    pub fn playable(
        &self,
        index: usize,
        team: Team,
        _world: &board::MyWorld,
        spoke_info: &SpokeInfo,
    ) -> Option<MoveType> {
        if team == Team::Neutral {
            return None;
        }

        let num_attack = spoke_info.get_num_attack(index);

        if num_attack[team] == 0 {
            return None;
        }

        match self.factions.get_cell_inner(index) {
            &unit::GameCell::Piece(unit::Piece {
                height: stack_height,
                team: rest,
                ..
            }) => {
                let height = stack_height.to_num();
                debug_assert!(height > 0);
                let height = height as i64;

                if num_attack[team] > height {
                    if num_attack[team] < num_attack[!team] {
                        Some(MoveType::Suicidal)
                    } else {
                        if rest == team {
                            Some(MoveType::Reinforce)
                        } else {
                            Some(MoveType::Capture)
                        }
                    }
                } else {
                    None
                }
            }
            unit::GameCell::Empty => {
                if num_attack[team] < num_attack[!team] {
                    Some(MoveType::Suicidal)
                } else {
                    Some(MoveType::Fresh)
                }
            }
        }
    }

    pub fn generate_suicidal<'b>(
        &'b self,
        world: &'b board::MyWorld,
        team: Team,
        spoke_info: &'b SpokeInfo,
    ) -> impl Iterator<Item = Coordinate> + use<'b> {
        world.land_as_vec.iter().filter_map(move |&index| {
            if let Some(f) = self.playable(index, team, world, spoke_info) {
                if f.is_suicidal() {
                    Some(Coordinate(index))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}

impl hex::HexDraw for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, radius: i8) -> Result<(), std::fmt::Error> {
        Axial::from_index(self).fmt(f, radius)
    }
}

//Signifies a normal move of a [1,6] stack.
#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
pub struct Coordinate(pub usize);

impl std::ops::Deref for Coordinate {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Default for Coordinate {
    fn default() -> Self {
        Self(Default::default())
    }
}

//TODO use
#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
pub struct GeneralMove {
    coord: Coordinate,
    typ: PieceType,
}

impl hex::HexDraw for GeneralMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, radius: i8) -> Result<(), std::fmt::Error> {
        self.coord.fmt(f, radius)
    }
}
