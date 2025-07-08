use crate::{
    main_logic::{AnimationCommand, CommandSender},
    moves::{MoveType, SpokeInfo},
};

use super::*;

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct MoveEffect {
    pub height: u8,
    pub destroyed_unit: Option<(u8, Team)>,
}

#[derive(PartialEq, Eq, Default, Serialize, Deserialize, Clone, Copy, Debug)]

pub struct LighthouseMove {
    pub coord: Coordinate,
}


impl hex::HexDraw for LighthouseMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, radius: i8) -> Result<(), std::fmt::Error> {
        Axial::from_index(&self.coord).fmt(f, radius)
    }
}

impl LighthouseMove{
    pub fn possible_moves<'b>(
        state: &'b GameState,
        world: &'b board::MyWorld,
        team: Team,
        spoke_info: &'b SpokeInfo,
        allow_suicidal: bool,
    ) -> impl Iterator<Item = NormalMove> + use<'b> {
        world.land_as_vec.iter().filter_map(move |&index| {
            if let Some(f) = NormalMove::playable(state,Coordinate(index), team, world, spoke_info) {
                if !f.is_suicidal() || allow_suicidal {
                    Some(NormalMove {
                        coord: Coordinate(index),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}

//Represents playing a normal piece at the specified coordinate
//The tactical AI considers millions of these moves only.
//The tactical AI does not consider light house moves
#[derive(PartialEq, Eq, Default, Serialize, Deserialize, Clone, Copy, Debug)]

pub struct NormalMove {
    pub coord: Coordinate,
}

impl hex::HexDraw for NormalMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, radius: i8) -> Result<(), std::fmt::Error> {
        Axial::from_index(&self.coord).fmt(f, radius)
    }
}

impl NormalMove {
    pub fn new_pass() -> NormalMove {
        NormalMove {
            coord: Coordinate(hex::PASS_MOVE_INDEX),
        }
    }
    pub fn is_pass(&self) -> bool {
        self.coord.0 == hex::PASS_MOVE_INDEX
    }
    pub fn possible_moves<'b>(
        state: &'b GameState,
        world: &'b board::MyWorld,
        team: Team,
        spoke_info: &'b SpokeInfo,
        allow_suicidal: bool,
    ) -> impl Iterator<Item = NormalMove> + use<'b> {
        world.land_as_vec.iter().filter_map(move |&index| {
            if let Some(f) = NormalMove::playable(state,Coordinate(index), team, world, spoke_info) {
                if !f.is_suicidal() || allow_suicidal {
                    Some(NormalMove {
                        coord: Coordinate(index),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
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
                destroyed_unit: None,
                height: 0,
            };
        }

        //let env = &mut game.env;
        let target_cell = self.coord.0;

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
            destroyed_unit,
            height: stack_size as u8,
        }
    }

    pub async fn animate_move<'a>(
        &'a self,
        team: Team,
        state: &unit::GameStateTotal,
        world: &board::MyWorld,
        data: &mut CommandSender,
    ) -> &'a NormalMove {
        let aa = self;
        if self.is_pass() {
            return self;
        }
        assert!(
            world.get_game_cells().inner[aa.coord.0 as usize],
            "uhoh {:?}",
            world.format(&aa.coord)
        );

        let ff = state.tactical.bake_fog(&state.fog[team.index()]);

        let end_points = ff.factions.iter_end_points(world, aa.coord.0);

        let mut ss = state.clone();

        let mut stack = 0;
        for (i, (dis, rest)) in end_points.into_iter().enumerate() {
            let Some(e) = rest else {
                continue;
            };
            let team2 = e.piece.team;

            if team2 != team {
                continue;
            }

            let unit = Axial::from_index(&aa.coord)
                .add(hex::Cube::from_arr(hex::OFFSETS[i]).ax.mul(dis as i8));

            data.wait_animation(
                AnimationCommand::Movement {
                    unit,
                    end: Axial::from_index(&aa.coord),
                },
                team,
                &mut ss,
            )
            .await;

            stack += 1;
            match state.tactical.factions.get_cell_inner(aa.coord.0) {
                unit::GameCell::Piece(unit::Piece { .. }) => {
                    ss.tactical.factions.remove_inner(aa.coord.0);
                }
                unit::GameCell::Empty => {}
            }
            //TODO can_attack correct value?
            ss.tactical
                .factions
                .add_cell_inner(aa.coord.0, stack, team, true);
        }

        aa
    }
    pub fn playable(
        state:&GameState,
        index: Coordinate,
        team: Team,
        _world: &board::MyWorld,
        spoke_info: &SpokeInfo,
    ) -> Option<MoveType> {
        let index=index.0;
        if team == Team::Neutral {
            return None;
        }

        let num_attack = spoke_info.get_num_attack(index);

        if num_attack[team] == 0 {
            return None;
        }

        match state.factions.get_cell_inner(index) {
            &unit::GameCell::Piece(unit::Piece {
                height: stack_height,
                team: rest,
                ..
            }) => {
                let height = stack_height.to_num();
                debug_assert!(height > 0);
                let height = height as i64;

                if num_attack[team] > height {
                    if num_attack[team] < num_attack[!team] {
                        Some(MoveType::Suicidal)
                    } else {
                        if rest == team {
                            Some(MoveType::Reinforce)
                        } else {
                            Some(MoveType::Capture)
                        }
                    }
                } else {
                    None
                }
            }
            unit::GameCell::Empty => {
                if num_attack[team] < num_attack[!team] {
                    Some(MoveType::Suicidal)
                } else {
                    Some(MoveType::Fresh)
                }
            }
        }
    }

    pub fn generate_suicidal<'b>(
        state:&'b GameState,
        world: &'b board::MyWorld,
        team: Team,
        spoke_info: &'b SpokeInfo,
    ) -> impl Iterator<Item = Coordinate> + use<'b> {
        world.land_as_vec.iter().filter_map(move |&index| {
            if let Some(f) = NormalMove::playable(state,Coordinate(index), team, world, spoke_info) {
                if f.is_suicidal() {
                    Some(Coordinate(index))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}
