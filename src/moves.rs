use super::*;

use crate::movement::movement_mesh::SmallMesh;

// #[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
// pub struct PartialMoveSigl {
//     pub unit: GridCoord,
//     pub moveto: GridCoord,
// }

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
pub struct ActualMove {
    pub original: GridCoord,
    pub moveto: GridCoord,
    pub attackto: GridCoord,
    pub effect: move_build::UndoInfo,
}

impl ActualMove {
    // pub async fn execute_move_ani(
    //     &self,
    //     state: &mut GameState,
    //     team: ActiveTeam,
    //     doop: &mut WorkerManager<'_>,
    // ) {
    //     let ActualMove {
    //         original,
    //         moveto,
    //         attackto,
    //         effect,
    //     } = self;
    //     let kk = move_build::MovePhase {
    //         original: *original,
    //         moveto: *moveto,
    //     };

    //     let effect = kk.animate(team, doop, state).await.apply(team, state);

    //     let a = kk
    //         .into_attack(*attackto)
    //         .animate(team, state, doop)
    //         .await
    //         .apply(team, state);
    // }

    pub fn execute_move_no_ani(&self, state: &mut GameState, team_index: ActiveTeam) {
        let ActualMove {
            original: unit,
            moveto,
            attackto,
            effect,
        } = self;

        let effect = move_build::MovePhase {
            original: *unit,
            moveto: *moveto,
        }
        .apply(team_index, state);

        let target_cell = attackto;

        let _ = move_build::ExtraPhase {
            original: *unit,
            moveto: *moveto,
            target: *target_cell,
        }
        .apply(team_index, state);
    }

    pub fn execute_undo(&self, state: &mut GameState, team_index: ActiveTeam) {
        let ActualMove {
            original: unit,
            moveto,
            attackto,
            effect,
        } = self;

        let k = move_build::ExtraPhase {
            original: *unit,
            moveto: *moveto,
            target: *attackto,
        };
        k.undo(&effect.extra_effect, state)
            .undo(team_index, &effect.move_effect, state);
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
                let mmm = move_build::MovePhase {
                    original: pos,
                    moveto: mm,
                };
                let effect = mmm.apply(team, state);

                let second_mesh =
                    state.generate_unit_possible_moves_inner2(&mm, ttt, team, Some(pos));

                for sm in second_mesh.iter_mesh(mm) {
                    assert!(!state.env.land.is_coord_set(sm));

                    let kkk = move_build::ExtraPhase {
                        original: pos,
                        moveto: mm,
                        target: sm,
                    };

                    let k = kkk.apply(team, state);

                    let mmo = moves::ActualMove {
                        original: pos,
                        moveto: mm,
                        attackto: sm,
                        effect: move_build::UndoInfo {
                            move_effect: effect.clone(),
                            extra_effect: k.clone(),
                        },
                    };
                    //Don't bother applying the extra move. just generate the sigl.
                    movs.push(mmo);

                    kkk.undo(&k, state);
                }

                //revert it back just the movement component.
                mmm.undo(team, &effect, state);
            }
        }
        movs
    }
}

use crate::ace::WorkerManager;
