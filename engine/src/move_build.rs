use gloo_console::console_dbg;
use hex::{Cube, PASS_MOVE, PASS_MOVE_INDEX};

use crate::moves::SpokeInfo;

use super::*;

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct MoveEffect {
    pushpull: PushInfo,
    powerup: PowerupAction,
    pub height: u8,
    pub destroyed_unit: Option<(u8, Team)>,
}

impl GameAxial for Axial {
    fn get(&self) -> &Axial {
        self
    }
}
pub trait GameAxial {
    fn get(&self) -> &Axial;
}



//Represents playing a normal piece at the specified coordinate
//The tactical AI considers millions of these moves only.
//The tactical AI does not consider light house moves
#[derive(PartialEq, Eq, Default, Serialize, Deserialize, Clone, Copy, Debug)]

pub struct NormalMove {
    pub coord: Coordinate,
}
impl NormalMove{
    pub fn new_pass()->NormalMove{
        NormalMove{coord:Coordinate(hex::PASS_MOVE_INDEX)}
    }
    pub fn is_pass(&self)->bool{

        self.coord.0==hex::PASS_MOVE_INDEX
    }
}

impl hex::HexDraw for NormalMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, radius: i8) -> Result<(), std::fmt::Error> {
        Axial::from_index(&self.coord).fmt(f, radius)
    }
}

impl NormalMove {
    pub fn undo(&self, _team_index: Team, effect: &MoveEffect, state: &mut GameState) {
        let moveto = self.coord.0;

        if moveto == hex::PASS_MOVE_INDEX {
            return;
        }

        if let Some((fooo, typ)) = effect.destroyed_unit {
            state.factions.add_cell_inner(moveto, fooo, typ, true);
        } else {
            state.factions.remove_inner(moveto)
        };
    }

    pub fn apply(
        &self,
        team: Team,
        game: &mut GameState,
        fog: &mesh::small_mesh::SmallMesh,
        world: &board::MyWorld,
        spoke_info: Option<&SpokeInfo>,
    ) -> MoveEffect {
        //this is a pass
        if self.coord.0 == hex::PASS_MOVE_INDEX {
            return MoveEffect {
                pushpull: PushInfo::None,
                powerup: PowerupAction::None,
                destroyed_unit: None,
                height: 0,
            };
        }

        //let env = &mut game.env;
        let target_cell = self.coord.0;
        let e = PushInfo::None;

        let stack_size = if let Some(sp) = spoke_info {
            sp.data[self.coord.0].num_attack[team]
        } else {
            let mut stack_size = 0;

            for (_, rest) in game
                .bake_fog(fog)
                .factions
                .iter_end_points(world, target_cell)
            {
                if let Some(e) = rest {
                    if e.piece.team == team {
                        stack_size += 1;
                    }
                }
            }
            stack_size
        };

        // for (i, h) in hex::OFFSETS.into_iter().enumerate() {
        //     for k in target_cell
        //         .to_cube()
        //         .ray_from_vector(hex::Cube::from_arr(h))
        //     {
        //         let k = k.to_axial();
        //         if !world.get_game_cells().is_set(k) {
        //             break;
        //         }

        //         if let Some((vv, team2)) = game.factions.cells.get_cell(k) {
        //             if team2 == team {
        //                 stack_size += 1;
        //             }
        //             break;
        //         }
        //     }
        // }

        //console_dbg!("Adding stacksize=", stack_size);

        let destroyed_unit = match game.factions.get_cell_inner(target_cell) {
            &unit::GameCell::Piece(unit::Piece {
                height: stack_height,
                team: v,
                ..
            }) => Some((stack_height.to_num() as u8, v)),
            unit::GameCell::Empty => None,
        };

        game.factions.remove_inner(target_cell);
        game.factions
            .add_cell_inner(target_cell, stack_size as u8, team, true);

        MoveEffect {
            pushpull: e,
            powerup: PowerupAction::None,
            destroyed_unit,
            height: stack_size as u8,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub enum PowerupAction {
    GotPowerup,
    DiscardedPowerup,
    None,
}

#[derive(Serialize, Deserialize, PartialOrd, Ord, Clone, Copy, Eq, PartialEq, Debug)]
pub enum PushInfo {
    UpgradedLand,
    PushedLand,
    PushedUnit,
    None,
}
