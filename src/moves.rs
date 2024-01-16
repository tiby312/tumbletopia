use super::*;

use crate::movement::{movement_mesh::Mesh, MovementMesh};

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct PartialMoveSigl {
    pub unit: GridCoord,
    pub moveto: GridCoord,
}

impl GameState {
    pub fn generate_unit_possible_moves_inner(
        &self,
        unit: &GridCoord,
        team: ActiveTeam,
        extra: Option<PartialMoveSigl>,
    ) -> movement::MovementMesh {
        let game = self;
        let unit = *unit;
        let mut mesh = movement::MovementMesh::new();

        let is_ship = if let Some(e) = extra {
            !game.env.land.is_coord_set(e.unit)
        } else {
            !game.env.land.is_coord_set(unit)
        };

        let cond = |a: GridCoord, extra: Option<PartialMoveSigl>, depth: usize| {
            let cc = if is_ship {
                !game.env.land.is_coord_set(a)
            } else {
                game.env.land.is_coord_set(a) && !game.env.forest.is_coord_set(a)
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

        if let Some(extra) = extra {
            let transition_to_land = {
                !game.env.land.is_coord_set(extra.unit) && game.env.land.is_coord_set(extra.moveto)
            };

            if transition_to_land {
                mesh.add_normal_cell(extra.unit.sub(&unit));
            } else {
                for (_, a) in unit.to_cube().ring(1) {
                    let a = a.to_axial();
                    if cond(a, Some(extra), 0) {
                        mesh.add_normal_cell(a.sub(&unit));
                    }
                }
            }
        } else {
            for (_, a) in unit.to_cube().ring(1) {
                let a = a.to_axial();

                if cond(a, None, 0) {
                    mesh.add_normal_cell(a.sub(&unit));

                    for (_, b) in a.to_cube().ring(1) {
                        let b = b.to_axial();
                        if cond(b, None, 1) {
                            mesh.add_normal_cell(b.sub(&unit));
                        }
                    }
                } else {
                    if game.env.land.is_coord_set(a)
                        && !game.env.forest.is_coord_set(a)
                        && is_ship
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
                    {
                        mesh.add_normal_cell(a.sub(&unit));
                    }
                }
            }
        }

        mesh
    }
}
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum ActualMove {
    CombinedMove(PartialMoveSigl, PartialMoveSigl),
}
impl ActualMove {
    pub async fn execute_move_ani(
        &self,
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
                let mesh =
                    state.generate_unit_possible_moves_inner(&unit.position, team_index, None);

                let unit = state
                    .factions
                    .relative_mut(team_index)
                    .this_team
                    .find_slow_mut(&o.unit)
                    .unwrap();

                let iii = moves::PartialMove {
                    this_unit: unit,
                    target: target_cell,
                    is_extra: None,
                    env: &mut state.env,
                };

                let iii = iii.execute_with_animation(team_index, doop, mesh).await;

                assert_eq!(iii.moveto, e.unit);

                let selected_unit = e.unit;
                let target_cell = e.moveto;

                let mesh =
                    state.generate_unit_possible_moves_inner(&selected_unit, team_index, Some(iii));

                let unit = state
                    .factions
                    .relative_mut(team_index)
                    .this_team
                    .find_slow_mut(&e.unit)
                    .unwrap();
                let iii = moves::PartialMove {
                    this_unit: unit,
                    target: target_cell,
                    is_extra: Some(iii),
                    env: &mut state.env,
                };
                iii.execute_with_animation(team_index, doop, mesh).await;
            }
        }
    }

    pub fn execute_move_no_ani(&self, state: &mut GameState, team_index: ActiveTeam) {
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
                    is_extra: None,
                    env: &mut state.env,
                };

                let iii = iii.execute(team_index);

                assert_eq!(iii.moveto, e.unit);

                let target_cell = e.moveto;

