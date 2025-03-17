use hex::HDir;

use super::*;

use crate::mesh::small_mesh::SmallMesh;

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

            if let Some((val, tt)) = self.tactical.factions.get_cell_inner(a) {
                if tt == team {
                    for b in fa.to_cube().range(val.try_into().unwrap()) {
                        if !world.get_game_cells().is_set(*b) {
                            continue;
                        }

                        fog.set_coord(*b, false);
                    }
                }
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
}

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
    pub fn new(_game: &GameState) -> Self {
        SpokeInfo {
            data: [SpokeCell {
                raw: [Team::Neutral; 6],
                num_attack: [0; 2],
            }; 256],
        }
    }

    pub fn process_move_better(
        &mut self,
        a: ActualMove,
        team: Team,
        world: &board::MyWorld,
        game: &GameState,
    ) -> SpokeTempInfo {
        let index = a.0;
        debug_assert!(
            world.get_game_cells().inner[index as usize],
            "uhoh {:?}",
            world.format(&a)
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
                if let Some((hh, tt)) = game.factions.get_cell_inner(index2 as usize) {
                    self.set(index, dd, tt);

                    return (
                        d as i8 + 1,
                        Some(unit::EndPoint {
                            index: index2 as usize,
                            height: hh as i8,
                            team: tt,
                        }),
                    );
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
        a: ActualMove,
        effect: &move_build::MoveEffect,
        _team: Team,
        _world: &board::MyWorld,
        _game: &GameState,
        spoke_temp: SpokeTempInfo,
    ) {
        let index = a.0;

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
                if let (_, Some(unit::EndPoint { team: t, .. })) = arr[hexdir.rotate_180() as usize]
                {
                    t
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
}

pub fn update_spoke_info(spoke_info: &mut SpokeInfo, world: &board::MyWorld, game: &GameState) {
    //tddt-t--dt---t-d-d-

    //Update spoke info
    for index in world.get_game_cells().inner.iter_ones() {
        for (i, (_, rest)) in game.factions.iter_end_points(world, index).enumerate() {
            let v = if let Some(unit::EndPoint { team, .. }) = rest {
                team
            } else {
                Team::Neutral
            };
            spoke_info.set(index, HDir::from(i as u8), v);
            debug_assert_eq!(v, spoke_info.get(index, HDir::from(i as u8)));
        }
    }
}

pub fn get_num_attack(spoke_info: &SpokeInfo, index: usize) -> &[i64; 2] {
    //let mut num_attack: [i64; 2] = [0, 0];
    let foo = &spoke_info.data[index];
    &foo.num_attack
    // for t in foo.iter() {
    //     match t {
    //         Thing::None => {}
    //         Thing::White => num_attack[0] += 1,
    //         Thing::Black => num_attack[1] += 1,
    //         Thing::Neutral => {}
    //     }
    // }
    // num_attack
}

// #[derive(Debug)]
// enum LosRayItem {
//     //Skip,
//     End { height: u8, team: Team },
//     Move,
// }
impl GameState {
    fn playable(
        &self,
        index: usize,
        team: Team,
        _world: &board::MyWorld,
        spoke_info: &SpokeInfo,
    ) -> Option<MoveType> {
        let num_attack = get_num_attack(spoke_info, index);

        if num_attack[team] == 0 {
            return None;
        }

        if num_attack[team] < num_attack[!team] {
            return None;
        }

        if let Some((height, rest)) = self.factions.get_cell_inner(index) {
            debug_assert!(height > 0);
            let height = height as i64;
            if num_attack[team] > height {
                if rest == team {
                    Some(MoveType::Reinforce)
                } else {
                    Some(MoveType::Capture)
                }
            } else {
                None
            }
        } else {
            Some(MoveType::Fresh)
        }
    }

    // fn moves_that_block(
    //     &self,
    //     index: usize,
    //     team: Team,
    //     world: &board::MyWorld,
    //     ret: &mut SmallMesh,
    //     spoke_info: &SpokeInfo,
    // ) {
    //     for dir in HDir::all() {
    //         //TODO there is a way to skip over spokes that we know are not opponent controlled.
    //         let mut cands = vec![];
    //         for index2 in unit::ray(Axial::from_index(index), dir, world).1 {
    //             if self
    //                 .playable(index2 as usize, team, world, spoke_info)
    //                 .is_some()
    //             {
    //                 cands.push(index2);
    //             }
    //             if let Some((_, team2)) = self.factions.get_cell_inner(index2 as usize) {
    //                 assert_eq!(spoke_info.retrieve(index, dir), Some(team2));
    //                 //If we already have this LOS, then any move along this ray wont increase the LOS,
    //                 //so toss all of them.
    //                 if team2 == !team {
    //                     //Add all the moves that we know would actually increase the LOS to this piece
    //                     for c in cands {
    //                         ret.inner.set(c as usize, true);
    //                     }
    //                     break;
    //                 } else {
    //                     break;
    //                 }
    //             } else {
    //                 //assert_eq!(spoke_info.retrieve(index,dir),None);
    //             }
    //         }
    //     }
    // }

    fn moves_that_block_better(
        &self,
        index: usize,
        team: Team,
        world: &board::MyWorld,
        ret: &mut SmallMesh,
        spoke_info: &SpokeInfo,
    ) {
        for dir in HDir::all() {
            let team2 = spoke_info.get(index, dir);
            if team2 == !team {
            } else {
                continue;
            }

            //tddtuts-utusbtddcdc
            for index2 in unit::ray(Axial::from_index(&index), dir, world).1 {
                let index2 = index2 as usize;
                let num_attack = get_num_attack(spoke_info, index2);

                if let Some((height, team2)) = self.factions.get_cell_inner(index2) {
                    debug_assert_eq!(team2, !team);
                    if num_attack[team] > height as i64 && num_attack[team] >= num_attack[!team] {
                        ret.inner.set(index2 as usize, true);
                    }
                    break;
                } else {
                    if num_attack[team] >= num_attack[!team] && num_attack[team] > 0 {
                        ret.inner.set(index2 as usize, true);
                    }
                }
            }

            // for (index2, fo) in self.los_ray(index, dir, world) {
            //     let num_attack = get_num_attack(spoke_info, index2);

            //     match fo {
            //         LosRayItem::Move => {
            //             if num_attack[team] >= num_attack[!team] && num_attack[team] > 0 {
            //                 ret.inner.set(index2 as usize, true);
            //             }
            //         }
            //         LosRayItem::End {
            //             height,
            //             team: team2,
            //         } => {
            //             debug_assert_eq!(team2, !team);
            //             if num_attack[team] > height as i64 && num_attack[team] >= num_attack[!team]
            //             {
            //                 ret.inner.set(index2 as usize, true);
            //             }
            //         }
            //     }
            // }
        }
    }

    fn moves_that_increase_los_better(
        &self,
        index: usize,
        team: Team,
        world: &board::MyWorld,
        ret: &mut SmallMesh,
        spoke_info: &SpokeInfo,
    ) {
        for dir in HDir::all() {
            let team2 = spoke_info.get(index, dir);
            if team2 == team {
                continue;
            }

            for index2 in unit::ray(Axial::from_index(&index), dir, world).1 {
                let index2 = index2 as usize;
                let num_attack = get_num_attack(spoke_info, index2);

                if let Some((height, team2)) = self.factions.get_cell_inner(index2) {
                    debug_assert!(team2 != team);

                    if num_attack[team] > height as i64 && num_attack[team] >= num_attack[!team] {
                        ret.inner.set(index2 as usize, true);
                    }
                    break;
                } else {
                    if num_attack[team] >= num_attack[!team] && num_attack[team] > 0 {
                        ret.inner.set(index2 as usize, true);
                    }
                }
            }
        }
    }

    pub fn generate_loud_moves(
        &self,
        world: &board::MyWorld,
        team: Team,
        spoke_info: &SpokeInfo,
    ) -> (SmallMesh, SmallMesh) {
        let mut ret = SmallMesh::new();
        let mut ret2 = SmallMesh::new();

        for &index in world.land_as_vec.iter() {
            let num_attack = get_num_attack(&spoke_info, index);

            if let Some((height, rest)) = self.factions.get_cell_inner(index) {
                let height = height as i64;

                //if this is our piece
                if rest == team {
                    //if we can reinforce, add that as a loud move
                    if num_attack[team] > height && num_attack[team] == num_attack[!team] {
                        ret.inner.set(index, true);
                    }

                    //if the enemy can capture it
                    if num_attack[!team] <= height {
                        continue;
                    }

                    if num_attack[!team] < num_attack[team] {
                        continue;
                    }

                    //If enemy is threatening to take and we have parity in LOS,
                    //if we increase our LOS, then we would be able to recapture this cell.
                    if num_attack[!team] == num_attack[team] {
                        //add every move coming out of this cell as a loud move
                        //that would increase the los of the cell being threatened.
                        self.moves_that_increase_los_better(
                            index,
                            team,
                            world,
                            &mut ret2,
                            &spoke_info,
                        );
                    } else if num_attack[!team] == num_attack[team] + 1 {
                        //If the enemy has one more than us, our only option
                        //is to block (aside from reinforcing which we covered above)
                        self.moves_that_block_better(index, team, world, &mut ret2, &spoke_info);
                    }
                } else {
                    //If it is an enemy piece, then
                    if num_attack[team] > height && num_attack[team] >= num_attack[!team] {
                        ret.inner.set(index, true);
                    }

                    // if this is an enemy piece that is in contention
                    // any move that adds a LOS on this piece is a loud move.
                    if (num_attack[team] == num_attack[!team]
                        || num_attack[team] + 1 == num_attack[!team])
                        && num_attack[team] >= height
                    {
                        self.moves_that_increase_los_better(
                            index,
                            team,
                            world,
                            &mut ret,
                            &spoke_info,
                        );
                    }
                }
            }
        }

        return (ret, ret2);

        //Add moves that are this team capture opponents.

        //For all opponent moves that would result in a capture of our pieces
        //    If we can reinforce this move, add that
        //
        //    if the opponent already has a massive advantage of LOS on the move, continue
        //    so now we know the opponent only has a LOS that is one more than the height of this cell
        //
        //    add all the moves coming out of that move
        //
        //

        //TODO DO THISSSSS
    }
    pub fn generate_possible_moves_movement<'b>(
        &'b self,
        world: &'b board::MyWorld,
        team: Team,
        spoke_info: &'b SpokeInfo,
    ) -> impl Iterator<Item = ActualMove> + use<'b> {
        if team == Team::Neutral {
            unreachable!();
        }

        world.land_as_vec.iter().filter_map(move |&index| {
            if let Some(_) = self.playable(index, team, world, spoke_info) {
                //mesh.inner.set(index, true);
                // match v {
                //     MoveType::Capture => captures.inner.set(index, true),
                //     MoveType::Reinforce => reinforcements.inner.set(index, true),
                //     MoveType::Fresh => {}
                // }
                Some(ActualMove(index))
            } else {
                None
            }
        })
    }
}

impl hex::HexDraw for ActualMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, radius: i8) -> Result<(), std::fmt::Error> {
        Axial::from_index(self).fmt(f, radius)
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
pub struct ActualMove(pub usize);

impl std::ops::Deref for ActualMove {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Default for ActualMove {
    fn default() -> Self {
        Self(Default::default())
    }
}
