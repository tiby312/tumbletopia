use super::*;

use crate::hex::HDir;
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
        typ: Type,
        team: ActiveTeam,
        last_move: Option<PartialMoveSigl>,
    ) -> movement::MovementMesh {
        let game = self;
        let unit = *unit;
        let mut mesh = movement::MovementMesh::new();

        let is_ship = if let Some(e) = last_move {
            !game.env.land.is_coord_set(e.unit)
        } else {
            !game.env.land.is_coord_set(unit)
        };

        let cond = |a: GridCoord, _extra: Option<PartialMoveSigl>, _depth: usize| {
            let is_world_cell = game.world.get_game_cells().is_coord_set(a);

            a != unit
                && is_world_cell
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

        if let Some(last_move) = last_move {
            let transition_to_land = {
                !game.env.land.is_coord_set(last_move.unit)
                    && game.env.land.is_coord_set(last_move.moveto)
            };

            if transition_to_land {
                mesh.add_normal_cell(last_move.unit.sub(&unit));
            } else {
                //if !is_ship {
                // for d in HDir::all() {
                //     for (l1, l2) in unit
                //         .to_cube()
                //         .ray(d)
                //         .map(|(x, y)| (x.to_axial(), y.to_axial()))
                //         .take(3)
                //     {
                //         if !game.world.get_game_cells().is_coord_set(l1) {
                //             break;
                //         }
                //         if !game.world.get_game_cells().is_coord_set(l2) {
                //             if l1 != unit {
                //                 mesh.add_normal_cell(l1.sub(&unit));
                //             }
                //             break;
                //         }

                //         if game
                //             .factions
                //             .relative(team)
                //             .this_team
                //             .find_slow(&l2)
                //             .is_some()
                //             || game
                //                 .factions
                //                 .relative(team)
                //                 .that_team
                //                 .find_slow(&l2)
                //                 .is_some()
                //         {
                //             if l1 != unit {
                //                 mesh.add_normal_cell(l1.sub(&unit));
                //             }
                //             break;
                //         }

                //         if game.env.land.is_coord_set(l2) && !game.env.forest.is_coord_set(l2) {
                //             continue;
                //         }

                //         if game.env.forest.is_coord_set(l2) {
                //             if l1 != unit {
                //                 mesh.add_normal_cell(l1.sub(&unit));
                //             }
                //             break;
                //         }

                //         if !game.env.land.is_coord_set(l2) {
                //             mesh.add_normal_cell(l2.sub(&unit));
                //             break;
                //         }
                //     }
                // }
                //mesh.remove_normal_cell(GridCoord::zero())
                //} else {
                for a in unit.to_cube().ring(1) {
                    let a = a.to_axial();

                    let j = if is_ship {
                        // !game.env.land.is_coord_set(a)
                        //true
                        !game.env.forest.is_coord_set(a)
                    } else {
                        /*has_adjacent_water(game, a) &&*/
                        !game.env.forest.is_coord_set(a)
                    };
                    if j && cond(a, Some(last_move), 0) {
                        mesh.add_normal_cell(a.sub(&unit));
                    }
                }
                //}
            }
        } else {
            let check_is_ship = |kk| {
                if is_ship {
                    !game.env.land.is_coord_set(kk)
                } else {
                    let k = game.env.land.is_coord_set(kk);

                    has_adjacent_water(game, kk) && k && !game.env.forest.is_coord_set(kk)
                }
            };

            if !is_ship && !has_adjacent_water(game, unit) {
                return mesh;
            }

            for a in unit.to_cube().ring(1) {
                let a = a.to_axial();

                if check_is_ship(a) && cond(a, None, 0) {
                    mesh.add_normal_cell(a.sub(&unit));

                    for b in a.to_cube().ring(1) {
                        let b = b.to_axial();

                        if check_is_ship(b) && cond(b, None, 1) {
                            mesh.add_normal_cell(b.sub(&unit));
                        }
                    }
                } else {
                    let water_to_land = game.env.land.is_coord_set(a)
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
                            .is_none();

                    if typ == Type::Marine && water_to_land {
                        mesh.add_normal_cell(a.sub(&unit));
                    }
                }
            }
        }

        mesh
    }
}

pub fn has_adjacent_water(game: &GameState, kk: GridCoord) -> bool {
    for j in kk.to_cube().ring(1) {
        // if !game.world.get_game_cells().is_coord_set(j.to_axial()) {
        //     continue;
        // }
        if !game.env.land.is_coord_set(j.to_axial()) {
            return true;
        }
    }
    false
}

#[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
pub struct ActualMove {
    pub unit: GridCoord,
    pub moveto: GridCoord,
    pub attackto: GridCoord,
}

