use std::collections::btree_map::Keys;

use super::*;

use crate::{hex::HDir, mesh::small_mesh::SmallMesh};

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

impl GameState {
    pub fn iter_end_points(
        &self,
        world: &board::MyWorld,
        unit: Axial,
    ) -> [(Axial, Option<(usize, ActiveTeam)>); 6] {
        let for_ray = |unit: Axial, dir: [i8; 3]| {
            unit.to_cube()
                .ray_from_vector(hex::Cube::from_arr(dir))
                .take_while(|k| {
                    let k = k.to_axial();
                    world.get_game_cells().is_set(k)
                })
                .map(|x| x.to_axial())
        };

        let iter_end_points = |unit: Axial| {
            hex::OFFSETS.map(|h| {
                let mut last_cell = (Axial::zero(), None);
                for k in for_ray(unit, h) {
                    last_cell.0 = k;

                    if let Some((a, b)) = self.factions.cells.get_cell(k) {
                        last_cell.1 = Some((a, b));

                        break;
                    }
                }
                last_cell
            })
        };

        iter_end_points(unit)
    }
    pub fn generate_possible_moves_movement(
        &self,
        world: &board::MyWorld,
        unit: Option<Axial>,
        team: ActiveTeam,
    ) -> SmallMesh {
        let game = self;
        let mut mesh = SmallMesh::new();

        // let for_ray = |unit: Axial, dir: [i8; 3]| {
        //     unit.to_cube()
        //         .ray_from_vector(hex::Cube::from_arr(dir))
        //         .take_while(|k| {
        //             let k = k.to_axial();
        //             world.get_game_cells().is_set(k)
        //         })
        //         .map(|x| x.to_axial())
        // };

        // let iter_end_points = |unit: Axial, team: ActiveTeam| {
        //     hex::OFFSETS.map(|h| {
        //         let mut last_cell = (Axial::zero(), None);
        //         for k in for_ray(unit, h) {
        //             last_cell.0 = k;

        //             if let Some((a, b)) = game.factions.cells.get_cell(k) {
        //                 last_cell.1 = Some((a, b));

        //                 break;
        //             }
        //         }
        //         last_cell
        //     })
        // };

        for pos in world.get_game_cells().iter_mesh() {
            let it = self.iter_end_points(world, pos);

            let mut potential_height = 0;
            let mut num_enemy = 0;
            for (_, rest) in it {
                if let Some((_, tt)) = rest {
                    if tt == team {
                        potential_height += 1;
                    } else {
                        if tt != ActiveTeam::Neutral {
                            num_enemy += 1;
                        }
                    }
                }
            }

            if potential_height == 0 {
                continue;
            }

            if potential_height < num_enemy {
                continue;
            }

            if let Some((height, rest)) = self.factions.cells.get_cell(pos) {
                if potential_height <= height {
                    continue;
                }
            }

            mesh.add(pos);
        }

        mesh
    }
}

fn for_every_cell(unit: Axial, mut func: impl FnMut(Axial, &[HDir]) -> bool) {
    for a in unit.to_cube().neighbours2() {
        let a = a.to_axial();
        let dir = unit.dir_to(&a);

        if func(a, &[dir]) {
            continue;
        }

        for b in a.to_cube().neighbours2() {
            let b = b.to_axial();
            let dir2 = a.dir_to(&b);

            if b.to_cube().dist(&unit.to_cube()) < a.to_cube().dist(&unit.to_cube()) {
                continue;
            }

            if func(b, &[dir, dir2]) {
                continue;
            }

            for c in b.to_cube().neighbours2() {
                let c = c.to_axial();
                let dir3 = b.dir_to(&c);

                if c.to_cube().dist(&unit.to_cube()) < b.to_cube().dist(&unit.to_cube()) {
                    continue;
                }

                if func(c, &[dir, dir2, dir3]) {
                    continue;
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
pub struct ActualMove {
    //pub original: Axial,
    pub moveto: Axial,
    //pub attackto: Axial,
}

impl GameState {
    pub fn for_all_moves_fast(
        &mut self,
        team: ActiveTeam,
        world: &board::MyWorld,
        mut func: impl FnMut(moves::ActualMove, &GameState),
    ) {
        //let state = self;

        for mm in self
            .generate_possible_moves_movement(world, None, team)
            .iter_mesh(Axial::zero())
        {
            // let mut mmm = move_build::MovePhase { moveto: mm };

            // let mut effect = mmm.apply(team, self, world);

            let mmo = moves::ActualMove { moveto: mm };

            func(mmo, self);

            //mmm.undo(team, &effect, self);
        }

        //let mut movs = Vec::new();
        //for i in 0..state.factions.relative(team).this_team.units.len() {
        // for pos in state.factions.relative(team).this_team.clone().iter_mesh() {
        //     let mesh = state.generate_possible_moves_movement(world, &pos, team);
        //     for mm in mesh.iter_mesh(pos) {
        //         //Temporarily move the player in the game world.
        //         //We do this so that the mesh generated for extra is accurate.
        //         let mut mmm = move_build::MovePhase {
        //             original: pos,
        //             moveto: mm,
        //         };

        //         let mut effect = mmm.apply(team, state, world);

        //         let second_mesh = state.generate_possible_moves_extra(world, &mmm, &effect, team);

        //         for sm in second_mesh.iter_mesh(mm) {
        //             assert!(!state.env.terrain.is_set(sm));

        //             let kkk = mmm.into_attack(sm);

        //             let k = kkk.apply(team, state, world, &effect);

        //             let mmo = moves::ActualMove {
        //                 original: pos,
        //                 moveto: mm,
        //                 attackto: sm,
        //             };

        //             let jjj = effect.combine(k);

        //             func(jjj.clone(), mmo, state);

        //             mmm = kkk.undo(&jjj.extra_effect, state);
        //             effect = jjj.move_effect;
        //         }

        //         //revert it back just the movement component.
        //         mmm.undo(team, &effect, state);
        //     }
        // }

        // {
        //     for a in movs.iter() {
        //         assert!(state
        //             .factions
        //             .relative(team)
        //             .this_team
        //             .units
        //             .is_set(a.original));
        //     }
        // }
        // movs
    }
}
