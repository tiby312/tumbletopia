use crate::{
    main_logic::{AnimationCommand, CommandSender},
    moves::{MoveType, SpokeInfo},
    unit::StackHeight,
};

use super::*;

#[derive(Copy, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct DestroyedUnit {
    pub height: StackHeight,
    pub team: Team,
    pub was_lighthouse: Option<Team>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum GenericMove<T, L> {
    Normal(T),
    Lighthouse(L),
}

impl hex::HexDraw for GenericMove<NormalMove, LighthouseMove> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, radius: i8) -> Result<(), std::fmt::Error> {
        match self {
            GenericMove::Normal(o) => o.fmt(f, radius),
            GenericMove::Lighthouse(o) => o.fmt(f, radius),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct NormalMoveEffect {
    pub destroyed_unit: Option<DestroyedUnit>,
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct LighthouseMoveEffect {
    nm: NormalMoveEffect,
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

impl LighthouseMove {
    pub fn possible_moves<'b>(
        state: &'b GameState,
        world: &'b board::MyWorld,
        team: Team,
        spoke_info: &'b SpokeInfo,
        allow_suicidal: bool,
    ) -> impl Iterator<Item = LighthouseMove> + use<'b> {
        //light house pieces should get added as neutral pieces to the game state
        //because they cannot attack.
        //The game state is used mainly to determine what moves the ai can play.
        //The ai does not normally care which team a lighhouse belongs to.
        //The only time it would care is in its evaluation function.
        //The eval function needs access to which pieces are lighthouses, but nothing else.
        //This simplifies a lot of the logic as far as adding and removing pieces
        //to the game state.
        //The bottom line is that lighhouses act as neutral pieces as far how they block
        //other pieces LOS as well as how they do not attack.

        //let world = state.darkness(world, team);

        //TODO optimize this
        let j: Vec<LighthouseMove> = world
            .land_as_vec
            .iter()
            .filter_map(move |&index| match state.factions.get_cell_inner(index) {
                unit::GameCell::Piece(_) => None,
                unit::GameCell::Empty => Some(LighthouseMove {
                    coord: Coordinate(index),
                }),
            })
            .collect();
        j.into_iter()
    }

    pub fn apply(
        &self,
        team: Team,
        game: &mut GameState,
        fog: &mesh::small_mesh::SmallMesh,
        world: &board::MyWorld,
        spoke_info: Option<&SpokeInfo>,
    ) -> LighthouseMoveEffect {
        assert_ne!(self.coord.0, hex::PASS_MOVE_INDEX);

        game.lighthouses
            .add_cell_inner(self.coord.0, StackHeight::Stack0, team);

        let nm = NormalMove {
            coord: self.coord,
            stack: StackHeight::Stack0,
        }
        .apply(Team::Neutral, game, fog, world, spoke_info);

        LighthouseMoveEffect { nm }
    }

    pub fn undo(&self, team: Team, effect: &LighthouseMoveEffect, state: &mut GameState) {
        assert_ne!(self.coord.0, hex::PASS_MOVE_INDEX);

        NormalMove {
            coord: self.coord,
            stack: StackHeight::Stack0,
        }
        .undo(team, &effect.nm, state);

        state.lighthouses.remove_inner(self.coord.0);
    }
}

//Represents playing a normal piece at the specified coordinate
//The tactical AI considers millions of these moves only.
//The tactical AI does not consider light house moves
#[derive(PartialEq, Eq, Default, Serialize, Deserialize, Clone, Copy, Debug)]

pub struct NormalMove {
    pub stack: StackHeight,
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
            stack: StackHeight::Stack0,
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
            if let Some(f) = NormalMove::playable(state, Coordinate(index), team, world, spoke_info)
            {
                if !f.is_suicidal() || allow_suicidal {
                    Some(NormalMove {
                        stack: Coordinate(index).determine_stack_height(
                            state,
                            world,
                            team,
                            Some(spoke_info),
                        ),
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
    pub fn undo(&self, _team_index: Team, effect: &NormalMoveEffect, state: &mut GameState) {
        let moveto = self.coord.0;

        if moveto == hex::PASS_MOVE_INDEX {
            return;
        }

        if let Some(dd) = effect.destroyed_unit {
            state.factions.add_cell_inner(moveto, dd.height, dd.team);

            if let Some(dd) = dd.was_lighthouse {
                state
                    .lighthouses
                    .add_cell_inner(moveto, StackHeight::Stack0, dd);
            }
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
    ) -> NormalMoveEffect {
        //this is a pass
        if self.coord.0 == hex::PASS_MOVE_INDEX {
            return NormalMoveEffect {
                destroyed_unit: None,
            };
        }

        //let env = &mut game.env;
        let target_cell = self.coord.0;

        let destroyed_unit = match game.factions.get_cell_inner(target_cell) {
            &unit::GameCell::Piece(unit::Piece { height, team, .. }) => {
                let was_lighthouse = {
                    match game.lighthouses.get_cell_inner(target_cell) {
                        unit::GameCell::Piece(pp) => Some(pp.team),
                        unit::GameCell::Empty => None,
                    }
                };
                Some(DestroyedUnit {
                    height,
                    team,
                    was_lighthouse,
                })
            }
            unit::GameCell::Empty => None,
        };

        game.factions.remove_inner(target_cell);
        game.factions.add_cell_inner(target_cell, self.stack, team);

        NormalMoveEffect { destroyed_unit }
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
                .add_cell_inner(aa.coord.0, StackHeight::from_num(stack), team);
        }

        aa
    }
    pub fn playable(
        state: &GameState,
        index: Coordinate,
        team: Team,
        _world: &board::MyWorld,
        spoke_info: &SpokeInfo,
    ) -> Option<MoveType> {
        let index = index.0;
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
                //debug_assert!(height > 0);
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
        state: &'b GameState,
        world: &'b board::MyWorld,
        team: Team,
        spoke_info: &'b SpokeInfo,
    ) -> impl Iterator<Item = Coordinate> + use<'b> {
        world.land_as_vec.iter().filter_map(move |&index| {
            if let Some(f) = NormalMove::playable(state, Coordinate(index), team, world, spoke_info)
            {
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
