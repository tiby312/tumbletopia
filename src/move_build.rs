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
            assert!(!state.env.fog.is_coord_set(a));
            state.env.fog.set_coord(a, true);
        }

        if let Some(m) = &meta.bomb {
            assert_eq!(unit, attackto);
            assert_eq!(unit.to_cube().dist(&moveto.to_cube()), 2);
            for a in m.0.iter_mesh(unit) {
                assert!(state.env.land.is_coord_set(a));
                state.env.land.set_coord(a, false);
            }
        } else {
            if state.env.forest.is_coord_set(attackto) {
                state.env.forest.set_coord(attackto, false);
            } else if state.env.land.is_coord_set(attackto) {
                state.env.land.set_coord(attackto, false);
            } else {
                unreachable!();
            }
        }

        MovePhase {
            original: self.original,
            moveto: self.moveto,
        }
    }
    //returns a mesh where set bits indicate cells
    //that were fog before this function was called,
    //and were then unfogged.
    fn compute_bomb(&self, game: &GameState) -> Option<BombInfo> {
        if self.target != self.original || self.original.to_cube().dist(&self.moveto.to_cube()) != 2
        {
            return None;
        }

        let mut mesh = SmallMesh::new();

        for a in self.original.to_cube().range(2).map(|a| a.to_axial()) {
            if !game.world.get_game_cells().is_coord_set(a) {
                continue;
            }

            if game.factions.contains(a) {
                continue;
            }

            if game.env.land.is_coord_set(a) {
                continue;
            }

            if game.env.fog.is_coord_set(a) {
                continue;
            }

            mesh.add(a.sub(&self.original));
        }

        Some(BombInfo(mesh))
    }

    pub fn apply(&self, team: ActiveTeam, game: &mut GameState) -> ExtraEffect {
        let original = self.original;
        let moveto = self.moveto;
        let target_cell = self.target;

        let bb = if let Some(bb) = self.compute_bomb(game) {
            bb.apply(original, game);
            Some(bb)
        } else {
            if !game.env.land.is_coord_set(target_cell) {
                game.env.land.set_coord(target_cell, true)
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

        let fog = compute_fog(moveto, &mut game.env);
        fog.apply(moveto, &mut game.env);

        ExtraEffect { fog, bomb: bb }
    }

    pub async fn animate(
        &self,
        team: ActiveTeam,
        state: &GameState,
        data: &mut ace::WorkerManager,
    ) -> &Self {
        let target = self.target;

        let mut gg = state.clone();

        if let Some(bb) = self.compute_bomb(state) {
            let k = self.original.to_cube();
            for a in std::iter::once(k).chain(k.ring(1)).chain(k.ring(2)) {
                if bb.0.is_set(a.sub(self.original.to_cube()).to_axial()) {
                    gg = data
                        .wait_animation(
                            animation::AnimationCommand::Terrain {
                                pos: a.to_axial(),
                                terrain_type: animation::TerrainType::Grass,
                                dir: animation::AnimationDirection::Up,
                            },
                            team,
                            gg,
                        )
                        .await;
                    gg.env.land.set_coord(a.to_axial(), true);
                }
            }
        } else {
            gg = data
                .wait_animation(
                    animation::AnimationCommand::Terrain {
                        pos: target,
                        terrain_type: animation::TerrainType::Grass,
                        dir: animation::AnimationDirection::Up,
                    },
                    team,
                    gg,
                )
                .await;

            gg.env.land.set_coord(target, true);
        }

        let fog = compute_fog(self.moveto, &state.env);

        //let mut game = state.clone();
        for a in fog.0.iter_mesh(self.moveto) {
            gg.env.fog.set_coord(a, false);
            // Change mesh
            gg = data
                .wait_animation(
                    animation::AnimationCommand::Terrain {
                        pos: a,
                        terrain_type: animation::TerrainType::Fog,
                        dir: animation::AnimationDirection::Down,
                    },
                    team,
                    gg,
                )
                .await;
        }

        self
    }
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct MoveEffect {
    pub pushpull: PushInfo,
    pub powerup: PowerupAction,
}
impl MoveEffect {
    pub fn combine(self, extra_effect: ExtraEffect) -> CombinedEffect {
        CombinedEffect {
            move_effect: self,
            extra_effect,
        }
    }
}

#[derive(Clone)]
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
        data: &mut ace::WorkerManager,
    ) -> &Self {
        let this_unit = self.original;
        let target = self.moveto;
        let walls = calculate_walls(this_unit, state);

        let unit = state
            .factions
            .relative(team)
            .this_team
            .find_slow(&this_unit)
            .unwrap();
        let mesh = state.generate_possible_moves_movement(&unit.position, unit.typ, team);

        let info = {
            let this_unit = state.factions.get_unit(team, self.original);
            let target_cell = self.moveto;
            let mut e = PushInfo::None;
            match this_unit.typ {
                Type::Warrior { .. } => {
                    if state.env.land.is_coord_set(target_cell) {
                        e = PushInfo::PushedLand;
                    }
                }
                Type::Archer => {
                    if state.env.land.is_coord_set(target_cell) {
                        e = PushInfo::PushedLand;
                    }
                }
            }

            e
        };
        let this_unit = state.factions.get_unit(team, this_unit);

        let mut ss = state.clone();
        ss.factions
            .relative_mut(team)
            .this_team
            .units
            .retain(|k| k.position != unit.position);

        let end = target;
        match info {
            PushInfo::PushedLand => {
                let dir = unit.position.dir_to(&end);
                let k = unit.position.advance(dir);
                assert!(ss.env.land.is_coord_set(k));
                ss.env.land.set_coord(k, false);
            }

            PushInfo::None => {}
        }

        let _ = data
            .wait_animation(
                animation::AnimationCommand::Movement {
                    unit: this_unit.clone(),
                    mesh,
                    walls,
                    end,
                    data: info,
                },
                team,
                ss,
            )
            .await;
        self
    }

    pub fn undo(&self, team_index: ActiveTeam, effect: &MoveEffect, state: &mut GameState) {
        let moveto = self.moveto;
        let unit = self.original;
        let k = state
            .factions
            .relative_mut(team_index)
            .this_team
            .find_slow_mut(&moveto)
            .unwrap();

        match effect.pushpull {
            PushInfo::PushedLand => {
                let dir = unit.dir_to(&moveto);
                let t3 = moveto.advance(dir);
                assert!(state.env.land.is_coord_set(t3));
                state.env.land.set_coord(t3, false);
                assert!(!state.env.land.is_coord_set(moveto));
                state.env.land.set_coord(moveto, true);
            }

            PushInfo::None => {}
        }
        k.position = unit;
    }

    pub fn apply(&self, team: ActiveTeam, game: &mut GameState) -> MoveEffect {
        let env = &mut game.env;
        let this_unit = game.factions.get_unit_mut(team, self.original);
        let target_cell = self.moveto;
        let mut e = PushInfo::None;

        match this_unit.typ {
            Type::Warrior { .. } => {
                if env.land.is_coord_set(target_cell) {
                    let dir = this_unit.position.dir_to(&target_cell);

                    env.land.set_coord(target_cell, false);

                    let kk = target_cell.advance(dir);

                    env.land.set_coord(kk, true);

                    e = PushInfo::PushedLand;
                }
            }
            Type::Archer => {
                unreachable!();
            }
        }

        let powerup = if game.env.powerups.contains(&target_cell) {
            game.env.powerups.retain(|&a| a != target_cell);
            if !this_unit.has_powerup {
                this_unit.has_powerup = true;
                PowerupAction::GotPowerup
            } else {
                // powerup is discarded
                PowerupAction::DiscardedPowerup
            }
        } else {
            PowerupAction::None
        };

        this_unit.position = target_cell;

        MoveEffect {
            pushpull: e,
            powerup,
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
    PushedLand,
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
            game.env.land.set_coord(original.add(a), true);
        }
    }
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct ExtraEffect {
    pub fog: FogInfo,
    pub bomb: Option<BombInfo>,
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
        if env.fog.is_coord_set(a.to_axial()) {
            mesh.add(a.to_axial().sub(&og));
        }
    }
    FogInfo(mesh)
}

fn calculate_walls(position: Axial, state: &GameState) -> SmallMesh {
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
