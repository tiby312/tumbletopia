use super::*;
use crate::mesh::small_mesh::SmallMesh;

impl crate::moves::ActualMove {
    // pub fn as_extra(&self) -> move_build::ExtraPhase {
    //     move_build::ExtraPhase {
    //         original: self.original,
    //         moveto: self.moveto,
    //         target: self.attackto,
    //     }
    // }
    pub fn as_move(&self) -> move_build::MovePhase {
        move_build::MovePhase {
            dir: self.dir,
            original: self.original,
            moveto: self.moveto,
        }
    }
}

// pub struct ExtraPhase {
//     pub original: Axial,
//     pub moveto: Axial,
//     pub target: Axial,
// }
// impl ExtraPhase {
//     pub fn undo(self, meta: &ExtraEffect, state: &mut GameState) -> MovePhase {
//         let moveto = self.moveto;
//         let unit = self.original;
//         let attackto = self.target;

//         for a in meta.fog.0.iter_mesh(moveto) {
//             assert!(!state.env.fog.is_set(a));
//             state.env.fog.set_coord(a, true);
//         }

//         if moveto != attackto {
//             if let Some(m) = &meta.bomb {
//                 assert_eq!(unit, attackto);
//                 assert_eq!(unit.to_cube().dist(&moveto.to_cube()), 2);
//                 for a in m.0.iter_mesh(unit) {
//                     assert!(state.env.terrain.land.is_set(a));
//                     state.env.terrain.land.set_coord(a, false);
//                 }
//             } else if state.env.terrain.forest.is_set(attackto) {
//                 state.env.terrain.forest.set_coord(attackto, false);
//             } else if state.env.terrain.land.is_set(attackto) {
//                 state.env.terrain.land.set_coord(attackto, false);
//             } else {
//                 unreachable!();
//             }
//         }

//         MovePhase {
//             original: self.original,
//             moveto: self.moveto,
//         }
//     }
//     //returns a mesh where set bits indicate cells
//     //that were fog before this function was called,
//     //and were then unfogged.
//     fn compute_bomb(&self, game: &GameState, world: &board::MyWorld) -> Option<BombInfo> {
//         // if self.target != self.original || self.original.to_cube().dist(&self.moveto.to_cube()) != 2
//         // {
//         //     return None;
//         // }

//         // let mut mesh = SmallMesh::new();

//         // for a in self.original.to_cube().range(2).map(|a| a.to_axial()) {
//         //     if !world.get_game_cells().is_set(a) {
//         //         continue;
//         //     }

//         //     if game.factions.has_a_set(a) {
//         //         continue;
//         //     }

//         //     if game.env.terrain.is_set(a) {
//         //         continue;
//         //     }

//         //     if game.env.fog.is_set(a) {
//         //         continue;
//         //     }

//         //     mesh.add(a.sub(&self.original));
//         // }

//         // Some(BombInfo(mesh))
//         return None;
//     }

//     pub fn apply(
//         &self,
//         _team: ActiveTeam,
//         game: &mut GameState,
//         world: &board::MyWorld,
//         mov_eff: &MoveEffect,
//     ) -> ExtraEffect {
//         let original = self.original;
//         let moveto = self.moveto;
//         let target_cell = self.target;

//         if self.moveto == self.target {
//             let fog = compute_fog(moveto, &mut game.env);

//             fog.apply(moveto, &mut game.env);

//             return ExtraEffect { fog, bomb: None };
//         }

//         let bb = if let Some(bb) = self.compute_bomb(game, world) {
//             bb.apply(original, game);
//             Some(bb)
//         } else {
//             if !game.env.terrain.land.is_set(target_cell) {
//                 game.env.terrain.land.set_coord(target_cell, true)
//             } else {
//                 // if !env.forest.is_coord_set(target_cell) {
//                 //     env.forest.set_coord(target_cell, true);
//                 // }
//                 unreachable!("WAT");
//             }
//             None
//         };

