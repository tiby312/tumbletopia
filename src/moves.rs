use super::*;

use crate::hex::HDir;
use crate::movement::movement_mesh::SmallMesh;

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct PartialMoveSigl {
    pub unit: GridCoord,
    pub moveto: GridCoord,
}

impl GameState {
    pub fn generate_unit_possible_moves_inner2(
        &self,
        unit: &GridCoord,
        typ: Type,
        team: ActiveTeam,
        last_move: Option<GridCoord>,
    ) -> SmallMesh {
        let game = self;
        let unit = *unit;
        let mut mesh = SmallMesh::new();

        let check_if_occ = |a: GridCoord, check_fog: bool| {
            let is_world_cell = game.world.get_game_cells().is_coord_set(a);

            let jjj = if check_fog {
                !game.env.fog.is_coord_set(a)
            } else {
                true
            };

            a != unit
                && is_world_cell
                && !game.env.land.is_coord_set(a)
                && jjj
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

        if let Some(original_pos) = last_move {
            // let jj=original_pos.to_cube().dist(&unit.to_cube());

            // let k = original_pos
            // .to_cube()
            // .neighbours()
            // .filter(|x| jj==2 && check_if_occ(x.to_axial(),true) && x.dist(&unit.to_cube()) == 1).flat_map(|a|a.ring(1));

            for a in unit
                .to_cube()
                .ring(1)
                .chain(std::iter::once(original_pos.to_cube()))
            {
                let a = a.to_axial();

                if check_if_occ(a, true) {
                    mesh.add(a.sub(&unit));

                    // for a in a.to_cube().ring(1) {
                    //     let a = a.to_axial();

                    //     if check_if_occ(a, true) {
                    //         mesh.add(a.sub(&unit));
                    //     }
                    // }
                }
            }
        } else {
            for a in unit.to_cube().ring(1) {
                let a = a.to_axial();
                let dir = unit.dir_to(&a);

                if check_if_occ(a, true) {
                    mesh.add(a.sub(&unit));

                    if typ.is_warrior() {
                        for b in a.to_cube().ring(1) {
                            let b = b.to_axial();

                            if check_if_occ(b, true) {
                                mesh.add(b.sub(&unit));
                            }
                        }
                    }
                } else {
                    if let Type::Warrior { powerup } = typ {
                        if game.env.land.is_coord_set(a) {
                            let check = a.advance(dir);
                            if check_if_occ(check, true) {
                                mesh.add(a.sub(&unit));
                            }
                        }
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
pub enum ActualMove {
    Normal {
        unit: GridCoord,
        moveto: GridCoord,
        attackto: GridCoord,
        effect: move_build::UndoInfo,
    },
    Powerup {
        unit: GridCoord,
        moveto: GridCoord,
    },
}

impl ActualMove {
    pub async fn execute_move_ani(
        &self,
        state: &mut GameState,
        team_index: ActiveTeam,
        doop: &mut WorkerManager<'_>,
    ) {
        match &self {
            &ActualMove::Normal {
                unit: unitt,
                moveto,
                attackto,
                effect,
            } => {
                let unit = state
                    .factions
                    .relative(team_index)
                    .this_team
                    .find_slow(&unitt)
                    .unwrap();
                let mesh = state.generate_unit_possible_moves_inner2(
                    &unit.position,
                    unit.typ,
                    team_index,
                    None,
                );

                let ttt = unit.typ;
                let iii = moves::PartialMove {
                    this_unit: *unitt,
                    target: *moveto,
                    is_extra: None,
                    state,
                };

                let (iii, effect, k) = iii.execute_with_animation(team_index, doop, mesh).await;
                assert!(k.is_none());
                //assert!(cont);

                let selected_unit = moveto;
                let target_cell = attackto;

                let mesh = state.generate_unit_possible_moves_inner2(
                    &selected_unit,
                    ttt,
                    team_index,
                    Some(*unitt),
                );

                let iii = moves::PartialMove {
                    this_unit: *moveto,
                    target: *target_cell,
                    is_extra: Some(iii),
                    state,
                };
                iii.execute_with_animation(team_index, doop, mesh).await;
            }
            &ActualMove::Powerup { unit, moveto } => {
                let iii = moves::PartialMove {
                    this_unit: *unit,
                    target: *moveto,
                    is_extra: None,
                    state,
                };
                iii.execute(team_index);
                // assert!(state.env.land.is_coord_set(moveto));
                // state.env.land.set_coord(moveto, false);
            }
        }
    }

    pub fn execute_move_no_ani(&self, state: &mut GameState, team_index: ActiveTeam) {
        match &self {
            &ActualMove::Normal {
                unit,
                moveto,
                attackto,
                effect,
            } => {
                let iii = moves::PartialMove {
                    this_unit: *unit,
                    target: *moveto,
                    is_extra: None,
                    state,
                };

                let (iii, effect, k) = iii.execute(team_index);
                assert!(k.is_none());
                //assert!(cont);

                let target_cell = attackto;

                let iii = moves::PartialMove {
                    this_unit: *moveto,
                    target: *target_cell,
                    is_extra: Some(iii),
                    state,
                };

                iii.execute(team_index);
            }
            &ActualMove::Powerup { unit, moveto } => {
                let iii = moves::PartialMove {
                    this_unit: *unit,
                    target: *moveto,
                    is_extra: None,
                    state,
                };
                iii.execute(team_index);
                //     assert!(state.env.land.is_coord_set(moveto));
                //     state.env.land.set_coord(moveto, false);
            }
        }
    }

    pub fn execute_undo(&self, state: &mut GameState, team_index: ActiveTeam) {
        match self {
            ActualMove::Normal {
                unit,
                moveto,
                attackto,
                effect,
            } => {
                let k = move_build::ExtraPhase1 {
                    original: *unit,
                    moveto: *moveto,
                    target_cell: *attackto,
                };
                k.undo(&effect.meta, state)
                    .undo(team_index, &effect.pushpull, state);

                // move_build::undo_extra(team_index, *unit, *moveto, *attackto, &effect.meta, state);
                // move_build::undo_movement(team_index, *unit, *moveto, &effect.pushpull, state)
            }
            &ActualMove::Powerup { unit, moveto } => {
                assert!(!state.env.land.is_coord_set(moveto));
                state.env.land.set_coord(moveto, true);
                let k = state
                    .factions
                    .relative_mut(team_index)
                    .this_team
                    .find_slow_mut(&unit)
                    .unwrap();
                let Type::Warrior { powerup } = &mut k.typ else {
                    unreachable!();
                };
                *powerup = true;
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
            let ttt = state.factions.relative_mut(team).this_team.units[i].typ;

            let mesh = state.generate_unit_possible_moves_inner2(&pos, ttt, team, None);
            for mm in mesh.iter_mesh(pos) {
                //Temporarily move the player in the game world.
                //We do this so that the mesh generated for extra is accurate.
                let ii = PartialMove {
                    this_unit: pos,
                    state,
                    target: mm,
                    is_extra: None,
                };
                let (il, effect, _) = ii.execute(team);

                //if cont {
                let second_mesh =
                    state.generate_unit_possible_moves_inner2(&mm, ttt, team, Some(pos));

                for sm in second_mesh.iter_mesh(mm) {
                    assert!(!state.env.land.is_coord_set(sm));

                    let ii = PartialMove {
                        this_unit: mm,
                        state,
                        target: sm,
                        is_extra: Some(il),
                    };
                    let (il2, _, k) = ii.execute(team);
                    let k = k.unwrap();
                    let mmo = moves::ActualMove::Normal {
                        unit: pos,
                        moveto: mm,
                        attackto: sm,
                        effect: move_build::UndoInfo {
                            pushpull: effect.unwrap(),
                            meta: k.clone(),
                        },
                    };
                    //Don't bother applying the extra move. just generate the sigl.
                    movs.push(mmo);

                    //mm.execute_undo(state,team);
                    move_build::ExtraPhase1 {
                        original: pos,
                        moveto: mm,
                        target_cell: sm,
                    }
                    .undo(&k, state);
                    //move_build::undo_extra(team, pos, mm, sm, &k, state);
                }

                //revert it back just the movement component.
                move_build::MovePhase1 {
                    unit: pos,
                    target: mm,
                }
                .undo(team, &effect.unwrap(), state);
                //move_build::undo_movement(team, pos, mm, &effect.unwrap(), state);
            }
        }
        movs
    }
}

use crate::ace::WorkerManager;

pub use partial::PartialMove;
pub mod partial {
    use crate::animation::TerrainType;

    use super::*;

    #[derive(Debug)]
    pub struct PartialMove<'a> {
        pub this_unit: GridCoord,
        pub state: &'a mut GameState,
        pub target: GridCoord,
        pub is_extra: Option<PartialMoveSigl>,
    }

    impl PartialMove<'_> {
        pub fn execute(
            self,
            team: ActiveTeam,
        ) -> (
            PartialMoveSigl,
            Option<move_build::PushPullInfo>,
            Option<move_build::MetaInfo>,
        ) {
            let this_unit = self.state.factions.get_unit_mut(team, self.this_unit);

            if let Some(extra) = self.is_extra {
                let (a, b) = move_build::ExtraPhase1 {
                    original: extra.unit,
                    moveto: this_unit.position,
                    target_cell: self.target,
                }
                .apply(team, self.state);

                // let (a, b) = move_build::apply_extra_move(
                //     extra.unit,
                //     this_unit.position,
                //     self.target,
                //     self.state,
                // );
                (a, None, Some(b))
            } else {
                let (g, h, pa) = move_build::MovePhase1 {
                    unit: self.this_unit,
                    target: self.target,
                }
                .apply(team, self.state);
                (g, Some(h), None)
                // apply_normal_move(
                //     this_unit,
                //     self.target,
                //     &mut self.state.env,
                //     self.state.world,
                // )
            }
        }
        pub async fn execute_with_animation(
            mut self,
            team: ActiveTeam,
            data: &mut ace::WorkerManager<'_>,
            mesh: SmallMesh,
        ) -> (
            PartialMoveSigl,
            Option<move_build::PushPullInfo>,
            Option<move_build::MetaInfo>,
        ) {
            fn calculate_walls(position: GridCoord, state: &GameState) -> SmallMesh {
                let env = &state.env;
                let mut walls = SmallMesh::new();

                for a in position.to_cube().range(2) {
                    let a = a.to_axial();
                    //TODO this is duplicated logic in selection function???
                    let cc = env.land.is_coord_set(a);
                    if cc || (a != position && state.factions.contains(a)) {
                        walls.add(a.sub(&position));
                    }
                }

                walls
            }
            if let Some(extra) = self.is_extra {
                let terrain_type = if !self.state.env.land.is_coord_set(self.target) {
                    animation::TerrainType::Grass
                } else {
                    if !self.state.env.forest.is_coord_set(self.target) {
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
                            dir: animation::AnimationDirection::Up,
                        },
                        team,
                    )
                    .await;

                let this_unit = self.state.factions.get_unit_mut(team, self.this_unit);

                let (f, g) = move_build::ExtraPhase1 {
                    original: extra.unit,
                    moveto: this_unit.position,
                    target_cell: self.target,
                }
                .apply(team, self.state);

                (f, None, Some(g))
            } else {
                let walls = calculate_walls(self.this_unit, self.state);

                let k = move_build::MovePhase1 {
                    unit: self.this_unit,
                    target: self.target,
                };

                k.animate(team, data, walls, self.state, self.this_unit, self.target)
                    .await;

                // let info = k.generate_info(team,self.state);

                // let this_unit = self.state.factions.get_unit_mut(team, self.this_unit);

                // let _ = data
                //     .wait_animation(
                //         animation::AnimationCommand::Movement {
                //             unit: this_unit.clone(),
                //             mesh,
                //             walls,
                //             end: self.target,
                //             data: info,
                //         },
                //         team,
                //     )
                //     .await;

                let (s, a, pa) = k.apply(team, self.state);

                (s, Some(a), None)
            }
        }
    }
}
