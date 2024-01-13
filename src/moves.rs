use super::*;

pub use partial_move::ActualMove;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum GameEnding {
    Win(ActiveTeam),
    Draw,
}

pub fn from_foo(input: &str) -> Result<Vec<ActualMove>, std::fmt::Error> {
    input
        .split(",")
        .filter(|a| *a != "")
        .map(|a| {
            dbg!(&a);
            let mut s = a.chars();

            match s.next().ok_or(std::fmt::Error)? {
                'N' => {
                    let s = s.as_str();
                    let mut k = s.split(":").map(|a| a.parse::<i16>());

                    let mut foo = || {
                        k.next()
                            .ok_or(std::fmt::Error)?
                            .map_err(|_| std::fmt::Error)
                    };

                    let unit = GridCoord([foo()?, foo()?]);
                    let moveto = GridCoord([foo()?, foo()?]);
                    todo!();
                    //Ok(ActualMove::NormalMove(PartialMoveSigl { unit, moveto }))
                }
                'E' => {
                    let s = s.as_str();
                    let mut k = s.split(":").map(|a| a.parse::<i16>());
                    let mut foo = || {
                        k.next()
                            .ok_or(std::fmt::Error)?
                            .map_err(|_| std::fmt::Error)
                    };
                    let unit = GridCoord([foo()?, foo()?]);
                    let moveto = GridCoord([foo()?, foo()?]);

                    let unit2 = GridCoord([foo()?, foo()?]);
                    let moveto2 = GridCoord([foo()?, foo()?]);
                    Ok(ActualMove::ExtraMove(
                        PartialMoveSigl { unit, moveto },
                        PartialMoveSigl {
                            unit: unit2,
                            moveto: moveto2,
                        },
                    ))
                }
                // 'I' => {
                //     let s = s.as_str();
                //     let mut k = s.split(":").map(|a| a.parse::<i16>());
                //     let mut foo = || {
                //         k.next()
                //             .ok_or(std::fmt::Error)?
                //             .map_err(|_| std::fmt::Error)
                //     };

                //     let unit = GridCoord([foo()?, foo()?]);
                //     let moveto = GridCoord([foo()?, foo()?]);
                //     Ok(ActualMove::Invade(InvadeSigl { unit, moveto }))
                // }
                //'S' => Ok(ActualMove::SkipTurn),
                'F' => {
                    let c = s.next().ok_or(std::fmt::Error)?;
                    Ok(ActualMove::GameEnd(match c {
                        'W' => GameEnding::Win(ActiveTeam::Cats),
                        'B' => GameEnding::Win(ActiveTeam::Dogs),
                        'D' => GameEnding::Draw,
                        _ => return Err(std::fmt::Error),
                    }))
                }
                _ => Err(std::fmt::Error),
            }
        })
        .collect()
}

pub fn to_foo(a: &[ActualMove], mut f: impl std::fmt::Write) -> std::fmt::Result {
    for a in a.iter() {
        match a {
            // ActualMove::Invade(i) => {
            //     let a = i.unit.0;
            //     let b = i.moveto.0;
            //     write!(f, "I{}:{}:{}:{},", a[0], a[1], b[0], b[1])?;
            // }
            // ActualMove::NormalMove(i) => {
            //     let a = i.unit.0;
            //     let b = i.moveto.0;
            //     write!(f, "N{}:{}:{}:{},", a[0], a[1], b[0], b[1])?;
            // }
            ActualMove::ExtraMove(i, j) => {
                let a = i.unit.0;
                let b = i.moveto.0;
                let c = j.unit.0;
                let d = j.moveto.0;
                write!(
                    f,
                    "E{}:{}:{}:{}:{}:{}:{}:{},",
                    a[0], a[1], b[0], b[1], c[0], c[1], d[0], d[1]
                )?;
            }
            ActualMove::SkipTurn => {
                write!(f, "S,")?;
            }
            ActualMove::GameEnd(g) => {
                let w = match g {
                    GameEnding::Win(ActiveTeam::Cats) => "W",
                    GameEnding::Win(ActiveTeam::Dogs) => "B",
                    GameEnding::Draw => "D",
                };

                write!(f, "F{}", w)?;
            }
        }
    }
    Ok(())
}

struct Doopa<'a, 'b> {
    data: &'a mut ace::WorkerManager<'b>,
}
impl<'a, 'b> Doopa<'a, 'b> {
    pub fn new(data: &'a mut ace::WorkerManager<'b>) -> Self {
        Doopa { data }
    }
    pub async fn wait_animation<W: UnwrapMe>(&mut self, m: W, team: ActiveTeam) -> W::Item {
        let an = m.into_command();
        let aa = self.data.wait_animation(an, team).await;
        W::unwrapme(aa.into_data())
    }
}