//         // let bb = if target_cell == original && original.to_cube().dist(&moveto.to_cube()) == 2 {
//         //     //if false{
//         //     let bb = compute_bomb(original, game);
//         //     bb.apply(original, game);
//         //     Some(bb)
//         // } else {
//         //     if !game.env.land.is_coord_set(target_cell) {
//         //         game.env.land.set_coord(target_cell, true)
//         //     } else {
//         //         // if !env.forest.is_coord_set(target_cell) {
//         //         //     env.forest.set_coord(target_cell, true);
//         //         // }
//         //         unreachable!("WAT");
//         //     }
//         //     None
//         // };

//         let mut fog = compute_fog(moveto, &mut game.env);

//         if let PushInfo::PushedUnit = mov_eff.pushpull {
//             let dir = original.dir_to(&moveto);
//             let check = moveto.advance(dir);
//             let fog2 = compute_fog(check, &mut game.env);

//             for f in fog2.0.iter_mesh(check) {
//                 fog.0.add(f.sub(&moveto));
//             }
//         }

//         fog.apply(moveto, &mut game.env);

//         ExtraEffect { fog, bomb: bb }
//     }

//     pub async fn animate(
//         &self,
//         team: ActiveTeam,
//         state: &GameState,
//         world: &board::MyWorld,
//         data: &mut ace::WorkerManager,
//     ) -> &Self {
//         let target = self.target;

//         let mut gg = state.clone();

//         if let Some(bb) = self.compute_bomb(state, world) {
//             let k = self.original.to_cube();
//             for a in std::iter::once(k).chain(k.ring(1)).chain(k.ring(2)) {
//                 if bb.0.is_set(a.sub(self.original.to_cube()).to_axial()) {
//                     data.wait_animation(
//                         animation::AnimationCommand::Terrain {
//                             pos: a.to_axial(),
//                             terrain_type: animation::TerrainType::Grass,
//                             dir: animation::AnimationDirection::Up,
//                         },
//                         team,
//                         &mut gg,
//                     )
//                     .await;
//                     gg.env.terrain.land.set_coord(a.to_axial(), true);
//                 }
//             }
//         } else {
//             data.wait_animation(
//                 animation::AnimationCommand::Terrain {
//                     pos: target,
//                     terrain_type: animation::TerrainType::Grass,
//                     dir: animation::AnimationDirection::Up,
//                 },
//                 team,
//                 &mut gg,
//             )
//             .await;

//             gg.env.terrain.land.set_coord(target, true);
//         }

//         let fog = compute_fog(self.moveto, &state.env);

//         //let mut game = state.clone();
//         for a in fog.0.iter_mesh(self.moveto) {
//             gg.env.fog.set_coord(a, false);
//             // Change mesh
//             data.wait_animation(
//                 animation::AnimationCommand::Terrain {
//                     pos: a,
//                     terrain_type: animation::TerrainType::Fog,
//                     dir: animation::AnimationDirection::Down,
//                 },
//                 team,
//                 &mut gg,
//             )
//             .await;
//         }

//         self
//     }
// }

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct MoveEffect {
    pub destroyed_unit: Option<UnitType>,
    pub promoted: bool,
}