                let iii = moves::PartialMove {
                    this_unit: unit,
                    target: target_cell,
                    is_extra: Some(iii),
                    env: &mut state.env,
                };

                iii.execute(team_index);
            }
        }
    }
    pub fn execute_undo(&self, state: &mut GameState, team_index: ActiveTeam) {
        match self {
            moves::ActualMove::CombinedMove(o, e) => {
                let k = state
                    .factions
                    .relative_mut(team_index)
                    .this_team
                    .find_slow_mut(&o.moveto)
                    .unwrap();

                let la = state.env.land.is_coord_set(e.moveto);
                let fr = state.env.forest.is_coord_set(e.moveto);
                let is_ship = match (la, fr) {
                    (true, false) => true,
                    (true, true) => false,
                    (false, true) => unreachable!(),
                    (false, false) => unreachable!(),
                };

                // let is_ship=if
                // if e.moveto==o.unit{
                //     state.env.land.is_coord_set(o.moveto)
                // }else{
                //     !state.env.land.is_coord_set(o.unit)
                // };

                // let is_ship = !state.env.land.is_coord_set(o.unit)
                //     || (state.env.land.is_coord_set(o.unit) && e.moveto == o.unit);

                //let is_ship = !state.env.land.is_coord_set(k.position);

                if is_ship {
                    assert!(state.env.land.is_coord_set(e.moveto));
                    state.env.land.set_coord(e.moveto, false);
                } else {
                    assert!(state.env.forest.is_coord_set(e.moveto));
                    state.env.forest.set_coord(e.moveto, false);
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

            let mesh = state.generate_unit_possible_moves_inner(&pos, team, None);
            for mm in mesh.iter_mesh(pos) {
                //Temporarily move the player in the game world.
                //We do this so that the mesh generated for extra is accurate.
                let ii = PartialMove {
                    this_unit: &mut state.factions.relative_mut(team).this_team.units[i],
                    env: &mut state.env,
                    target: mm,
                    is_extra: None,
                };
                let il = ii.execute(team);

                let second_mesh = state.generate_unit_possible_moves_inner(&mm, team, Some(il));

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
        pub is_extra: Option<PartialMoveSigl>,
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
        target_cell: GridCoord,
        original: GridCoord,
        env: &mut Environment,
    ) -> PartialMoveSigl {
        let is_ship = !env.land.is_coord_set(original);

        if is_ship {
            env.land.set_coord(target_cell, true);
        } else {
            env.forest.set_coord(target_cell, true);
        }

        PartialMoveSigl {
            unit,
            moveto: target_cell,
        }
    }

    impl PartialMove<'_> {
        pub fn execute(self, _team: ActiveTeam) -> PartialMoveSigl {
            if let Some(extra) = self.is_extra {
                apply_extra_move(self.this_unit.position, self.target, extra.unit, self.env)
            } else {
                apply_normal_move(self.this_unit, self.target)
            }
        }
        pub async fn execute_with_animation(
            mut self,
            team: ActiveTeam,
            data: &mut ace::WorkerManager<'_>,
            mesh: MovementMesh,
        ) -> PartialMoveSigl {
            fn calculate_walls(position: GridCoord, env: &Environment) -> Mesh {
                let mut walls = Mesh::new();

                let is_ship = !env.land.is_coord_set(position);

                for a in position.to_cube().range(2) {
                    let a = a.to_axial();
                    //TODO this is duplicated logic in selection function???
                    let cc = if is_ship {
                        env.land.is_coord_set(a)
                    } else {
                        !env.land.is_coord_set(a) || env.forest.is_coord_set(a)
                    };
                    if cc {
                        walls.add(a.sub(&position));
                    }
                }

                walls
            }
            if let Some(extra) = self.is_extra {
                apply_extra_move(
                    self.this_unit.position,
                    self.target,
                    extra.unit,
                    &mut self.env,
                )
            } else {
                let walls = calculate_walls(self.this_unit.position, &self.env);

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
            }
        }
    }
}
