use super::*;
use crate::{movement::movement_mesh::SmallMesh, moves::*};

// pub struct CompleteMove {
//     pub original: GridCoord,
//     pub moveto: GridCoord,
//     pub target: GridCoord,
// }

pub struct ExtraPhase {
    pub original: GridCoord,
    pub moveto: GridCoord,
    pub target: GridCoord,
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

        if !meta.bomb.0.is_empty() {
            assert_eq!(unit, attackto);
            assert_eq!(unit.to_cube().dist(&moveto.to_cube()), 2);
            for a in meta.bomb.0.iter_mesh(unit) {
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

    pub fn apply(&self, team: ActiveTeam, game: &mut GameState) -> ExtraEffect {
        let original = self.original;
        let moveto = self.moveto;
        let target_cell = self.target;
        let mut bb = BombInfo(SmallMesh::new());
        if target_cell == original && original.to_cube().dist(&moveto.to_cube()) == 2 {
            //if false{
            bb = detonate_bomb(original, game);
        } else {
            if !game.env.land.is_coord_set(target_cell) {
                game.env.land.set_coord(target_cell, true)
            } else {
                // if !env.forest.is_coord_set(target_cell) {
                //     env.forest.set_coord(target_cell, true);
                // }
                unreachable!("WAT");
            }
        }

        let fog = uncover_fog(moveto, &mut game.env);

        ExtraEffect { fog, bomb: bb }
    }

    pub async fn animate(
        &self,
        team: ActiveTeam,
        state: &GameState,
        data: &mut ace::WorkerManager<'_>,
    ) -> &Self {
        let target = self.target;

        let terrain_type = if !state.env.land.is_coord_set(target) {
            animation::TerrainType::Grass
        } else {
            if !state.env.forest.is_coord_set(target) {
                animation::TerrainType::Mountain
            } else {
                unreachable!()
            }
        };

        let _ = data
            .wait_animation(
                animation::AnimationCommand::Terrain {
                    pos: target,
                    terrain_type,
                    dir: animation::AnimationDirection::Up,
                },
                team,
            )
            .await;
        self
    }
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct MoveEffect {
    pub pushpull: PushPullInfo,
    pub powerup: PowerupAction,
}

#[derive(Clone)]
pub struct MovePhase {
    pub original: GridCoord,
    pub moveto: GridCoord,
}
impl MovePhase {
    pub fn into_attack(self, target: GridCoord) -> ExtraPhase {
        ExtraPhase {
            original: self.original,
            moveto: self.moveto,
            target,
        }
    }
    pub async fn animate(
        &self,
        team: ActiveTeam,
        data: &mut ace::WorkerManager<'_>,
        state: &GameState,
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
        let mesh = state.generate_unit_possible_moves_inner2(&unit.position, unit.typ, team, None);

        let k = move_build::MovePhase {
            original: this_unit,
            moveto: target,
        };

        let info = {
            let this_unit = state.factions.get_unit(team, self.original);
            let target_cell = self.moveto;
            let mut e = PushPullInfo::None;
            match this_unit.typ {
                Type::Warrior { .. } => {
                    if state.env.land.is_coord_set(target_cell) {
                        e = PushPullInfo::PushedLand;
                    }
                }
                Type::Archer => {
                    if state.env.land.is_coord_set(target_cell) {
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
        };
        let this_unit = state.factions.get_unit(team, this_unit);

        let _ = data
            .wait_animation(
                animation::AnimationCommand::Movement {
                    unit: this_unit.clone(),
                    mesh,
                    walls,
                    end: target,
                    data: info,
                },
                team,
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

        // for a in effect.fog.0.iter_mesh(moveto) {
        //     assert!(!state.env.fog.is_coord_set(a));
        //     state.env.fog.set_coord(a, true);
        // }

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

    pub fn apply(&self, team: ActiveTeam, game: &mut GameState) -> MoveEffect {
        let env = &mut game.env;
        let this_unit = game.factions.get_unit_mut(team, self.original);
        let target_cell = self.moveto;
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

        let orig = this_unit.position;

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
pub enum PushPullInfo {
    PushedLand,
    PulledLand,
    None,
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct UndoInfo {
    pub move_effect: MoveEffect,
    pub extra_effect: ExtraEffect,
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
struct BombInfo(pub SmallMesh);

//returns a mesh where set bits indicate cells
//that were fog before this function was called,
//and were then unfogged.
fn detonate_bomb(original: GridCoord, game: &mut GameState) -> BombInfo {
    let mut mesh = SmallMesh::new();

    for a in original.to_cube().range(2).map(|a| a.to_axial()) {
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

        mesh.add(a.sub(&original));
    }

    for a in mesh.iter_mesh(GridCoord([0; 2])) {
        game.env.land.set_coord(original.add(a), true);
    }
    BombInfo(mesh)
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct ExtraEffect {
    pub fog: FogInfo,
    pub bomb: BombInfo,
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct FogInfo(pub SmallMesh);

//returns a mesh where set bits indicate cells
//that were fog before this function was called,
//and were then unfogged.
pub fn uncover_fog(og: GridCoord, env: &mut Environment) -> FogInfo {
    let mut mesh = SmallMesh::new();
    for a in og.to_cube().range(1) {
        if env.fog.is_coord_set(a.to_axial()) {
            mesh.add(a.to_axial().sub(&og));
        }
    }

    for a in mesh.iter_mesh(GridCoord([0; 2])) {
        env.fog.set_coord(og.add(a), false);
    }
    FogInfo(mesh)
}

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
