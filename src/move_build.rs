use super::*;
use crate::{movement::movement_mesh::SmallMesh, moves::*};

pub struct MovePhase1 {
    pub unit: GridCoord,
    pub target: GridCoord,
    pub team: ActiveTeam,
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
    pub fn execute(self, game: &mut GameState) -> (PartialMoveSigl, PushPullInfo, PowerupAction) {
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

        (
            PartialMoveSigl {
                unit: orig,
                moveto: target_cell,
            },
            e,
            powerup,
        )
    }
}

pub fn apply_extra_move(
    original: GridCoord,
    moveto: GridCoord,
    target_cell: GridCoord,
    game: &mut GameState,
) -> (PartialMoveSigl, MetaInfo) {
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

    (
        PartialMoveSigl {
            unit: moveto,
            moveto: target_cell,
        },
        MetaInfo { fog, bomb: bb },
    )
}

pub fn undo_movement(
    team_index: ActiveTeam,
    unit: GridCoord,
    moveto: GridCoord,
    effect: &PushPullInfo,
    state: &mut GameState,
) {
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

    match effect {
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

pub enum PowerupAction {
    GotPowerup,
    DiscardedPowerup,
    None,
}

pub fn undo_extra(
    team_index: ActiveTeam,
    unit: GridCoord,
    moveto: GridCoord,
    attackto: GridCoord,
    meta: &MetaInfo,
    state: &mut GameState,
) {
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
}

#[derive(PartialOrd, Ord, Clone, Copy, Eq, PartialEq, Debug)]
pub enum PushPullInfo {
    PushedLand,
    PulledLand,
    None,
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct UndoInfo {
    pub pushpull: PushPullInfo,
    pub meta: MetaInfo,
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct BombInfo(pub SmallMesh);

//returns a mesh where set bits indicate cells
//that were fog before this function was called,
//and were then unfogged.
pub fn detonate_bomb(original: GridCoord, game: &mut GameState) -> BombInfo {
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
pub struct MetaInfo {
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