use crate::movement::{movement_mesh::Mesh, MovementMesh};

pub enum ExtraMove<T> {
    ExtraMove { unit: T },
    FinishMoving,
}

#[derive(Debug, Clone)]
pub struct InvadeSigl {
    pub unit: GridCoord,
    pub moveto: GridCoord,
}

#[derive(Debug, Clone)]
pub struct MovementSigl {
    pub unit: GridCoord,
    pub moveto: GridCoord,
}
#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct PartialMoveSigl {
    pub unit: GridCoord,
    pub moveto: GridCoord,
}
impl PartialMoveSigl {
    pub fn to_movement(&self) -> MovementSigl {
        MovementSigl {
            unit: self.unit,
            moveto: self.moveto,
        }
    }
}

pub use partial_move::PartialMove;
pub mod partial_move {
    use duckduckgeo::dists::grid::Grid;

    use crate::movement::Filter;

    use super::*;

    fn calculate_walls(position: GridCoord, typ: Type, land: &BitField, forest: &BitField) -> Mesh {
        let mut walls = Mesh::new();

        for a in position.to_cube().range(2) {
            let a = a.to_axial();
            //TODO this is duplicated logic in selection function???
            let cc = if typ == Type::Ship {
                land.is_set(a)
            } else {
                !land.is_set(a) && forest.is_set(a)
            };
            if cc {
                walls.add(a.sub(&position));
            }
        }

        walls
    }