#[derive(Clone, Debug)]
pub struct MovePhase {
    pub dir: OParity,
    pub original: Axial,
    pub moveto: Axial,
}
impl MovePhase {
    pub async fn animate(
        &self,
        team: ActiveTeam,
        state: &GameState,
        world: &board::MyWorld,
        data: &mut ace::WorkerManager,
    ) -> &Self {
        //TODO remove

        let target = self.moveto;
        //let paths = calculate_paths(self.original, self.moveto, state, world);

        //assert!(state.factions.get_all_team(team).is_set(self.original));

        //let mesh = state.generate_possible_moves_movement(world, &self.original, team);

        let info = {
            let target_cell = self.moveto;
            let mut e = PushInfo::None;

            if state.env.terrain.land.is_set(target_cell) {
                e = PushInfo::PushedLand;
            }

            e
        };
        //let this_unit = state.factions.get_unit(team, this_unit);

        let mut ss = state.clone();

        let (ttt, _) = ss.factions.get_board_mut(self.dir).remove(self.original);

        let end = target;
        match info {
            PushInfo::PushedLand => {
                let dir = self.original.dir_to(&end);
                let k = self.original.advance(dir);
                assert!(ss.env.terrain.land.is_set(k));
                ss.env.terrain.land.set_coord(k, false);
            }
            PushInfo::UpgradedLand => {
                //TODO fooo
            }
            PushInfo::PushedUnit => {
                //TODO animate
            }

            PushInfo::None => {}
        }

        //let capturing = state.factions.get_all_team(team.not()).is_set(end);
        //if !capturing {
        // let path = mesh::path(
        //     &mesh,
        //     self.original,
        //     self.moveto,
        //     &paths,
        //     state,
        //     team,
        //     world,
        //     capturing,
        // );

        data.wait_animation(
            animation::AnimationCommand::Movement {
                unit: self.original,
                ttt,
                end,
                data: info,
                parity: self.dir,
            },
            team,
            &mut ss,
        )
        .await;
        //}
        self
    }

    pub fn undo(&self, team_index: ActiveTeam, effect: &MoveEffect, state: &mut GameState) {
        let moveto = self.moveto;
        let unit = self.original;

        if effect.promoted {
            let (u, t) = state.factions.get_board_mut(self.dir.flip()).remove(moveto);
            state.factions.get_board_mut(self.dir.flip()).add_piece(
                moveto,
                team_index,
                UnitType::Pawn,
            );
        }

        state.factions.flip(moveto);

        state
            .factions
            .get_board_mut(self.dir)
            .move_unit(moveto, unit);

        // let curr = state.factions.parity.is_set(moveto);
        // state.factions.parity.set(unit, !curr);
        // state.factions.parity.remove(moveto);

        if let Some((typ)) = effect.destroyed_unit {
            //matches!(effect.pushpull, PushInfo::None);
            //TODO need to store parity of taken piece!!!!
            state
                .factions
                .get_board_mut(self.dir)
                .add_piece(moveto, team_index.not(), typ);

            //let j = &mut state.factions.relative_mut(team_index).that_team.units;
            //assert_eq!(fooo, moveto);
            //j.set_coord(moveto, true);
        }

        // match effect.pushpull {
        //     PushInfo::UpgradedLand => {
        //         assert_eq!(unit.to_cube().dist(&moveto.to_cube()), 1);

        //         let dir = unit.dir_to(&moveto);
        //         let t3 = moveto.advance(dir);

        //         if state.env.terrain.land.is_set(t3) {
        //             panic!("This is impossible!");
        //         } else if state.env.terrain.forest.is_set(t3) {
        //             state.env.terrain.forest.set_coord(t3, false);
        //             state.env.terrain.land.set_coord(t3, true);
        //             state.env.terrain.land.set_coord(moveto, true);
        //         } else if state.env.terrain.mountain.is_set(t3) {
        //             state.env.terrain.mountain.set_coord(t3, false);
        //             state.env.terrain.forest.set_coord(t3, true);
        //             state.env.terrain.forest.set_coord(moveto, true);
        //         }
        //     }
        //     PushInfo::PushedUnit => {
        //         todo!()
        //         // assert_eq!(unit.to_cube().dist(&moveto.to_cube()), 1);
        //         // let dir = unit.dir_to(&moveto);
        //         // let t3 = moveto.advance(dir);

        //         // let tt = state.factions.relative_mut(team_index);
        //         // if tt.this_team.units.is_set(t3) {
        //         //     tt.this_team.units.set_coord(t3, false);
        //         //     tt.this_team.units.set_coord(moveto, true);
        //         // } else if tt.that_team.units.is_set(t3) {
        //         //     tt.that_team.units.set_coord(t3, false);
        //         //     tt.that_team.units.set_coord(moveto, true);
        //         // } else {
        //         //     unreachable!("PushedUnit enum error");
        //         // }
        //     }
        //     PushInfo::PushedLand => {
        //         assert_eq!(unit.to_cube().dist(&moveto.to_cube()), 1);

        //         let dir = unit.dir_to(&moveto);
        //         let t3 = moveto.advance(dir);

        //         if state.env.terrain.land.is_set(t3) {
        //             state.env.terrain.land.set_coord(t3, false);
        //             state.env.terrain.land.set_coord(moveto, true);
        //         } else if state.env.terrain.forest.is_set(t3) {
        //             state.env.terrain.forest.set_coord(t3, false);
        //             state.env.terrain.forest.set_coord(moveto, true);
        //         } else if state.env.terrain.mountain.is_set(t3) {
        //             state.env.terrain.mountain.set_coord(t3, false);
        //             state.env.terrain.mountain.set_coord(moveto, true);
        //         }

        //         // assert!(state.env.terrain.land.is_set(t3));
        //         // state.env.terrain.land.set_coord(t3, false);
        //         // assert!(!state.env.terrain.land.is_set(moveto));
        //         // state.env.terrain.land.set_coord(moveto, true);
        //     }

        //     PushInfo::None => {}
        // }
    }

