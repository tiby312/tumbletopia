use super::*;

use crate::hex::HDir;
use crate::movement::movement_mesh::SmallMesh;

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

        if let Some(last_move) = last_move {
            for a in unit.to_cube().ring(1) {
                let a = a.to_axial();

                if check_if_occ(a, true) {
                    mesh.add(a.sub(&unit));

                    for a in a.to_cube().ring(1) {
                        let a = a.to_axial();

                        if check_if_occ(a, true) {
                            mesh.add(a.sub(&unit));
                        }
                    }
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
                            if check_if_occ(check, false) {
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
        effect: UndoInfo,
    },
    Powerup {
        unit: GridCoord,
        moveto: GridCoord,
    },
}

//returns a mesh where set bits indicate cells
//that were fog before this function was called,
//and were then unfogged.
pub fn uncover_fog(og: GridCoord, env: &mut Environment) -> SmallMesh {
    let mut mesh = SmallMesh::new();
    for a in og.to_cube().range(1) {
        if env.fog.is_coord_set(a.to_axial()) {
            mesh.add(a.to_axial().sub(&og));
        }
    }

    for a in mesh.iter_mesh(GridCoord([0; 2])) {
        env.fog.set_coord(og.add(a), false);
    }
    mesh
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
                let mesh = state.generate_unit_possible_moves_inner(
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

                let (iii, effect) = iii.execute_with_animation(team_index, doop, mesh).await;

                //assert!(cont);

                let selected_unit = moveto;
                let target_cell = attackto;

                let mesh = state.generate_unit_possible_moves_inner(
                    &selected_unit,
                    ttt,
                    team_index,
                    Some(iii),
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

                let (iii, effect) = iii.execute(team_index);

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
                if state.env.forest.is_coord_set(*attackto) {
                    state.env.forest.set_coord(*attackto, false);
                } else if state.env.land.is_coord_set(*attackto) {
                    state.env.land.set_coord(*attackto, false);
                } else {
                    unreachable!();
                }

                undo_movement(team_index, *unit, *moveto, effect, state)
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

            let mesh = state.generate_unit_possible_moves_inner(&pos, ttt, team, None);
            for mm in mesh.iter_mesh(pos) {
                //Temporarily move the player in the game world.
                //We do this so that the mesh generated for extra is accurate.
                let ii = PartialMove {
                    this_unit: pos,
                    state,
                    target: mm,
                    is_extra: None,
                };
                let (il, effect) = ii.execute(team);

                //if cont {
                let second_mesh =
                    state.generate_unit_possible_moves_inner(&mm, ttt, team, Some(il));

                for sm in second_mesh.iter_mesh(mm) {
                    assert!(!state.env.land.is_coord_set(sm));

                    //Don't bother applying the extra move. just generate the sigl.
                    movs.push(moves::ActualMove::Normal {
                        unit: pos,
                        moveto: mm,
                        attackto: sm,
                        effect: effect.clone().unwrap(),
                    })
                }

                //console_dbg!(effect);

                // if let UndoInformation::PulledLand=effect{
                //     console_dbg!("YOOOOOOOO");
                // }

                //revert it back just the movement component.
                undo_movement(team, pos, mm, &effect.unwrap(), state);

                //state.factions.relative_mut(team).this_team.units[i].position = pos;
                // } else {
                //     let j = moves::ActualMove::Powerup {
                //         unit: pos,
                //         moveto: mm,
                //     };
                //     movs.push(j.clone());
                //     j.execute_undo(state, team);
                // }
            }
        }
        movs
    }
}

use crate::ace::WorkerManager;

#[derive(PartialOrd, Ord, Clone, Copy, Eq, PartialEq, Debug)]
pub enum PushPullInfo {
    PushedLand,
    PulledLand,
    None,
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct UndoInfo {
    pub pushpull: PushPullInfo,
    pub fog: SmallMesh,
}

pub fn undo_movement(
    team_index: ActiveTeam,
    unit: GridCoord,
    moveto: GridCoord,
    effect: &UndoInfo,
    state: &mut GameState,
) {
    let k = state
        .factions
        .relative_mut(team_index)
        .this_team
        .find_slow_mut(&moveto)
        .unwrap();

    for a in effect.fog.iter_mesh(moveto) {
        assert!(!state.env.fog.is_coord_set(a));
        state.env.fog.set_coord(a, true);
    }

    match effect.pushpull {
        PushPullInfo::PushedLand => {
            let dir = unit.dir_to(&moveto);
            let t3 = moveto.advance(dir);
            assert!(state.env.land.is_coord_set(t3));
            state.env.land.set_coord(t3, false);
            assert!(!state.env.land.is_coord_set(moveto));
            state.env.land.set_coord(moveto, true);
        }
        PushPullInfo::PulledLand => {
            let dir = unit.dir_to(&moveto);
            let t3 = unit.back(dir);
            assert!(state.env.land.is_coord_set(unit));
            state.env.land.set_coord(unit, false);
            assert!(!state.env.land.is_coord_set(t3));
            state.env.land.set_coord(t3, true);
        }
        PushPullInfo::None => {}
    }
    k.position = unit;
}

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

    //TODO wrap in private
    pub struct MovePhase1 {
        unit: GridCoord,
        target: GridCoord,
        team: ActiveTeam,
    }
    impl MovePhase1 {
        pub fn generate_info(&self, game: &GameState) -> PushPullInfo {
            let this_unit = game.factions.get_unit(self.team, self.unit);
            let target_cell = self.target;
            let mut e = PushPullInfo::None;
            match this_unit.typ {
                Type::Warrior { .. } => {
                    if game.env.land.is_coord_set(target_cell) {
                        e = PushPullInfo::PushedLand;
                    }
                }
                Type::Archer => {
                    if game.env.land.is_coord_set(target_cell) {
                        e = PushPullInfo::PushedLand;
                    }
                    // let dir = self.unit.dir_to(&self.target);
                    // let k = self.unit.back(dir);
                    // if game.env.land.is_coord_set(k) {
                    //     e = UndoInformation::PulledLand;
                    // }
                }
            }

            e
        }
        pub fn execute(self, game: &mut GameState) -> (PartialMoveSigl, UndoInfo) {
            let env = &mut game.env;
            let this_unit = game.factions.get_unit_mut(self.team, self.unit);
            let target_cell = self.target;
            let mut e = PushPullInfo::None;

            match this_unit.typ {
                Type::Warrior { .. } => {
                    if env.land.is_coord_set(target_cell) {
                        let dir = this_unit.position.dir_to(&target_cell);

                        env.land.set_coord(target_cell, false);

                        let kk = target_cell.advance(dir);

                        env.land.set_coord(kk, true);

                        e = PushPullInfo::PushedLand;
                    }
                }
                Type::Archer => {
                    unreachable!();
                    // if env.land.is_coord_set(target_cell) {
                    //     let dir = this_unit.position.dir_to(&target_cell);

                    //     env.land.set_coord(target_cell, false);

                    //     let kk = target_cell.advance(dir);

                    //     env.land.set_coord(kk, true);

                    //     e = UndoInformation::PushedLand;
                    // }
                }
            }

            let orig = this_unit.position;

            this_unit.position = target_cell;
            let fog = uncover_fog(target_cell, env);

            (
                PartialMoveSigl {
                    unit: orig,
                    moveto: target_cell,
                },
                UndoInfo { pushpull: e, fog },
            )
        }
    }

    fn apply_extra_move(
        original: GridCoord,
        moveto: GridCoord,
        target_cell: GridCoord,
        game: &mut GameState,
    ) -> PartialMoveSigl {
        if !game.env.land.is_coord_set(target_cell) {
            game.env.land.set_coord(target_cell, true)
        } else {
            // if !env.forest.is_coord_set(target_cell) {
            //     env.forest.set_coord(target_cell, true);
            // }
            unreachable!("WAT");
        }

        PartialMoveSigl {
            unit: moveto,
            moveto: target_cell,
        }
    }

    impl PartialMove<'_> {
        pub fn execute(self, team: ActiveTeam) -> (PartialMoveSigl, Option<UndoInfo>) {
            let this_unit = self.state.factions.get_unit_mut(team, self.this_unit);

            if let Some(extra) = self.is_extra {
                (
                    apply_extra_move(extra.unit, this_unit.position, self.target, self.state),
                    None,
                )
            } else {
                let (g, h) = MovePhase1 {
                    unit: self.this_unit,
                    target: self.target,
                    team,
                }
                .execute(self.state);
                (g, Some(h))
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
        ) -> (PartialMoveSigl, Option<UndoInfo>) {
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

                (
                    apply_extra_move(extra.unit, this_unit.position, self.target, self.state),
                    None,
                )
            } else {
                let walls = calculate_walls(self.this_unit, self.state);

                let k = MovePhase1 {
                    unit: self.this_unit,
                    target: self.target,
                    team,
                };
                let info = k.generate_info(self.state);

                let this_unit = self.state.factions.get_unit_mut(team, self.this_unit);

                let _ = data
                    .wait_animation(
                        animation::AnimationCommand::Movement {
                            unit: this_unit.clone(),
                            mesh,
                            walls,
                            end: self.target,
                            data: info,
                        },
                        team,
                    )
                    .await;

                let (s, a) = k.execute(self.state);

                (s, Some(a))
            }
        }
    }
}
