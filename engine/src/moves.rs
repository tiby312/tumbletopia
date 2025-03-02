use bitvec::array::BitArray;
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

pub const PASS_MOVE: Axial = Axial { q: -5, r: 9 };
pub const PASS_MOVE_INDEX: usize = const { mesh::small_mesh::conv(PASS_MOVE) };

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
            let fa = mesh::small_mesh::inverse(a);

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

    pub fn update_fog_spokes(&mut self, world: &board::MyWorld, team: Team) {
        //TODO also need to convert ice blacks to grass blocks to emulate visition mode???
        //TODO also replace enemy units with mountains to allow suicidal moves
        let res = self
            .tactical
            .bake_fog(&self.fog[team])
            .generate_possible_moves_movement(world, team);

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
            let fa = mesh::small_mesh::inverse(a);

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
impl GameState {
    fn playable(&self, index: usize, team: Team, world: &board::MyWorld) -> Option<MoveType> {
        let mut num_attack: [i64; 2] = [0, 0];

        for (_, rest) in self.factions.iter_end_points(world, index) {
            if let Some((_, team)) = rest {
                if team == Team::Neutral {
                    continue;
                }
                num_attack[team] += 1;
            }
        }

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

    fn moves_that_increase_los_by_one(
        &self,
        index: usize,
        team: Team,
        world: &board::MyWorld,
        ret: &mut SmallMesh,
    ) {
        'outer: for dir in HDir::all() {
            let mut cands = vec![];
            for index2 in unit::ray(mesh::small_mesh::inverse(index), dir, world).1 {
                if self.playable(index2 as usize, team, world).is_some() {
                    cands.push(index2);
                }
                if let Some((_, team2)) = self.factions.get_cell_inner(index2 as usize) {
                    //If we already have this LOS, then any move along this ray wont increase the LOS,
                    //so toss all of them.
                    if team2 == team {
                        continue 'outer;
                    } else {
                        break;
                    }
                }
            }
            //Add all the moves that we know would actually increase the LOS to this piece
            for c in cands {
                ret.inner.set(c as usize, true);
            }
        }
    }

    pub fn generate_loud_moves(&self, world: &board::MyWorld, team: Team) -> SmallMesh {
        let (verif, _, _) = self.generate_possible_moves_movement(world, team);

        let mut ret = SmallMesh::new();
        for index in world.get_game_cells().inner.iter_ones() {
            let mut num_attack: [i64; 2] = [0, 0];

            for (_, rest) in self.factions.iter_end_points(world, index) {
                if let Some((_, team)) = rest {
                    if team == Team::Neutral {
                        continue;
                    }
                    num_attack[team] += 1;
                }
            }

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

                    //If there is one more enemy LOS on this piece
                    if num_attack[!team] == num_attack[team] + 1 {
                        //add every move coming out of this cell as a loud move
                        //that would increase the los of the cell being threatened.
                        self.moves_that_increase_los_by_one(index, team, world, &mut ret);
                    }
                } else {
                    //If it is an enemy piece, then
                    if num_attack[team] > height && num_attack[team] >= num_attack[!team] {
                        ret.inner.set(index, true);
                    }

                    // if this is an enemy piece that is in contention
                    // any move that adds a LOS on this piece is a loud move.
                    if num_attack[team] == num_attack[!team] && num_attack[team] >= height {
                        self.moves_that_increase_los_by_one(index, team, world, &mut ret);
                    }
                }
            }
        }

        // for a in ret.inner.iter_ones() {
        //     if !verif.inner[a]{
        //         let res = move_build::to_letter_coord(&mesh::small_mesh::inverse(a), world);
        //         let k=format!("{}{}", res.0, res.1);
        //         gloo_console::console_dbg!(k);
        //         panic!("FAAAIL");
        //     }
        // }
        assert_eq!(((!verif.inner) & ret.inner).count_ones(), 0);

        //gloo_console::console_dbg!("num loud moves",ret.inner.count_ones());
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
            if let Some(v) = self.playable(index, team, world) {
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
