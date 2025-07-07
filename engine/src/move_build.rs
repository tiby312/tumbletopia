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

impl Coordinate {
    

    pub fn undo(&self, _team_index: Team, effect: &MoveEffect, state: &mut GameState) {
        let moveto = self.0;

        if moveto == hex::PASS_MOVE_INDEX {
            return;
        }

        if let Some((fooo, typ)) = effect.destroyed_unit {
            state.factions.add_cell_inner(moveto, fooo, typ);
        } else {
            state.factions.remove_inner(moveto)
        };

    }

    pub fn apply_darkness2(
        &self,
        team: Team,
        game: &mut GameState,
        fog: &mesh::small_mesh::SmallMesh,
        world: &board::MyWorld,
        spoke_info: Option<&SpokeInfo>,
    ) -> MoveEffect {
        assert!(self.0 != PASS_MOVE_INDEX);

        let darkness = game.darkness(world, team);
        let playable = game.convert_to_playable(world, team);
        console_dbg!("Hello");
        if darkness.inner[self.0] {
            //find teamates that can attempt to help
            //for each of these teamates, determine how far they can go.
            let mut num_attacking = 0;
            for a in hex::OFFSETS {
                console_dbg!("what2");
                let ax = Axial::from_index(&self.0).to_cube();

                let mut potential_reinforcer = None;
                for k in ax.ray_from_vector(Cube::from_arr(a)) {
                    if !world.land.is_set(k.ax) {
                        break;
                    }

                    match playable.factions.get_cell(k.ax) {
                        &unit::GameCell::Piece(unit::Piece {
                            
                            team: fa,
                            ..
                        }) => {
                            if fa == team {
                                potential_reinforcer = Some(k);
                                //we have found a team mate that might be able to reach us.
                            }

                            break;
                        }
                        unit::GameCell::Empty => {}
                    }
                }

                console_dbg!("what");
                if let Some(found) = potential_reinforcer {
                    //slowly iterate towards the center, stopping at any obstacle.
                    for k in found.ray_from_vector(
                        Cube::from_arr(a)
                            .rotate_60_left()
                            .rotate_60_left()
                            .rotate_60_left(),
                    ) {
                        match game.factions.get_cell(k.ax) {
                            &unit::GameCell::Piece(unit::Piece {
                                team: fa,
                                ..
                            }) => {
                                if fa != team {
                                    let spot = k.add(Cube::from_arr(a));

                                    assert!(spot.ax != PASS_MOVE);
                                    game.factions.remove(spot.ax);
                                    game.factions.add_cell(spot.ax, 1 as u8, team);
                                    break;
                                }
                            }
                            unit::GameCell::Empty => {}
                        }

                        if k.ax == ax.ax {
                            //team mate can make it
                            num_attacking += 1;
                            break;
                        }
                    }
                }
                console_dbg!("what");
                match game.factions.get_cell_inner(self.0) {
                    unit::GameCell::Piece(unit::Piece {
                        height: stack_height,
                        team: v,
                        ..
                    }) => {
                        if num_attacking > stack_height.to_num() {
                            game.factions.remove_inner(self.0);
                            game.factions
                                .add_cell_inner(self.0, num_attacking as u8, team);
                        }
                    }
                    unit::GameCell::Empty => {
                        if num_attacking > 0 {
                            game.factions.remove_inner(self.0);
                            game.factions
                                .add_cell_inner(self.0, num_attacking as u8, team);
                        }
                    }
                }

                console_dbg!("what");
            }
            MoveEffect {
                pushpull: PushInfo::None,
                powerup: PowerupAction::None,
                destroyed_unit: None,
                height: 0 as u8,
            }
        } else {
            self.apply(team, game, fog, world, spoke_info)
        }

        // if move is into darkness {
        //     from the point selected, trace outward in dark mask.
        //     if we encounter a friend unit, then tracebackward without dark mask
        //     if we get back to original point, great. otherwise we have hit an obstacle.
        // }
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
        if self.0 == hex::PASS_MOVE_INDEX {
            return MoveEffect {
                pushpull: PushInfo::None,
                powerup: PowerupAction::None,
                destroyed_unit: None,
                height: 0,
            };
        }

        //let env = &mut game.env;
        let target_cell = self.0;
        let e = PushInfo::None;

        let stack_size = if let Some(sp) = spoke_info {
            sp.data[self.0].num_attack[team]
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
            .add_cell_inner(target_cell, stack_size as u8, team);

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
