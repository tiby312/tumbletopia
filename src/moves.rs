use super::*;

pub use partial_move::ActualMove;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum GameEnding {
    Win(ActiveTeam),
    Draw,
}

// pub fn from_foo(input: &str) -> Result<Vec<ActualMove>, std::fmt::Error> {
//     input
//         .split(",")
//         .filter(|a| *a != "")
//         .map(|a| {
//             dbg!(&a);
//             let mut s = a.chars();

//             match s.next().ok_or(std::fmt::Error)? {
//                 'N' => {
//                     let s = s.as_str();
//                     let mut k = s.split(":").map(|a| a.parse::<i16>());

//                     let mut foo = || {
//                         k.next()
//                             .ok_or(std::fmt::Error)?
//                             .map_err(|_| std::fmt::Error)
//                     };

//                     let unit = GridCoord([foo()?, foo()?]);
//                     let moveto = GridCoord([foo()?, foo()?]);
//                     todo!();
//                     //Ok(ActualMove::NormalMove(PartialMoveSigl { unit, moveto }))
//                 }
//                 'E' => {
//                     let s = s.as_str();
//                     let mut k = s.split(":").map(|a| a.parse::<i16>());
//                     let mut foo = || {
//                         k.next()
//                             .ok_or(std::fmt::Error)?
//                             .map_err(|_| std::fmt::Error)
//                     };
//                     let unit = GridCoord([foo()?, foo()?]);
//                     let moveto = GridCoord([foo()?, foo()?]);

//                     let unit2 = GridCoord([foo()?, foo()?]);
//                     let moveto2 = GridCoord([foo()?, foo()?]);
//                     Ok(ActualMove::ExtraMove(
//                         PartialMoveSigl { unit, moveto },
//                         PartialMoveSigl {
//                             unit: unit2,
//                             moveto: moveto2,
//                         },
//                     ))
//                 }
//                 // 'I' => {
//                 //     let s = s.as_str();
//                 //     let mut k = s.split(":").map(|a| a.parse::<i16>());
//                 //     let mut foo = || {
//                 //         k.next()
//                 //             .ok_or(std::fmt::Error)?
//                 //             .map_err(|_| std::fmt::Error)
//                 //     };

//                 //     let unit = GridCoord([foo()?, foo()?]);
//                 //     let moveto = GridCoord([foo()?, foo()?]);
//                 //     Ok(ActualMove::Invade(InvadeSigl { unit, moveto }))
//                 // }
//                 //'S' => Ok(ActualMove::SkipTurn),
//                 'F' => {
//                     let c = s.next().ok_or(std::fmt::Error)?;
//                     Ok(ActualMove::GameEnd(match c {
//                         'W' => GameEnding::Win(ActiveTeam::Cats),
//                         'B' => GameEnding::Win(ActiveTeam::Dogs),
//                         'D' => GameEnding::Draw,
//                         _ => return Err(std::fmt::Error),
//                     }))
//                 }
//                 _ => Err(std::fmt::Error),
//             }
//         })
//         .collect()
// }

// pub fn to_foo(a: &[ActualMove], mut f: impl std::fmt::Write) -> std::fmt::Result {
//     for a in a.iter() {
//         match a {
//             // ActualMove::Invade(i) => {
//             //     let a = i.unit.0;
//             //     let b = i.moveto.0;
//             //     write!(f, "I{}:{}:{}:{},", a[0], a[1], b[0], b[1])?;
//             // }
//             // ActualMove::NormalMove(i) => {
//             //     let a = i.unit.0;
//             //     let b = i.moveto.0;
//             //     write!(f, "N{}:{}:{}:{},", a[0], a[1], b[0], b[1])?;
//             // }
//             ActualMove::ExtraMove(i, j) => {
//                 let a = i.unit.0;
//                 let b = i.moveto.0;
//                 let c = j.unit.0;
//                 let d = j.moveto.0;
//                 write!(
//                     f,
//                     "E{}:{}:{}:{}:{}:{}:{}:{},",
//                     a[0], a[1], b[0], b[1], c[0], c[1], d[0], d[1]
//                 )?;
//             }
//             ActualMove::SkipTurn => {
//                 write!(f, "S,")?;
//             }
//             ActualMove::GameEnd(g) => {
//                 let w = match g {
//                     GameEnding::Win(ActiveTeam::Cats) => "W",
//                     GameEnding::Win(ActiveTeam::Dogs) => "B",
//                     GameEnding::Draw => "D",
//                 };

//                 write!(f, "F{}", w)?;
//             }
//         }
//     }
//     Ok(())
// }

// struct Doopa<'a, 'b> {
//     data: &'a mut ace::WorkerManager<'b>,
// }
// impl<'a, 'b> Doopa<'a, 'b> {
//     pub fn new(data: &'a mut ace::WorkerManager<'b>) -> Self {
//         Doopa { data }
//     }
//     pub async fn wait_animation<W: UnwrapMe>(&mut self, m: W, team: ActiveTeam) -> W::Item {
//         let an = m.into_command();
//         let aa = self.data.wait_animation(an, team).await;
//         W::unwrapme(aa.into_data())
//     }
// }