    pub fn apply(
        &self,
        team: ActiveTeam,
        game: &mut GameState,
        world: &board::MyWorld,
    ) -> (MoveEffect, GameFinishingMove) {
        let env = &mut game.env;
        let target_cell = self.moveto;
        //let mut e = PushInfo::None;

        let mut destroyed_unit = None;

        let mut finish = GameFinishingMove::NotFinished;
        {
            let terrain = &mut env.terrain;

            if game
                .factions
                .get_board_mut(self.dir)
                .get_all_team(team.not())
                .is_set(target_cell)
            {
                let (k, pp) = game.factions.get_board_mut(self.dir).remove(target_cell);
                destroyed_unit = Some(k);

                if let UnitType::King = k {
                    let k = if team == ActiveTeam::White {
                        GameOver::WhiteWon
                    } else {
                        GameOver::BlackWon
                    };

                    finish = GameFinishingMove::Finished(k);
                }
            }
        }

        // let powerup = if game.env.powerups.contains(&target_cell) {
        //     game.env.powerups.retain(|&a| a != target_cell);
        //     unreachable!()
        //     // if !this_unit.has_powerup {
        //     //     this_unit.has_powerup = true;
        //     //     PowerupAction::GotPowerup
        //     // } else {
        //     //     // powerup is discarded
        //     //     PowerupAction::DiscardedPowerup
        //     // }
        // } else {
        //     PowerupAction::None
        // };

        let mut target_cell = target_cell;

        game.factions
            .get_board_mut(self.dir)
            .move_unit(self.original, target_cell);
        game.factions.flip(target_cell);

        let promoted = if game
            .factions
            .get_board(self.dir.flip())
            .get_unit_at(target_cell)
            .0
            == UnitType::Pawn
        {
            if team == ActiveTeam::White {
                if target_cell.q == 0 {
                    true
                } else {
                    false
                }
            } else {
                if target_cell.q == 7 {
                    true
                } else {
                    false
                }
            }
        } else {
            false
        };

        if promoted {
            let (u, t) = game
                .factions
                .get_board_mut(self.dir.flip())
                .remove(target_cell);
            game.factions.get_board_mut(self.dir.flip()).add_piece(
                target_cell,
                team,
                UnitType::Queen,
            );
        }

        (
            MoveEffect {
                //pushpull: e,
                //powerup,
                promoted,
                destroyed_unit,
            },
            finish,
        )
    }
}

// fn undo_transfer_other_board(game: &mut GameState, team: ActiveTeam, mut orig: Axial) {
//     let orig2 = orig;
//     if orig.q >= 0 {
//         orig.q -= 8;
//     } else {
//         orig.q += 8;
//     }
//     game.factions
//         .relative_mut(team)
//         .this_team
//         .move_unit(orig, orig2);
// }

