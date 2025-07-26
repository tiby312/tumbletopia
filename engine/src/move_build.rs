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
    pub lighthouse_was_removed: Option<Team>,
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
    nm: Option<NormalMoveEffect>,
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
    // pub fn playable(
    //     state: &GameState,
    //     index: Coordinate,
    //     team: Team,
    //     world: &board::MyWorld,
    //     spoke_info: &SpokeInfo,
    // ) -> Option<MoveType> {
    //     let index = index.0;
    //     if team == Team::Neutral {
    //         return None;
    //     }

    //     let mut num_friendly_lighthouses=0;
    //     //TODO optimize??? (Not sure if it worth it. ai shouldnt care about this)
    //     for (i, (_, rest)) in state.factions.iter_end_points(world, index).enumerate() {
    //         if let Some(e) = rest {
    //             match state.lighthouses.get_cell_inner(i){
    //                 unit::GameCell::Piece(e) => if e.team==team{
    //                     num_friendly_lighthouses+=1;
    //                 },
    //                 unit::GameCell::Empty => todo!(),
    //             }
    //         }
    //     }

    //     let num_attack = spoke_info.get_num_attack(index);

    //     if num_attack[team] == 0 {
    //         return None;
    //     }

    //     match state.factions.get_cell_inner(index) {
    //         &unit::GameCell::Piece(unit::Piece {
    //             height: stack_height,
    //             team: rest,
    //             ..
    //         }) => {
    //             let height = stack_height.to_num();
    //             //debug_assert!(height > 0);
    //             let height = height as i64;

    //             if num_attack[team] > height {
    //                 if num_attack[team] < num_attack[!team] {
    //                     Some(MoveType::Suicidal)
    //                 } else {
    //                     if rest == team {
    //                         Some(MoveType::Reinforce)
    //                     } else {
    //                         Some(MoveType::Capture)
    //                     }
    //                 }
    //             } else {
    //                 None
    //             }
    //         }
    //         unit::GameCell::Empty => {
    //             if num_attack[team] < num_attack[!team] {
    //                 Some(MoveType::Suicidal)
    //             } else {
    //                 Some(MoveType::Fresh)
    //             }
    //         }
    //     }
    // }

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

        //First replace all lighthouses as neutral pieces.
        //Then find all playable normal moves.

        let mut game = state.clone();

        //Forbid making a lighthouse from a lighthouse
        for &a in world.land_as_vec.iter() {
            match game.factions.get_cell_inner(a) {
                unit::GameCell::Piece(f) => {
                    if f.team == team && f.has_lighthouse {
                        game.factions.remove_inner(a);
                        game.factions
                            .add_cell_inner(a, StackHeight::Stack6, Team::Neutral, false);
                    }
                }
                unit::GameCell::Empty => {}
            }
        }

        let state = &game;

        //TODO inefficient
        let sp = SpokeInfo::new(&state, world);
        let v: Vec<_> = NormalMove::possible_moves(&state, world, team, &sp, allow_suicidal)
            .map(|x| LighthouseMove { coord: x.coord })
            .collect();

        v.into_iter()

        // //TODO optimize this
        // let j: Vec<LighthouseMove> = world
        //     .land_as_vec
        //     .iter()
        //     .filter_map(move |&index| {

        //             match state.lighthouses.get_cell_inner(index) {
        //             unit::GameCell::Piece(_) => None,
        //             unit::GameCell::Empty => Some(LighthouseMove {
        //                 coord: Coordinate(index),
        //             }),
        //         }},
        //     )
        //     .collect();
        // j.into_iter()
    }

    pub fn apply(
        &self,
        team: Team,
        game: &mut GameState,
        //fog: &mesh::small_mesh::SmallMesh,
        world: &board::MyWorld,
        spoke_info: Option<&SpokeInfo>,
    ) -> LighthouseMoveEffect {
        assert_ne!(self.coord.0, hex::PASS_MOVE_INDEX);

        let nm = match game.factions.get_cell_inner(self.coord.0) {
            unit::GameCell::Piece(_) => None,
            unit::GameCell::Empty => {
                let nm = NormalMove {
                    coord: self.coord,
                    stack: StackHeight::Stack0,
                }
                .apply(team, game, world, spoke_info);

                Some(nm)
            }
        };

        game.factions
            .add_cell_inner(self.coord.0, StackHeight::Stack0, team, true);

        LighthouseMoveEffect { nm }
    }

    pub fn undo(&self, effect: &LighthouseMoveEffect, state: &mut GameState) {
        assert_ne!(self.coord.0, hex::PASS_MOVE_INDEX);

        if let Some(fe) = &effect.nm {
            NormalMove {
                coord: self.coord,
                stack: StackHeight::Stack0,
            }
            .undo(fe, state);

            // if let Some(k)=fe.destroyed_unit{
            //     if let Some(ff)=k.lighthouse_was_removed{
            //         match &mut state.factions.cells[self.coord.0]{
            //             unit::GameCell::Piece(o) => o.has_lighthouse=true,
            //             unit::GameCell::Empty => {},
            //         }

            //     }

            // }
            // match &mut state.factions.cells[self.coord.0]{
            //     unit::GameCell::Piece(o) => o.has_lighthouse=false,
            //     unit::GameCell::Empty => {},
            // }
        }

        // match state.factions.cells[self.coord.0]{

        // }

        //state.lighthouses.remove_inner(self.coord.0);
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
    pub fn undo(&self, effect: &NormalMoveEffect, state: &mut GameState) {
        let moveto = self.coord.0;

        if moveto == hex::PASS_MOVE_INDEX {
            return;
        }

        if let Some(dd) = effect.destroyed_unit {
            state.factions.add_cell_inner(
                moveto,
                dd.height,
                dd.team,
                dd.lighthouse_was_removed.is_some(),
            );
        } else {
            state.factions.remove_inner(moveto)
        };
    }

    pub fn apply(
        &self,
        team: Team,
        game: &mut GameState,
        //fog: &mesh::small_mesh::SmallMesh,
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
            &unit::GameCell::Piece(pp) => {
                let lighthouse_was_removed = if pp.has_lighthouse {
                    if team != pp.team { Some(pp.team) } else { None }
                } else {
                    None
                };

                Some(DestroyedUnit {
                    height: pp.height,
                    team: pp.team,
                    lighthouse_was_removed,
                })
            }
            unit::GameCell::Empty => None,
        };

        let has_lighthouse = match game.factions.get_cell_inner(target_cell) {
            unit::GameCell::Piece(o) => {
                if o.has_lighthouse {
                    if let Some(d) = destroyed_unit {
                        if d.lighthouse_was_removed.is_some() {
                            false
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            unit::GameCell::Empty => false,
        };

        game.factions.remove_inner(target_cell);
        game.factions
            .add_cell_inner(target_cell, self.stack, team, has_lighthouse);

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

        //let ff = state.tactical.bake_fog(&state.fog[team.index()]);
        let ff = &state.tactical;

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
            ss.tactical.factions.add_cell_inner(
                aa.coord.0,
                StackHeight::from_num(stack),
                team,
                false,
            );
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

    //TODO use an iterator adaptor instead
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