use crate::movement::{movement_mesh::Mesh, MovementMesh};

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct PartialMoveSigl {
    pub unit: GridCoord,
    pub moveto: GridCoord,
}

pub use partial_move::PartialMove;
pub mod partial_move {

    use super::*;

    pub fn generate_unit_possible_moves_inner(
        unit: &GridCoord,
        typ: Type,
        game: &GameState,
        team: ActiveTeam,
        extra: bool,
    ) -> movement::MovementMesh {
        let unit = *unit;
        let mut mesh = movement::MovementMesh::new();

        let cond = |a: GridCoord| {
            let cc = if typ == Type::Ship {
                !game.env.land.is_coord_set(a)
            } else if typ == Type::Foot {
                game.env.land.is_coord_set(a) && !game.env.forest.is_coord_set(a)
            } else {
                unreachable!();
            };

            let is_world_cell = game.world.get_game_cells().is_coord_set(a);

            a != unit
                && is_world_cell
                && cc
                && game
                    .factions
                    .relative(team)
                    .this_team
                    .find_slow(&a)
                    .is_none()
                && game
                    .factions
                    .relative(team)
                    .that_team
                    .find_slow(&a)
                    .is_none()
        };

        for (_, a) in unit.to_cube().ring(1) {
            let a = a.to_axial();

            if cond(a) {
                mesh.add_normal_cell(a.sub(&unit));

                if !extra {
                    for (_, b) in a.to_cube().ring(1) {
                        let b = b.to_axial();
                        if cond(b) {
                            mesh.add_normal_cell(b.sub(&unit));
                        }
                    }
                }
            }
        }

        mesh
    }
    #[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
    pub enum ActualMove {
        CombinedMove(PartialMoveSigl, PartialMoveSigl),
    }
    impl ActualMove {
        pub async fn execute_move_ani(
            self,
            state: &mut GameState,
            team_index: ActiveTeam,
            doop: &mut WorkerManager<'_>,
        ) {
            match self {
                moves::ActualMove::CombinedMove(o, e) => {
                    let target_cell = o.moveto;
                    let unit = state
                        .factions
                        .relative(team_index)
                        .this_team
                        .find_slow(&o.unit)
                        .unwrap();
                    let typ = unit.typ;
                    let mesh = generate_unit_possible_moves_inner(
                        &unit.position,
                        unit.typ,
                        &state,
                        team_index,
                        false,
                    );

                    let unit = state
                        .factions
                        .relative_mut(team_index)
                        .this_team
                        .find_slow_mut(&o.unit)
                        .unwrap();

                    let iii = moves::PartialMove {
                        this_unit: unit,
                        target: target_cell,
                        is_extra: false,
                        env: &mut state.env,
                    };

                    let iii = iii.execute_with_animation(team_index, doop, mesh).await;

                    assert_eq!(iii.moveto, e.unit);

                    let selected_unit = e.unit;
                    let target_cell = e.moveto;

                    let mesh = generate_unit_possible_moves_inner(
                        &selected_unit,
                        typ,
                        state,
                        team_index,
                        true,
                    );

                    let unit = state
                        .factions
                        .relative_mut(team_index)
                        .this_team
                        .find_slow_mut(&e.unit)
                        .unwrap();
                    let iii = moves::PartialMove {
                        this_unit: unit,
                        target: target_cell,
                        is_extra: true,
                        env: &mut state.env,
                    };
                    iii.execute_with_animation(team_index, doop, mesh).await;
                }
            }
        }

        pub fn execute_move_no_ani(self, state: &mut GameState, team_index: ActiveTeam) {
            match self {
                moves::ActualMove::CombinedMove(o, e) => {
                    let target_cell = o.moveto;
                    let unit = state
                        .factions
                        .relative_mut(team_index)
                        .this_team
                        .find_slow_mut(&o.unit)
                        .unwrap();

                    let iii = moves::PartialMove {
                        this_unit: unit,
                        target: target_cell,
                        is_extra: false,
                        env: &mut state.env,
                    };

                    let iii = iii.execute(team_index);

                    assert_eq!(iii.moveto, e.unit);

                    let target_cell = e.moveto;

                    let iii = moves::PartialMove {
                        this_unit: unit,
                        target: target_cell,
                        is_extra: true,
                        env: &mut state.env,
                    };

                    iii.execute(team_index);
                }
            }
        }
        pub fn execute_undo(self, state: &mut GameState, team_index: ActiveTeam) {
            match self {
                moves::ActualMove::CombinedMove(o, e) => {
                    let k = state
                        .factions
                        .relative_mut(team_index)
                        .this_team
                        .find_slow_mut(&o.moveto)
                        .unwrap();

                    match k.typ {
                        Type::Ship => {
                            assert!(state.env.land.is_coord_set(e.moveto));
                            state.env.land.set_coord(e.moveto, false);
                        }
                        Type::Foot => {
                            assert!(state.env.forest.is_coord_set(e.moveto));
                            state.env.forest.set_coord(e.moveto, false);
                        }
                        _ => {
                            unreachable!()
                        }
                    }

                    k.position = o.unit;
                }
            }
        }
    }