// fn transfer_other_board(game: &mut GameState, team: ActiveTeam, orig: Axial) {
//     let mut target_cell = orig;
//     if target_cell.q >= 0 {
//         target_cell.q -= 8;
//     } else {
//         target_cell.q += 8;
//     }
//     game.factions
//         .relative_mut(team)
//         .this_team
//         .move_unit(orig, target_cell);
// }

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub enum PowerupAction {
    GotPowerup,
    DiscardedPowerup,
    None,
}

#[derive(PartialOrd, Ord, Clone, Copy, Eq, PartialEq, Debug)]
pub enum PushInfo {
    UpgradedLand,
    PushedLand,
    PushedUnit,
    None,
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct CombinedEffect {
    pub move_effect: MoveEffect,
    pub extra_effect: ExtraEffect,
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
struct BombInfo(pub SmallMesh);
impl BombInfo {
    fn apply(&self, original: Axial, game: &mut GameState) {
        for a in self.0.iter_mesh(Axial::zero()) {
            game.env.terrain.land.set_coord(original.add(a), true);
        }
    }
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct ExtraEffect {
    fog: FogInfo,
    bomb: Option<BombInfo>,
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct FogInfo(pub SmallMesh);

impl FogInfo {
    pub fn apply(&self, og: Axial, env: &mut Environment) {
        for a in self.0.iter_mesh(Axial::zero()) {
            env.fog.set_coord(og.add(a), false);
        }
    }
}

//returns a mesh where set bits indicate cells
//that were fog before this function was called,
//and were then unfogged.
pub fn compute_fog(og: Axial, env: &Environment) -> FogInfo {
    let mut mesh = SmallMesh::new();
    for a in og.to_cube().range(1) {
        if env.fog.is_set(a.to_axial()) {
            mesh.add(a.to_axial().sub(&og));
        }
    }
    FogInfo(mesh)
}

// fn calculate_paths(
//     position: Axial,
//     target: Axial,
//     state: &GameState,
//     world: &board::MyWorld,
// ) -> SmallMesh {
//     let typ = state.factions.units.get_type(position);

//     let env = &state.env;
//     let mut paths = SmallMesh::new();

//     paths.add(target.sub(&position));

//     for a in position.to_cube().range(2) {
//         let a = a.to_axial();
//         //TODO this is duplicated logic in selection function???

//         if !env.fog.is_set(a)
//             && !env.terrain.is_set(a)
//             && a != position
//             && !state.factions.units.is_set(a)
//             && world.get_game_cells().is_set(a)
//         {
//             //if a != target {
//             paths.add(a.sub(&position));
//             //}
//         }
//     }
//     // match typ {
//     //     UnitType::Mouse => {
//     //         paths.add(target.sub(&position));

//     //         for a in position.to_cube().range(2) {
//     //             let a = a.to_axial();
//     //             //TODO this is duplicated logic in selection function???

//     //             if !env.fog.is_set(a)
//     //                 && !env.terrain.is_set(a)
//     //                 && a != position
//     //                 && !state.factions.has_a_set(a)
//     //                 && world.get_game_cells().is_set(a)
//     //             {
//     //                 //if a != target {
//     //                 paths.add(a.sub(&position));
//     //                 //}
//     //             }
//     //         }
//     //     }
//     //     UnitType::Rabbit => {
//     //         paths.add(target.sub(&position));

//     //         for a in position.to_cube().range(2) {
//     //             let a = a.to_axial();
//     //             //TODO this is duplicated logic in selection function???

//     //             if env.terrain.is_set(a) {
//     //                 paths.add(a.sub(&position));
//     //             }
//     //         }

//     //         let pos: Vec<_> = paths.iter_mesh(position).collect();

//     //         console_dbg!("walls size={}", pos);
//     //     }
//     // }

//     paths
// }
