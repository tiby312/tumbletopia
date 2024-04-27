use super::*;
use crate::mesh::small_mesh::SmallMesh;

impl crate::moves::ActualMove {
    pub fn as_extra(&self) -> move_build::ExtraPhase {
        move_build::ExtraPhase {
            original: self.original,
            moveto: self.moveto,
            target: self.attackto,
        }
    }
    pub fn as_move(&self) -> move_build::MovePhase {
        move_build::MovePhase {
            original: self.original,
            moveto: self.moveto,
        }
    }
}

pub struct ExtraPhase {
    pub original: Axial,
    pub moveto: Axial,
    pub target: Axial,
}
impl ExtraPhase {
    pub fn undo(self, meta: &ExtraEffect, state: &mut GameState) -> MovePhase {
        let moveto = self.moveto;
        let unit = self.original;
        let attackto = self.target;

        for a in meta.fog.0.iter_mesh(moveto) {
            assert!(!state.env.fog.is_set(a));
            state.env.fog.set_coord(a, true);
        }

        if let Some(m) = &meta.bomb {
            assert_eq!(unit, attackto);
            assert_eq!(unit.to_cube().dist(&moveto.to_cube()), 2);
            for a in m.0.iter_mesh(unit) {
                assert!(state.env.terrain.land.is_set(a));
                state.env.terrain.land.set_coord(a, false);
            }
        } else if state.env.terrain.forest.is_set(attackto) {
            state.env.terrain.forest.set_coord(attackto, false);
        } else if state.env.terrain.land.is_set(attackto) {
            state.env.terrain.land.set_coord(attackto, false);
        } else {
            unreachable!();
        }

        MovePhase {
            original: self.original,
            moveto: self.moveto,
        }
    }
    //returns a mesh where set bits indicate cells
    //that were fog before this function was called,
    //and were then unfogged.
    fn compute_bomb(&self, game: &GameState, world: &board::MyWorld) -> Option<BombInfo> {
        if self.target != self.original || self.original.to_cube().dist(&self.moveto.to_cube()) != 2
        {
            return None;
        }

        let mut mesh = SmallMesh::new();

        for a in self.original.to_cube().range(2).map(|a| a.to_axial()) {
            if !world.get_game_cells().is_set(a) {
                continue;
            }

            if game.factions.has_a_set(a) {
                continue;
            }

            if game.env.terrain.is_set(a) {
                continue;
            }

            if game.env.fog.is_set(a) {
                continue;
            }

            mesh.add(a.sub(&self.original));
        }

        Some(BombInfo(mesh))
    }

    pub fn apply(
        &self,
        _team: ActiveTeam,
        game: &mut GameState,
        world: &board::MyWorld,
        mov_eff: &MoveEffect,
    ) -> ExtraEffect {
        let original = self.original;
        let moveto = self.moveto;
        let target_cell = self.target;

        let bb = if let Some(bb) = self.compute_bomb(game, world) {
            bb.apply(original, game);
            Some(bb)
        } else {
            if !game.env.terrain.land.is_set(target_cell) {
                game.env.terrain.land.set_coord(target_cell, true)
            } else {
                // if !env.forest.is_coord_set(target_cell) {
                //     env.forest.set_coord(target_cell, true);
                // }
                unreachable!("WAT");
            }
            None
        };

        // let bb = if target_cell == original && original.to_cube().dist(&moveto.to_cube()) == 2 {
        //     //if false{
        //     let bb = compute_bomb(original, game);
        //     bb.apply(original, game);
        //     Some(bb)
        // } else {
        //     if !game.env.land.is_coord_set(target_cell) {
        //         game.env.land.set_coord(target_cell, true)
        //     } else {
        //         // if !env.forest.is_coord_set(target_cell) {
        //         //     env.forest.set_coord(target_cell, true);
        //         // }
        //         unreachable!("WAT");
        //     }
        //     None
        // };

        let mut fog = compute_fog(moveto, &mut game.env);

        if let PushInfo::PushedUnit = mov_eff.pushpull {
            let dir = original.dir_to(&moveto);
            let check = moveto.advance(dir);
            let fog2 = compute_fog(check, &mut game.env);
            //console_dbg!("HAAAAAY",fog,fog2);

            for f in fog2.0.iter_mesh(check) {
                fog.0.add(f.sub(&moveto));
            }
            //TODO put this in a function
            //fog.0.inner|=fog2.0.inner;
        }

        fog.apply(moveto, &mut game.env);

        ExtraEffect { fog, bomb: bb }
    }

