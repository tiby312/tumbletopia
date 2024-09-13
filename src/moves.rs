
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

#[test]
fn doop() {}

impl GameState {
    // pub fn loud_moves(&self, world: &board::MyWorld, team: ActiveTeam) -> SmallMesh {
    //     let game = self;
    //     let mut mesh = SmallMesh::new();

    //     if team == ActiveTeam::Neutral {
    //         return mesh;
    //     }

    //     for pos in world.get_game_cells().iter_mesh() {
    //         let it = self.iter_end_points(world, pos);

    //         let mut potential_height = 0;
    //         let mut num_enemy = 0;
    //         for (_, rest) in it {
    //             if let Some((_, tt)) = rest {
    //                 if tt == team {
    //                     potential_height += 1;
    //                 } else {
    //                     if tt != ActiveTeam::Neutral {
    //                         num_enemy += 1;
    //                     }
    //                 }
    //             }
    //         }

    //         if potential_height == 0 {
    //             continue;
    //         }

    //         if potential_height < num_enemy {
    //             continue;
    //         }

    //         if let Some((height, rest)) = self.factions.get_cell(pos) {
    //             if potential_height <= height {
    //                 continue;
    //             }

    //             if rest != team {
    //                 mesh.add(pos);
    //             }
    //         }

    //         //mesh.add(pos);
    //     }

    //     mesh
    // }

    pub fn generate_possible_moves_movement(
        &self,
        world: &board::MyWorld,
        unit: Option<Axial>,
        team: ActiveTeam,
    ) -> (SmallMesh, SmallMesh, SmallMesh) {
        let game = self;
        let mut mesh = SmallMesh::new();
        let mut captures = SmallMesh::new();
        let mut reinforcements = SmallMesh::new();
        if team == ActiveTeam::Neutral {
            return (mesh, captures, reinforcements);
        }

        for index in world.get_game_cells().inner.iter_ones() {
            let pos = mesh::small_mesh::inverse(index);

            let it = self.factions.iter_end_points(world, index);

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

            if let Some((height, rest)) = self.factions.get_cell_inner(index) {
                if potential_height <= height {
                    continue;
                }

                if rest != team {
                    captures.inner.set(index, true);

                    //captures.add(pos)
                } else {
                    reinforcements.inner.set(index, true);
                }
            }

            mesh.inner.set(index, true);

            //mesh.add(pos);
        }

        (mesh, captures, reinforcements)
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
    pub moveto: usize,
    //pub attackto: Axial,
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