    fn apply_normal_move(this_unit: &mut UnitData, target_cell: GridCoord) -> PartialMoveSigl {
        let orig = this_unit.position;
        this_unit.position = target_cell;

        let sigl = PartialMoveSigl {
            unit: orig,
            moveto: target_cell,
        };

        //let unit=game_view.this_team.find_slow_mut(&target_cell).unwrap();
        sigl
    }
    fn apply_extra_move(
        unit: GridCoord,
        typ: Type,
        target_cell: GridCoord,
        env: &mut Environment,
    ) -> PartialMoveSigl {
        let sigl = PartialMoveSigl {
            unit,
            moveto: target_cell,
        };

        if typ == Type::Ship {
            env.land.add(target_cell);
        } else if typ == Type::Foot {
            env.forest.add(target_cell);
        }
        sigl
    }

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
                game.env
                    .land
                    .iter_mesh(GridCoord([0; 2]))
                    .find(|&b| a == b)
                    .is_none()
            } else if typ == Type::Foot {
                game.env
                    .land
                    .iter_mesh(GridCoord([0; 2]))
                    .find(|&b| a == b)
                    .is_some()
                    && game
                        .env
                        .forest
                        .iter_mesh(GridCoord([0; 2]))
                        .find(|&b| a == b)
                        .is_none()
            } else {
                unreachable!();
            };

            let is_world_cell = game.world.get_game_cells().is_set(a);

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
        let cond2 = |a: GridCoord| true;

        for (_, a) in unit.to_cube().ring(1) {
            let a = a.to_axial();

            if cond(a) {
                if cond2(a) {
                    mesh.add_normal_cell(a.sub(&unit));
                }
                if !extra {
                    for (_, b) in a.to_cube().ring(1) {
                        let b = b.to_axial();
                        //TODO inefficient
                        if cond(b) {
                            if cond2(b) {
                                mesh.add_normal_cell(b.sub(&unit));
                            }
                        }
                    }
                }
            }
        }

        mesh
    }
    #[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
    pub enum ActualMove {
        ExtraMove(PartialMoveSigl, PartialMoveSigl),
        SkipTurn,
        GameEnd(GameEnding),
    }
    impl ActualMove {
        pub async fn execute_move_ani(
            self,
            state: &mut GameState,
            team_index: ActiveTeam,
            doop: &mut WorkerManager<'_>,
        ) {
            //let mut game = state.view_mut(team_index);
            //let mut game_history = MoveLog::new();

            match self {
                // moves::ActualMove::NormalMove(o) => {
                //     todo!();

                // }
                moves::ActualMove::ExtraMove(o, e) => {
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

                    // let iii = moves::partial_move::PartialMove {
                    //     selected_unit,
                    //     typ: unit.typ,
                    //     end: target_cell,
                    //     is_extra: true,
                    // };
                    iii.execute_with_animation(team_index, doop, mesh).await;
                }
                ActualMove::SkipTurn => {}
                ActualMove::GameEnd(_) => todo!(),
            }
        }

        pub fn execute_move_no_ani(self, state: &mut GameState, team_index: ActiveTeam) {
            match self {
                moves::ActualMove::ExtraMove(o, e) => {
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

                    let selected_unit = e.unit;
                    let target_cell = e.moveto;

                    let iii = moves::PartialMove {
                        this_unit: unit,
                        target: target_cell,
                        is_extra: true,
                        env: &mut state.env,
                    };

                    iii.execute(team_index);
                }
                _ => {
                    unreachable!()
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
                    apply_normal_move(
                        &mut state.factions.relative_mut(team).this_team.units[i],
                        mm,
                    );

                    let second_mesh =
                        generate_unit_possible_moves_inner(&mm, typ, &state, team, true);

                    for sm in second_mesh.iter_mesh(mm) {
                        //Don't bother applying the extra move. just generate the sigl.
                        movs.push(moves::ActualMove::ExtraMove(
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

    #[derive(Debug)]
    pub struct PartialMove<'a> {
        pub this_unit: &'a mut UnitData,
        pub env: &'a mut Environment,
        pub target: GridCoord,
        pub is_extra: bool,
    }

    // #[derive(Clone, Debug)]
    // pub struct PartialMove {
    //     pub selected_unit: GridCoord,
    //     pub typ: Type,
    //     pub end: GridCoord,
    //     pub is_extra: bool,
    // }

    impl PartialMove<'_> {
        pub fn execute(self, team: ActiveTeam) -> PartialMoveSigl {
            if !self.is_extra {
                let sigl = apply_normal_move(self.this_unit, self.target);
                sigl
            } else {
                let sigl = apply_extra_move(
                    self.this_unit.position,
                    self.this_unit.typ,
                    self.target,
                    self.env,
                );

                sigl
            }
        }
        pub async fn execute_with_animation(
            mut self,
            team: ActiveTeam,
            data: &mut ace::WorkerManager<'_>,
            mesh: MovementMesh,
        ) -> PartialMoveSigl {
            // let is_extra = self.is_extra;
            // let selected_unit = self.selected_unit;
            // let target_cell = self.end;
            // let typ = self.typ;

            if !self.is_extra {
                // let start = selected_unit;
                // let end = target_cell;
                // let this_unit = game_view
                //     .factions
                //     .relative_mut(team)
                //     .this_team
                //     .find_slow_mut(&start)
                //     .unwrap();

                let walls = calculate_walls(
                    self.this_unit.position,
                    self.this_unit.typ,
                    &mut self.env.land,
                    &mut self.env.forest,
                );

                let _ = Doopa::new(data)
                    .wait_animation(
                        Movement::new(self.this_unit.clone(), mesh, walls, self.target),
                        team,
                    )
                    .await;

                let sigl = apply_normal_move(self.this_unit, self.target);

                sigl
            } else {
                let sigl = apply_extra_move(
                    self.this_unit.position,
                    self.this_unit.typ,
                    self.target,
                    &mut self.env,
                );

                sigl
            }
        }
    }
}

trait UnwrapMe {
    type Item;

    fn direct_unwrap(self) -> Self::Item;
    fn into_command(self) -> animation::AnimationCommand;
    fn unwrapme(a: animation::AnimationCommand) -> Self::Item;
}
struct Movement {
    start: UnitData,
    mesh: MovementMesh,
    walls: Mesh,
    end: GridCoord,
}
impl Movement {
    pub fn new(start: UnitData, mesh: MovementMesh, walls: Mesh, end: GridCoord) -> Self {
        Movement {
            start,
            mesh,
            walls,
            end,
        }
    }
}
impl UnwrapMe for Movement {
    type Item = UnitData;

    fn direct_unwrap(mut self) -> Self::Item {
        // let last_dir = self
        //     .mesh
        //     .path(self.end.sub(&self.start.position))
        //     .last()
        //     .unwrap();

        //TODO is this right????
        self.start.position = self.end;
        self.start
    }
    fn into_command(self) -> animation::AnimationCommand {
        animation::AnimationCommand::Movement {
            unit: self.start,
            mesh: self.mesh,
            walls: self.walls,
            end: self.end,
        }
    }
    fn unwrapme(a: animation::AnimationCommand) -> Self::Item {
        let animation::AnimationCommand::Movement { unit, .. } = a else {
            unreachable!()
        };
        unit
    }
}

// struct Attack {
//     attacker: UnitData,
//     defender: UnitData,
// }
// impl Attack {
//     pub fn new(attacker: UnitData, defender: UnitData) -> Self {
//         Attack { attacker, defender }
//     }
// }
// impl UnwrapMe for Attack {
//     type Item = [UnitData; 2];
//     fn direct_unwrap(self) -> Self::Item {
//         [self.attacker, self.defender]
//     }
//     fn into_command(self) -> animation::AnimationCommand {
//         animation::AnimationCommand::Attack {
//             attacker: self.attacker,
//             defender: self.defender,
//         }
//     }
//     fn unwrapme(a: animation::AnimationCommand) -> Self::Item {
//         let animation::AnimationCommand::Attack { attacker, defender } = a else {
//             unreachable!()
//         };
//         [attacker, defender]
//     }
// }
