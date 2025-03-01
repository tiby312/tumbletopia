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

pub enum LoudMove {
    Capture(usize),
    Reinforcement(usize),
}

impl GameState {
    pub fn generate_loud_moves<'a>(&'a self, world: &'a board::MyWorld, team: Team) -> SmallMesh {
        let mut ret = SmallMesh::new();
        for index in world.get_game_cells().inner.iter_ones() {
            let mut num_attack: [i64; 2] = [0, 0];

            for (_, rest) in self.factions.iter_end_points(world, index) {
                if let Some((_, team)) = rest {
                    num_attack[team] += 1;
                }
            }

            if let Some((height, rest)) = self.factions.get_cell_inner(index) {
                let height = height as i64;
                // if num_attack[team] <= height as i64 {
                //     continue;
                // }

                //if this is our piece
                if rest == team {
                    //if the enemy can capture it
                    if num_attack[team.not()] > height && num_attack[team.not()] >= num_attack[team]
                    {
                        //if we can reinforce, add that as a loud move
                        if num_attack[team] == num_attack[team.not()] {
                            ret.inner.set(index, true);
                        }

                        if num_attack[team.not()] == num_attack[team] + 1 {

                            //TODO add every move coming out of this cell as a loud move
                        }
                    }
                }

                if rest != team {
                    //if num_attack[team.not()]>

                    //This is us capturing an enemy
                    ret.inner.set(index, true);
                } else {
                    //This is us reinforcing a friendly
                    ret.inner.set(index, true);
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
            let it = self.factions.iter_end_points(world, index);

            let mut potential_height = 0;
            let mut num_enemy = 0;
            for (_, rest) in it {
                if let Some((_, tt)) = rest {
                    if tt == team {
                        potential_height += 1;
                    } else {
                        if tt != Team::Neutral {
                            num_enemy += 1;
                        }
                    }
                }
            }

            if potential_height == 0 {
                continue;
            }

            //if !allow_suicidal {
            if potential_height < num_enemy {
                continue;
            }
            //}

            if let Some((height, rest)) = self.factions.get_cell_inner(index) {
                if potential_height <= height {
                    continue;
                }

                if rest != team {
                    captures.inner.set(index, true);
                } else {
                    reinforcements.inner.set(index, true);
                }
            }

            mesh.inner.set(index, true);
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