    pub async fn animate(
        &self,
        team: ActiveTeam,
        state: &GameState,
        world: &board::MyWorld,
        data: &mut ace::WorkerManager,
    ) -> &Self {
        let target = self.target;

        let mut gg = state.clone();

        if let Some(bb) = self.compute_bomb(state, world) {
            let k = self.original.to_cube();
            for a in std::iter::once(k).chain(k.ring(1)).chain(k.ring(2)) {
                if bb.0.is_set(a.sub(self.original.to_cube()).to_axial()) {
                    data.wait_animation(
                        animation::AnimationCommand::Terrain {
                            pos: a.to_axial(),
                            terrain_type: animation::TerrainType::Grass,
                            dir: animation::AnimationDirection::Up,
                        },
                        team,
                        &mut gg,
                    )
                    .await;
                    gg.env.terrain.land.set_coord(a.to_axial(), true);
                }
            }
        } else {
            data.wait_animation(
                animation::AnimationCommand::Terrain {
                    pos: target,
                    terrain_type: animation::TerrainType::Grass,
                    dir: animation::AnimationDirection::Up,
                },
                team,
                &mut gg,
            )
            .await;

            gg.env.terrain.land.set_coord(target, true);
        }

        let fog = compute_fog(self.moveto, &state.env);

        //let mut game = state.clone();
        for a in fog.0.iter_mesh(self.moveto) {
            gg.env.fog.set_coord(a, false);
            // Change mesh
            data.wait_animation(
                animation::AnimationCommand::Terrain {
                    pos: a,
                    terrain_type: animation::TerrainType::Fog,
                    dir: animation::AnimationDirection::Down,
                },
                team,
                &mut gg,
            )
            .await;
        }

        self
    }
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct MoveEffect {
    pushpull: PushInfo,
    powerup: PowerupAction,
    destroyed_unit: Option<Axial>,
}
impl MoveEffect {
    pub fn combine(self, extra_effect: ExtraEffect) -> CombinedEffect {
        CombinedEffect {
            move_effect: self,
            extra_effect,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MovePhase {
    pub original: Axial,
    pub moveto: Axial,
}
impl MovePhase {
    pub fn into_attack(self, target: Axial) -> ExtraPhase {
        ExtraPhase {
            original: self.original,
            moveto: self.moveto,
            target,
        }
    }
    pub async fn animate(
        &self,
        team: ActiveTeam,
        state: &GameState,
        world: &board::MyWorld,
        data: &mut ace::WorkerManager,
    ) -> &Self {
        let target = self.moveto;
        let walls = calculate_walls(self.original, state, world);

        assert!(state
            .factions
            .relative(team)
            .this_team
            .units
            .is_set(self.original));

        let mesh = state.generate_possible_moves_movement(world, &self.original, team);

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

        ss.factions
            .relative_mut(team)
            .this_team
            .units
            .set_coord(self.original, false);

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

        data.wait_animation(
            animation::AnimationCommand::Movement {
                unit: self.original,
                mesh,
                walls,
                end,
                data: info,
            },
            team,
            &mut ss,
        )
        .await;
        self
    }

    pub fn undo(&self, team_index: ActiveTeam, effect: &MoveEffect, state: &mut GameState) {
        let moveto = self.moveto;
        let unit = self.original;

        let jj = &mut state.factions.relative_mut(team_index).this_team.units;
        jj.set_coord(moveto, false);
        jj.set_coord(unit, true);

        if let Some(fooo) = effect.destroyed_unit {
            matches!(effect.pushpull, PushInfo::None);
            let j = &mut state.factions.relative_mut(team_index).that_team.units;
            assert_eq!(fooo, moveto);
            j.set_coord(moveto, true);
        }

        match effect.pushpull {
            PushInfo::UpgradedLand => {
                assert_eq!(unit.to_cube().dist(&moveto.to_cube()), 1);

                let dir = unit.dir_to(&moveto);
                let t3 = moveto.advance(dir);

                if state.env.terrain.land.is_set(t3) {
                    panic!("This is impossible!");
                } else if state.env.terrain.forest.is_set(t3) {
                    state.env.terrain.forest.set_coord(t3, false);
                    state.env.terrain.land.set_coord(t3, true);
                    state.env.terrain.land.set_coord(moveto, true);
                } else if state.env.terrain.mountain.is_set(t3) {
                    state.env.terrain.mountain.set_coord(t3, false);
                    state.env.terrain.forest.set_coord(t3, true);
                    state.env.terrain.forest.set_coord(moveto, true);
                }
            }
            PushInfo::PushedUnit => {
                assert_eq!(unit.to_cube().dist(&moveto.to_cube()), 1);
                let dir = unit.dir_to(&moveto);
                let t3 = moveto.advance(dir);

                let tt = state.factions.relative_mut(team_index);
                if tt.this_team.units.is_set(t3) {
                    tt.this_team.units.set_coord(t3, false);
                    tt.this_team.units.set_coord(moveto, true);
                } else if tt.that_team.units.is_set(t3) {
                    tt.that_team.units.set_coord(t3, false);
                    tt.that_team.units.set_coord(moveto, true);
                } else {
                    unreachable!("PushedUnit enum error");
                }
            }
            PushInfo::PushedLand => {
                assert_eq!(unit.to_cube().dist(&moveto.to_cube()), 1);

                let dir = unit.dir_to(&moveto);
                let t3 = moveto.advance(dir);

                if state.env.terrain.land.is_set(t3) {
                    state.env.terrain.land.set_coord(t3, false);
                    state.env.terrain.land.set_coord(moveto, true);
                } else if state.env.terrain.forest.is_set(t3) {
                    state.env.terrain.forest.set_coord(t3, false);
                    state.env.terrain.forest.set_coord(moveto, true);
                } else if state.env.terrain.mountain.is_set(t3) {
                    state.env.terrain.mountain.set_coord(t3, false);
                    state.env.terrain.mountain.set_coord(moveto, true);
                }

                // assert!(state.env.terrain.land.is_set(t3));
                // state.env.terrain.land.set_coord(t3, false);
                // assert!(!state.env.terrain.land.is_set(moveto));
                // state.env.terrain.land.set_coord(moveto, true);
            }

            PushInfo::None => {}
        }
    }

    pub fn apply(
        &self,
        team: ActiveTeam,
        game: &mut GameState,
        world: &board::MyWorld,
    ) -> MoveEffect {
        let env = &mut game.env;
        let target_cell = self.moveto;
        let mut e = PushInfo::None;

        let mut destroyed_unit = None;

        // let this_unit=move |factions:&mut Factions|{
        //     factions.relative_mut(team).this_team.units.iter_mut().find(|x|x.position==self.original).unwrap()
        // };

        {
            let terrain = &mut env.terrain;

            let foo = game.factions.relative_mut(team);

            if foo.that_team.units.is_set(target_cell) {
                let dir = self.original.dir_to(&target_cell);
                let check = target_cell.advance(dir);

                if env.terrain.is_set(check) || !world.get_game_cells().is_set(check) {
                    assert!(!env.fog.is_set(target_cell));
                    assert!(!env.fog.is_set(check));

                    foo.that_team.units.set_coord(target_cell, false);
                    destroyed_unit = Some(target_cell);
                } else if world.get_game_cells().is_set(check)
                    && !env.terrain.is_set(check)
                    && !foo.has_a_set(check)
                {
                    foo.that_team.units.set_coord(target_cell, false);
                    foo.that_team.units.set_coord(check, true);
                    e = PushInfo::PushedUnit;
                }
            } else if foo.this_team.units.is_set(target_cell) {
                let dir = self.original.dir_to(&target_cell);
                let check = target_cell.advance(dir);

                if world.get_game_cells().is_set(check)
                    && !env.terrain.is_set(check)
                    && !foo.has_a_set(check)
                {
                    foo.this_team.units.set_coord(target_cell, false);
                    foo.this_team.units.set_coord(check, true);

                    e = PushInfo::PushedUnit;
                }
            } else if terrain.land.is_set(target_cell) {
                let dir = self.original.dir_to(&target_cell);
                let kk = target_cell.advance(dir);

                terrain.land.set_coord(target_cell, false);

                if terrain.land.is_set(kk) {
                    terrain.land.set_coord(kk, false);
                    terrain.forest.set_coord(kk, true);

                    e = PushInfo::UpgradedLand;
                } else {
                    assert!(!terrain.is_set(kk));
                    terrain.land.set_coord(kk, true);

                    e = PushInfo::PushedLand;
                }
            }

            // if terrain.forest.is_set(target_cell) {
            //     let dir = this_unit.position.dir_to(&target_cell);
            //     let kk = target_cell.advance(dir);

            //     terrain.forest.set_coord(target_cell, false);

            //     if terrain.forest.is_set(kk) {
            //         terrain.forest.set_coord(kk, false);
            //         terrain.mountain.set_coord(kk, true);

            //         e = PushInfo::UpgradedLand;
            //     } else {
            //         assert!(!terrain.is_set(kk));
            //         terrain.forest.set_coord(kk, true);

            //         e = PushInfo::PushedLand;
            //     }
            // }
        }

        let powerup = if game.env.powerups.contains(&target_cell) {
            game.env.powerups.retain(|&a| a != target_cell);
            unreachable!()
            // if !this_unit.has_powerup {
            //     this_unit.has_powerup = true;
            //     PowerupAction::GotPowerup
            // } else {
            //     // powerup is discarded
            //     PowerupAction::DiscardedPowerup
            // }
        } else {
            PowerupAction::None
        };

        game.factions
            .relative_mut(team)
            .this_team
            .units
            .set_coord(self.original, false);
        game.factions
            .relative_mut(team)
            .this_team
            .units
            .set_coord(target_cell, true);

        MoveEffect {
            pushpull: e,
            powerup,
            destroyed_unit,
        }
    }
}

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

fn calculate_walls(position: Axial, state: &GameState, world: &board::MyWorld) -> SmallMesh {
    let env = &state.env;
    let mut walls = SmallMesh::new();

    for a in position.to_cube().range(2) {
        let a = a.to_axial();
        //TODO this is duplicated logic in selection function???
        let cc = env.terrain.is_set(a);
        if cc || (a != position && state.factions.has_a_set(a)) || !world.get_game_cells().is_set(a)
        {
            walls.add(a.sub(&position));
        }
    }

    walls
}
