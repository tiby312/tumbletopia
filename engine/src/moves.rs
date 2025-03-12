use hex::HDir;

use super::*;

use crate::{board::MyWorld, mesh::small_mesh::SmallMesh};

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
            let fa = Axial::from_index(a);

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
        world: &board::MyWorld,
        team: Team,
        spoke_info: &moves::SpokeInfo,
    ) {
        //TODO also need to convert ice blacks to grass blocks to emulate visition mode???
        //TODO also replace enemy units with mountains to allow suicidal moves
        let res = self
            .tactical
            .bake_fog(&self.fog[team])
            .generate_possible_moves_movement(world, team, spoke_info);

        let fog = match team {
            Team::White => &mut self.fog[0],
            Team::Black => &mut self.fog[1],
            Team::Neutral => unreachable!(),
        };

        fog.inner &= !res.0.inner;

        let pieces = match team {
            Team::White => self.tactical.factions.piece.inner & self.tactical.factions.team.inner,
            Team::Black => self.tactical.factions.piece.inner & !self.tactical.factions.team.inner,
            Team::Neutral => unreachable!(),
        };

        fog.inner &= !pieces;

        for a in pieces.iter_ones() {
            let fa = Axial::from_index(a);

            for a in hex::HDir::all() {
                let mut pos = fa;
                loop {
                    pos = pos.advance(a);

                    if !world.get_game_cells().is_set(pos) {
                        break;
                    }

                    if !res.0.is_set(pos) {
                        let np = pos; //pos.advance(a);
                        if fog.is_set(np) {
                            fog.set_coord(np, false);
                        }

                        break;
                    }
                }
            }
        }
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Thing {
    None,
    White,
    Black,
    Neutral,
}

impl Thing {
    pub fn value(&self) -> i64 {
        match self {
            Thing::None => 0,
            Thing::White => 1,
            Thing::Black => -1,
            Thing::Neutral => 0,
        }
    }
}
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct SpokeCell {
    raw: [Thing; 6],
    pub num_attack: [i64; 2],
}

pub struct SpokeTempInfo {
    data: [(i8, Option<unit::EndPoint>); 6],
}

impl SpokeInfo {
    pub fn new(_game: &GameState) -> Self {
        SpokeInfo {
            data: [SpokeCell {
                raw: [Thing::None; 6],
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
        let index = a.moveto;
        let mut it = hex::HDir::all().map(move |dd| {
            let (dis, it) = unit::ray(Axial::from_index(index), dd, world);
            for (d, index2) in it.enumerate() {
                assert!(index != index2 as usize);
                self.set(index2 as usize, dd.rotate_180(), Some(team));
                if let Some((hh, tt)) = game.factions.get_cell_inner(index2 as usize) {
                    self.set(index, dd, Some(tt));

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
            self.set(index, dd, None);
            (dis, None)
        });

        SpokeTempInfo {
            data: std::array::from_fn(|_| it.next().unwrap()),
        }
    }

    pub fn process_move(
        &mut self,
        a: ActualMove,
        team: Team,
        world: &board::MyWorld,
        game: &GameState,
    ) -> SpokeTempInfo {
        let index = a.moveto;

        //TODO this is iterating twice.
        //instead of finding end points then backtracking
        //should instead update as you go.

        let mut it = game.factions.iter_end_points(world, index);
        let arr = std::array::from_fn(|_| it.next().unwrap());

        for (i, (dis, rest)) in arr.iter().enumerate() {
            let hexdir = HDir::from(i as u8);

            let st = if let &Some(unit::EndPoint {
                team: tt, index: _, ..
            }) = rest
            {
                self.set(index, hexdir, Some(tt));
                1
            } else {
                self.set(index, hexdir, None);
                0
            };

            let stride = board::STRIDES[hexdir as usize] as isize;

            let mut index2: isize = index as isize;

            for _ in 0..dis - 1 + st {
                index2 += stride;
                self.set(index2 as usize, hexdir.rotate_180(), Some(team));
            }
        }

        SpokeTempInfo { data: arr }
    }

    pub fn undo_move(
        &mut self,
        a: ActualMove,
        effect: move_build::MoveEffect,
        team: Team,
        world: &board::MyWorld,
        game: &GameState,
        spoke_temp: SpokeTempInfo,
    ) {
        let index = a.moveto;

        // let mut it = game.factions.iter_end_points(world, index);
        // let arr: [(i8, Option<unit::EndPoint>); 6] = std::array::from_fn(|_| it.next().unwrap());
        // let kk:&[(i8, Option<unit::EndPoint>); 6]=&spoke_temp.data;
        // assert!(kk==&arr);
        let arr = &spoke_temp.data;

        for (hexdir, (i, (dis, rest))) in HDir::all().zip(arr.iter().enumerate()) {
            let st = if let &Some(unit::EndPoint {
                team: tt, index: _, ..
            }) = rest
            {
                1
            } else {
                0
            };

            let stride = board::STRIDES[hexdir as usize] as isize;

            let mut index2: isize = index as isize;

            let oppt = if let Some((_, t2)) = effect.destroyed_unit {
                Some(t2)
            } else {
                if let (_, Some(unit::EndPoint { team: t, .. })) = arr[hexdir.rotate_180() as usize]
                {
                    Some(t)
                } else {
                    None
                }
            };

            for _ in 0..*dis - 1 + st {
                index2 += stride;
                self.set(index2 as usize, hexdir.rotate_180(), oppt);
            }
        }
    }

    fn set(&mut self, index: usize, dir: HDir, val: Option<Team>) {
        let tt = match val {
            None => Thing::None,
            Some(Team::White) => Thing::White,
            Some(Team::Black) => Thing::Black,
            Some(Team::Neutral) => Thing::Neutral,
        };

        let new_value = tt.value();
        let old_value = self.data[index].raw[dir as usize].value();

        match (old_value, new_value) {
            (-1, -1) => {}
            (-1, 0) => {
                self.data[index].num_attack[1] -= 1;
            }
            (0, -1) => {
                self.data[index].num_attack[1] += 1;
            }
            (-1, 1) => {
                self.data[index].num_attack[0] += 1;
                self.data[index].num_attack[1] -= 1;
            }
            (1, 1) => {}
            (1, 0) => {
                self.data[index].num_attack[0] -= 1;
            }
            (0, 1) => {
                self.data[index].num_attack[0] += 1;
            }
            (1, -1) => {
                self.data[index].num_attack[0] -= 1;
                self.data[index].num_attack[1] += 1;
            }
            (0, 0) => {}
            _ => unreachable!("{:?} {:?}", old_value, new_value),
        }

        //let diff=new_value-old_value;
        //self.data[index].num_attack

        self.data[index].raw[dir as usize] = tt;
    }
    pub fn get(&self, index: usize, dir: HDir) -> Option<Team> {
        match self.data[index].raw[dir as usize] {
            Thing::None => None,
            Thing::White => Some(Team::White),
            Thing::Black => Some(Team::Black),
            Thing::Neutral => Some(Team::Neutral),
        }
    }
}

pub fn update_spoke_info(spoke_info: &mut SpokeInfo, world: &board::MyWorld, game: &GameState) {
    //tddt-t--dt---t-d-d-

    //Update spoke info
    for index in world.get_game_cells().inner.iter_ones() {
        for (i, (_, rest)) in game.factions.iter_end_points(world, index).enumerate() {
            let v = if let Some(unit::EndPoint { team, .. }) = rest {
                Some(team)
            } else {
                None
            };
            spoke_info.set(index, HDir::from(i as u8), v);
            assert_eq!(v, spoke_info.get(index, HDir::from(i as u8)));
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

#[derive(Debug)]
enum LosRayItem {
    //Skip,
    End { height: u8, team: Team },
    Move,
}
impl GameState {
    fn los_ray<'b>(
        &'b self,
        index2: usize,
        dir: HDir,
        world: &'b MyWorld,
    ) -> impl Iterator<Item = (usize, LosRayItem)> + use<'b> {
        let mut blocked = false;
        unit::ray(Axial::from_index(index2), dir, world)
            .1
            .filter_map(move |index| {
                if blocked {
                    return None;
                }

                let index = index as usize;

                Some((
                    index,
                    if let Some((height, rest)) = self.factions.get_cell_inner(index) {
                        assert!(height > 0);

                        blocked = true;
                        // let height = height as i64;
                        // if num_attack[team] > height && num_attack[team] >= num_attack[!team] {
                        //     if rest == team {
                        //         LosRayItem::End(Some(MoveType::Reinforce))
                        //     } else {
                        //         LosRayItem::End(Some(MoveType::Capture))
                        //     }
                        // } else {
                        //     LosRayItem::End(None)
                        // }
                        LosRayItem::End { height, team: rest }
                    } else {
                        LosRayItem::Move
                        // if num_attack[team] < num_attack[!team] {
                        //     LosRayItem::Skip
                        // } else {
                        //     LosRayItem::Move
                        // }
                    },
                ))
            })
            .fuse()
    }

    //TODO prefer los_ray
    fn playable(
        &self,
        index: usize,
        team: Team,
        _world: &board::MyWorld,
        spoke_info: &SpokeInfo,
    ) -> Option<MoveType> {
        let num_attack = get_num_attack(spoke_info, index);

        // let num_attack = if let Some(spoke_info) = spoke_info {
        //     get_num_attack(spoke_info, index)
        // } else {
        //     let mut num_attack: [i64; 2] = [0, 0];

        //     for (_, rest) in self.factions.iter_end_points(world, index) {
        //         if let Some((_, team)) = rest {
        //             if team == Team::Neutral {
        //                 continue;
        //             }
        //             num_attack[team] += 1;
        //         }
        //     }
        //     num_attack
        // };

        if num_attack[team] == 0 {
            return None;
        }

        if num_attack[team] < num_attack[!team] {
            return None;
        }

        if let Some((height, rest)) = self.factions.get_cell_inner(index) {
            assert!(height > 0);
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
            if let Some(team2) = spoke_info.get(index, dir) {
                if team2 == !team {
                } else {
                    continue;
                }
            } else {
                continue;
            }

            //tddtuts-utusbtddcdc
            for (index2, fo) in self.los_ray(index, dir, world) {
                let num_attack = get_num_attack(spoke_info, index2);

                match fo {
                    LosRayItem::Move => {
                        if num_attack[team] >= num_attack[!team] && num_attack[team] > 0 {
                            ret.inner.set(index2 as usize, true);
                        }
                    }
                    LosRayItem::End {
                        height,
                        team: team2,
                    } => {
                        assert_eq!(team2, !team);
                        if num_attack[team] > height as i64 && num_attack[team] >= num_attack[!team]
                        {
                            ret.inner.set(index2 as usize, true);
                        }
                    }
                }
            }
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
            if let Some(team2) = spoke_info.get(index, dir) {
                if team2 == team {
                    continue;
                }
            }

            //tddtuts-utusbtddcdc
            for (index2, fo) in self.los_ray(index, dir, world) {
                let num_attack = get_num_attack(spoke_info, index2);

                match fo {
                    LosRayItem::Move => {
                        if num_attack[team] >= num_attack[!team] && num_attack[team] > 0 {
                            ret.inner.set(index2 as usize, true);
                        }
                    }
                    LosRayItem::End {
                        height,
                        team: team2,
                    } => {
                        assert!(team2 != team);
                        if num_attack[team] > height as i64 && num_attack[team] >= num_attack[!team]
                        {
                            ret.inner.set(index2 as usize, true);
                        }
                    }
                }
            }
        }
    }
    // fn moves_that_increase_los(
    //     &self,
    //     index: usize,
    //     team: Team,
    //     world: &board::MyWorld,
    //     ret: &mut SmallMesh,
    //     spoke_info: &SpokeInfo,
    // ) {
    //     'outer: for dir in HDir::all() {
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
    //                 //If we already have this LOS, then any move along this ray wont increase the LOS,
    //                 //so toss all of them.
    //                 if team2 == team {
    //                     continue 'outer;
    //                 } else {
    //                     break;
    //                 }
    //             }
    //         }
    //         //Add all the moves that we know would actually increase the LOS to this piece
    //         for c in cands {
    //             ret.inner.set(c as usize, true);
    //         }
    //     }
    // }

    pub fn generate_loud_moves(
        &self,
        world: &board::MyWorld,
        team: Team,
        spoke_info: &SpokeInfo,
    ) -> SmallMesh {
        //TODO remove
        //let (verif, _, _) = self.generate_possible_moves_movement(world, team);

        //let mut spoke_info = SpokeInfo::new();

        //update_spoke_info(&mut spoke_info, world, self);

        // 6*3 possibiilties for each spoke.
        // data structure will be 2 bitfields.
        // it would be the size 6*n, where n is number of cells.
        // los(n)=[6*n,6*n+1,6*n+2...6*n+5]

        let mut ret = SmallMesh::new();
        for index in world.get_game_cells().inner.iter_ones() {
            // let mut num_attack: [i64; 2] = [0, 0];

            // for (_, rest) in self.factions.iter_end_points(world, index) {
            //     if let Some((_, team)) = rest {
            //         if team == Team::Neutral {
            //             continue;
            //         }
            //         num_attack[team] += 1;
            //     }
            // }

            let num_attack = get_num_attack(&spoke_info, index);

            //assert_eq!(num_attack,num_attack2);

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
                            &mut ret,
                            &spoke_info,
                        );

                        // let mut c1 = SmallMesh::new();
                        // self.moves_that_increase_los(index, team, world, &mut c1, spoke_info);
                        // let mut c2 = SmallMesh::new();
                        // self.moves_that_increase_los_better(
                        //     index, team, world, &mut c2, spoke_info,
                        // );
                        // //assert_eq!(c1, c2);
                        // use std::ops::BitXor;
                        // let c3 = c1.inner.bitxor(c2.inner);
                        // let k: Vec<_> = c3
                        //     .iter_ones()
                        //     .map(|ii| {
                        //         move_build::to_letter_coord(&mesh::small_mesh::inverse(ii), world)
                        //     })
                        //     .collect();
                        // let ind =
                        //     move_build::to_letter_coord(&mesh::small_mesh::inverse(index), world);
                        // assert_eq!(
                        //     c1,
                        //     c2,
                        //     "{}::{:?}::{:?}::index={:?}",
                        //     self.into_string(world),
                        //     k,
                        //     team,
                        //     ind
                        // );
                    } else if num_attack[!team] == num_attack[team] + 1 {
                        //If the enemy has one more than us, our only option
                        //is to block (aside from reinforcing which we covered above)
                        self.moves_that_block_better(index, team, world, &mut ret, &spoke_info);

                        // let mut c1 = SmallMesh::new();
                        // self.moves_that_block(index, team, world, &mut c1, spoke_info);
                        // let mut c2 = SmallMesh::new();
                        // self.moves_that_block_better(index, team, world, &mut c2, spoke_info);

                        // //tddtuts-utusbtddcdc

                        // //-------------b-r--k------------------
                        // let k: Vec<_> = c2
                        //     .inner
                        //     .iter_ones()
                        //     .map(|ii| {
                        //         move_build::to_letter_coord(&mesh::small_mesh::inverse(ii), world)
                        //     })
                        //     .collect();
                        // let ind =
                        //     move_build::to_letter_coord(&mesh::small_mesh::inverse(index), world);
                        // assert_eq!(
                        //     c1,
                        //     c2,
                        //     "{}::{:?}::{:?}::index={:?}",
                        //     self.into_string(world),
                        //     k,
                        //     team,
                        //     ind
                        // );
                        // gloo_console::console_dbg!("passed test!");
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

        return ret;

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
    pub fn generate_possible_moves_movement(
        &self,
        world: &board::MyWorld,
        team: Team,
        spoke_info: &SpokeInfo,
    ) -> (SmallMesh, SmallMesh, SmallMesh) {
        let mut mesh = SmallMesh::new();

        // if allow_pass {
        //     mesh.inner.set(PASS_MOVE_INDEX, true);
        // }

        let mut captures = SmallMesh::new();
        let mut reinforcements = SmallMesh::new();
        if team == Team::Neutral {
            unreachable!();
            //return (mesh, captures, reinforcements);
        }

        for index in world.get_game_cells().inner.iter_ones() {
            if let Some(v) = self.playable(index, team, world, spoke_info) {
                mesh.inner.set(index, true);
                match v {
                    MoveType::Capture => captures.inner.set(index, true),
                    MoveType::Reinforce => reinforcements.inner.set(index, true),
                    MoveType::Fresh => {}
                }
            }
        }

        (mesh, captures, reinforcements)
    }
}

// fn for_every_cell(unit: Axial, mut func: impl FnMut(Axial, &[HDir]) -> bool) {
//     for a in unit.to_cube().neighbours2() {
//         let a = a.to_axial();
//         let dir = unit.dir_to(&a);

//         if func(a, &[dir]) {
//             continue;
//         }

//         for b in a.to_cube().neighbours2() {
//             let b = b.to_axial();
//             let dir2 = a.dir_to(&b);

//             if b.to_cube().dist(&unit.to_cube()) < a.to_cube().dist(&unit.to_cube()) {
//                 continue;
//             }

//             if func(b, &[dir, dir2]) {
//                 continue;
//             }

//             for c in b.to_cube().neighbours2() {
//                 let c = c.to_axial();
//                 let dir3 = b.dir_to(&c);

//                 if c.to_cube().dist(&unit.to_cube()) < b.to_cube().dist(&unit.to_cube()) {
//                     continue;
//                 }

//                 if func(c, &[dir, dir2, dir3]) {
//                     continue;
//                 }
//             }
//         }
//     }
// }

impl hex::HexDraw for ActualMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, radius: i8) -> Result<(), std::fmt::Error> {
        Axial::from_index(self.moveto).fmt(f, radius)
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
pub struct ActualMove {
    pub moveto: usize,
}
impl Default for ActualMove {
    fn default() -> Self {
        Self {
            moveto: Default::default(),
        }
    }
}

impl GameState {
    // pub fn for_all_moves_fast(
    //     &mut self,
    //     team: ActiveTeam,
    //     world: &board::MyWorld,
    //     mut func: impl FnMut(moves::ActualMove, &move_build::MoveEffect, &GameState),
    // ) {
    //     //let state = self;

    //     for mm in self
    //         .generate_possible_moves_movement(world, None, team).0
    //         .iter_mesh(Axial::zero())
    //     {
    //         let mut mmm = ActualMove { moveto: mm };

    //         let mut effect = mmm.apply(team, self, world);

    //         let mmo = moves::ActualMove { moveto: mm };

    //         func(mmo, &effect, self);

    //         mmm.undo(team, &effect, self);
    //     }
    // }
}