    impl GameState {
        pub fn for_all_moves_fast(&mut self, team: ActiveTeam) -> Vec<moves::ActualMove> {
            let state = self;
            let mut movs = Vec::new();
            for i in 0..state.factions.relative(team).this_team.units.len() {
                let pos = state.factions.relative_mut(team).this_team.units[i].position;
                let typ = state.factions.relative_mut(team).this_team.units[i].typ;

                let mesh = generate_unit_possible_moves_inner(&pos, typ, &state, team, false);
                for mm in mesh.iter_mesh(pos) {
                    //Temporarily move the player in the game world.
                    //We do this so that the mesh generated for extra is accurate.
                    let ii = PartialMove {
                        this_unit: &mut state.factions.relative_mut(team).this_team.units[i],
                        env: &mut state.env,
                        target: mm,
                        is_extra: false,
                    };
                    ii.execute(team);

                    let second_mesh =
                        generate_unit_possible_moves_inner(&mm, typ, &state, team, true);

                    for sm in second_mesh.iter_mesh(mm) {
                        //Don't bother applying the extra move. just generate the sigl.
                        movs.push(moves::ActualMove::CombinedMove(
                            moves::PartialMoveSigl {
                                unit: pos,
                                moveto: mm,
                            },
                            moves::PartialMoveSigl {
                                unit: mm,
                                moveto: sm,
                            },
                        ))
                    }

                    //revert it back.
                    state.factions.relative_mut(team).this_team.units[i].position = pos;
                }
            }
            movs
        }
    }

    use crate::ace::WorkerManager;

    pub use partial::PartialMove;
    pub mod partial {
        use super::*;
        #[derive(Debug)]
        pub struct PartialMove<'a> {
            pub this_unit: &'a mut UnitData,
            pub env: &'a mut Environment,
            pub target: GridCoord,
            pub is_extra: bool,
        }

        fn apply_normal_move(this_unit: &mut UnitData, target_cell: GridCoord) -> PartialMoveSigl {
            let orig = this_unit.position;
            this_unit.position = target_cell;

            PartialMoveSigl {
                unit: orig,
                moveto: target_cell,
            }
        }
        fn apply_extra_move(
            unit: GridCoord,
            typ: Type,
            target_cell: GridCoord,
            env: &mut Environment,
        ) -> PartialMoveSigl {
            if typ == Type::Ship {
                env.land.set_coord(target_cell, true);
            } else if typ == Type::Foot {
                env.forest.set_coord(target_cell, true);
            }

            PartialMoveSigl {
                unit,
                moveto: target_cell,
            }
        }
        impl PartialMove<'_> {
            pub fn execute(self, _team: ActiveTeam) -> PartialMoveSigl {
                if !self.is_extra {
                    apply_normal_move(self.this_unit, self.target)
                } else {
                    apply_extra_move(
                        self.this_unit.position,
                        self.this_unit.typ,
                        self.target,
                        self.env,
                    )
                }
            }
            pub async fn execute_with_animation(
                mut self,
                team: ActiveTeam,
                data: &mut ace::WorkerManager<'_>,
                mesh: MovementMesh,
            ) -> PartialMoveSigl {
                fn calculate_walls(position: GridCoord, typ: Type, env: &Environment) -> Mesh {
                    let mut walls = Mesh::new();

                    for a in position.to_cube().range(2) {
                        let a = a.to_axial();
                        //TODO this is duplicated logic in selection function???
                        let cc = if typ == Type::Ship {
                            env.land.is_coord_set(a)
                        } else {
                            !env.land.is_coord_set(a) && env.forest.is_coord_set(a)
                        };
                        if cc {
                            walls.add(a.sub(&position));
                        }
                    }

                    walls
                }
                if !self.is_extra {
                    let walls =
                        calculate_walls(self.this_unit.position, self.this_unit.typ, &mut self.env);

                    let _ = data
                        .wait_animation(
                            animation::AnimationCommand::Movement {
                                unit: self.this_unit.clone(),
                                mesh,
                                walls,
                                end: self.target,
                            },
                            team,
                        )
                        .await;

                    apply_normal_move(self.this_unit, self.target)
                } else {
                    apply_extra_move(
                        self.this_unit.position,
                        self.this_unit.typ,
                        self.target,
                        &mut self.env,
                    )
                }
            }
        }
    }
}