impl ActualMove {
    pub async fn execute_move_ani(
        &self,
        state: &mut GameState,
        team_index: ActiveTeam,
        doop: &mut WorkerManager<'_>,
    ) {
        let unitt = self.unit;
        let moveto = self.moveto;
        let attackto = self.attackto;

        let target_cell = moveto;
        let unit = state
            .factions
            .relative(team_index)
            .this_team
            .find_slow(&unitt)
            .unwrap();
        let mesh =
            state.generate_unit_possible_moves_inner(&unit.position, unit.typ, team_index, None);

        let unit = state
            .factions
            .relative_mut(team_index)
            .this_team
            .find_slow_mut(&unitt)
            .unwrap();

        let ttt = unit.typ;
        let iii = moves::PartialMove {
            this_unit: unit,
            target: target_cell,
            is_extra: None,
            env: &mut state.env,
        };

        let iii = iii.execute_with_animation(team_index, doop, mesh).await;

        let selected_unit = moveto;
        let target_cell = attackto;

        let mesh =
            state.generate_unit_possible_moves_inner(&selected_unit, ttt, team_index, Some(iii));

        let unit = state
            .factions
            .relative_mut(team_index)
            .this_team
            .find_slow_mut(&moveto)
            .unwrap();
        let iii = moves::PartialMove {
            this_unit: unit,
            target: target_cell,
            is_extra: Some(iii),
            env: &mut state.env,
        };
        iii.execute_with_animation(team_index, doop, mesh).await;
    }

    pub fn execute_move_no_ani(&self, state: &mut GameState, team_index: ActiveTeam) {
        let unitt = self.unit;
        let moveto = self.moveto;
        let attackto = self.attackto;

        let target_cell = moveto;
        let unit = state
            .factions
            .relative_mut(team_index)
            .this_team
            .find_slow_mut(&unitt)
            .unwrap();

        let iii = moves::PartialMove {
            this_unit: unit,
            target: target_cell,
            is_extra: None,
            env: &mut state.env,
        };

        let iii = iii.execute(team_index);

        let target_cell = attackto;

        let iii = moves::PartialMove {
            this_unit: unit,
            target: target_cell,
            is_extra: Some(iii),
            env: &mut state.env,
        };

        iii.execute(team_index);
    }
    pub fn execute_undo(&self, state: &mut GameState, team_index: ActiveTeam) {
        let unitt = self.unit;
        let moveto = self.moveto;
        let attackto = self.attackto;

        let k = state
            .factions
            .relative_mut(team_index)
            .this_team
            .find_slow_mut(&moveto)
            .unwrap();

        if state.env.forest.is_coord_set(attackto) {
            state.env.forest.set_coord(attackto, false);
        } else if state.env.land.is_coord_set(attackto) {
            state.env.land.set_coord(attackto, false);
        } else {
            unreachable!();
        }

        k.position = unitt;
    }
}

impl GameState {
    pub fn for_all_moves_fast(&mut self, team: ActiveTeam) -> Vec<moves::ActualMove> {
        let state = self;
        let mut movs = Vec::new();
        for i in 0..state.factions.relative(team).this_team.units.len() {
            let pos = state.factions.relative_mut(team).this_team.units[i].position;
            let ttt = state.factions.relative_mut(team).this_team.units[i].typ;

            let mesh = state.generate_unit_possible_moves_inner(&pos, ttt, team, None);
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

                let second_mesh =
                    state.generate_unit_possible_moves_inner(&mm, ttt, team, Some(il));

                for sm in second_mesh.iter_mesh(mm) {
                    //Don't bother applying the extra move. just generate the sigl.
                    movs.push(moves::ActualMove {
                        unit: pos,
                        moveto: mm,
                        attackto: sm,
                    })
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
        this_unit: &mut UnitData,
        target_cell: GridCoord,
        _original: GridCoord,
        env: &mut Environment,
    ) -> PartialMoveSigl {
        if !env.land.is_coord_set(target_cell) {
            env.land.set_coord(target_cell, true)
        } else {
            if !env.forest.is_coord_set(target_cell) {
                env.forest.set_coord(target_cell, true);
            }
        }

        PartialMoveSigl {
            unit: this_unit.position,
            moveto: target_cell,
        }
    }

    impl PartialMove<'_> {
        pub fn execute(self, _team: ActiveTeam) -> PartialMoveSigl {
            if let Some(extra) = self.is_extra {
                apply_extra_move(self.this_unit, self.target, extra.unit, self.env)
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
                let terrain_type = if !self.env.land.is_coord_set(self.target) {
                    animation::TerrainType::Grass
                } else {
                    if !self.env.forest.is_coord_set(self.target) {
                        animation::TerrainType::Mountain
                    } else {
                        unreachable!()
                    }
                };

                let _ = data
                    .wait_animation(
                        animation::AnimationCommand::Terrain {
                            pos: self.target,
                            terrain_type,
                        },
                        team,
                    )
                    .await;
                apply_extra_move(self.this_unit, self.target, extra.unit, &mut self.env)
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
