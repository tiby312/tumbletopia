use self::selection::JustMoveLog;

use super::*;
pub mod ai;
pub mod selection;
use crate::{CellSelection, GameState, UnitData};

use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};

pub struct GameWrap<T> {
    pub game: GameState,
    pub team: ActiveTeam,
    pub data: T,
}
impl<T> GameWrap<T> {
    pub fn with_data<K>(self, a: K) -> GameWrap<K> {
        GameWrap {
            game: self.game,
            team: self.team,
            data: a,
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Animate(animation::AnimationCommand),
    GetMouseInputSelection {
        selection: CellSelection,
        grey: bool,
    },
    GetMouseInputNoSelect,
    Nothing,
    ShowUndo,
    HideUndo,
    Popup(String),
    Poke,
}

#[derive(Debug)]
pub enum Response {
    MouseWithSelection(CellSelection, MouseEvent<Axial>),
    Mouse(MouseEvent<Axial>),
    AnimationFinish,
    Ack,
}

#[derive(Debug)]
pub enum MouseEvent<T> {
    Normal(T),
    Undo,
}

pub struct WorkerManager {
    pub sender: Sender<GameWrap<Command>>,
    pub receiver: Receiver<GameWrap<Response>>,
}

impl WorkerManager {
    pub async fn wait_animation(
        &mut self,
        animation: animation::AnimationCommand,
        team: ActiveTeam,
        game: &mut GameState,
    ) {
        let data = self
            .send_command(team, game, Command::Animate(animation))
            .await;

        let Response::AnimationFinish = data else {
            unreachable!();
        };
    }

    async fn get_mouse_with_mesh(
        &mut self,
        cell: &mut CellSelection,
        team: ActiveTeam,
        game: &mut GameState,
        grey: bool,
    ) -> MouseEvent<Axial> {
        let selection = std::mem::take(cell);

        let b = self
            .send_command(
                team,
                game,
                Command::GetMouseInputSelection { selection, grey },
            )
            .await;

        let Response::MouseWithSelection(mut cell2, o) = b else {
            unreachable!();
        };

        std::mem::swap(&mut cell2, cell);

        o
    }

    async fn get_mouse(&mut self, team: ActiveTeam, game: &mut GameState) -> MouseEvent<Axial> {
        let b = self
            .send_command(team, game, Command::GetMouseInputNoSelect)
            .await;

        let Response::Mouse(o) = b else {
            unreachable!();
        };

        o
    }

    // async fn poke(&mut self, team: ActiveTeam, game: GameState) {
    //     self.sender
    //         .send(GameWrap {
    //             game,
    //             data: Command::Poke,
    //             team,
    //         })
    //         .await
    //         .unwrap();

    //     let GameWrapResponse { game: _gg, data } = self.receiver.next().await.unwrap();

    //     let Response::Ack = data else {
    //         unreachable!();
    //     };
    // }

    async fn send_popup(&mut self, str: &str, team: ActiveTeam, game: &mut GameState) {
        self.send_command(team, game, Command::Popup(str.into()))
            .await;

        let GameWrap { data, .. } = self.receiver.next().await.unwrap();

        let Response::Ack = data else {
            unreachable!();
        };
    }

    // TODO do this
    // async fn send_command_mut(
    //     &mut self,
    //     team: ActiveTeam,
    //     game: &mut GameState,
    //     co: Command,
    // ) -> Response {

    //     self.sender
    //         .send(GameWrap {
    //             game,
    //             data: co,
    //             team,
    //         })
    //         .await
    //         .unwrap();

    //     let GameWrapResponse { game, data } = self.receiver.next().await.unwrap();

    //     (game, data)
    // }

    //TODO use
    async fn send_command(
        &mut self,
        team: ActiveTeam,
        game1: &mut GameState,
        co: Command,
    ) -> Response {
        let game2 = std::mem::take(game1);
        self.sender
            .send(GameWrap {
                game: game2,
                data: co,
                team,
            })
            .await
            .unwrap();

        let GameWrap { mut game, data, .. } = self.receiver.next().await.unwrap();

        std::mem::swap(&mut game, game1);

        data
    }
}

pub struct SelectType {
    coord: Axial,
    team: ActiveTeam,
}

pub enum LoopRes<T> {
    EndTurn((moves::ActualMove, move_build::CombinedEffect)),
    Deselect,
    Undo,
    Select(T),
}

pub async fn reselect_loop(
    doop: &mut WorkerManager,
    game: &mut GameState,
    world: &board::MyWorld,
    team: ActiveTeam,
    have_moved: &mut Option<selection::HaveMoved>,
    mut selected_unit: SelectType,
) -> LoopRes<SelectType> {
    console_dbg!(have_moved.is_some());
    //At this point we know a friendly unit is currently selected.

    let unwrapped_selected_unit = selected_unit.coord;

    let unit = game
        .factions
        .relative(selected_unit.team)
        .this_team
        .find_slow(&unwrapped_selected_unit)
        .unwrap()
        .clone();

    let grey = if selected_unit.team == team {
        //If we are in the middle of a extra attack move, make sure
        //no other friendly unit is selectable until we finish moving the
        //the unit that has been partially moved.
        if let Some(e) = have_moved {
            e.the_move.moveto != selected_unit.coord
        } else {
            false
        }
    } else {
        true
    };

    let cca = if let Some(have_moved) = have_moved {
        (selected_unit.coord == have_moved.the_move.moveto).then(|| {
            game.generate_possible_moves_extra(
                world,
                &have_moved.the_move,
                unit.typ,
                selected_unit.team,
            )
        })
    } else {
        None
    };

    let cca = cca.unwrap_or_else(|| {
        game.generate_possible_moves_movement(world, &unit.position, unit.typ, selected_unit.team)
    });

    let mut cell = CellSelection::MoveSelection(unwrapped_selected_unit, cca, have_moved.clone());

    let pototo = doop
        .get_mouse_with_mesh(&mut cell, selected_unit.team, game, grey)
        .await;

    let mouse_world = match pototo {
        MouseEvent::Normal(t) => t,
        MouseEvent::Undo => {
            //End the turn. Ok because we are not int he middle of anything.
            //return LoopRes::EndTurn;
            //unreachable!();
            return LoopRes::Undo;
        }
    };
    let target_cell = mouse_world;

    //This is the cell the user selected from the pool of available moves for the unit
    let CellSelection::MoveSelection(_, ss, _) = cell else {
        unreachable!()
    };

    //If we just clicked on ourselves, just deselect.
    if target_cell == unwrapped_selected_unit {
        return LoopRes::Deselect;
    }

    let contains = ss.is_set(target_cell.sub(&unwrapped_selected_unit));

    //If we select a friendly unit quick swap
    if let Some(target) = game
        .factions
        .relative(selected_unit.team)
        .this_team
        .find_slow(&target_cell)
    {
        let tt = target.position;
        if !contains {
            //it should be impossible for a unit to move onto a friendly
            //assert!(!contains);
            selected_unit.coord = tt;
            return LoopRes::Select(selected_unit);
        }
    }

    //If we select an enemy unit quick swap
    if let Some(target) = game
        .factions
        .relative(selected_unit.team)
        .that_team
        .find_slow(&target_cell)
    {
        let tt = target.position;
        if selected_unit.team != team || !contains {
            //If we select an enemy unit thats outside of our units range.
            selected_unit.coord = tt;
            selected_unit.team = selected_unit.team.not();
            return LoopRes::Select(selected_unit);
        }
    }

    //If we selected an empty space, deselect.
    if !contains {
        return LoopRes::Deselect;
    }

    //If we are trying to move an enemy piece, deselect.
    if selected_unit.team != team {
        return LoopRes::Deselect;
    }

    // If we are trying to move a piece while in the middle of another
    // piece move, deselect.
    if let Some(e) = have_moved {
        if unwrapped_selected_unit != e.the_move.moveto {
            return LoopRes::Deselect;
        }
    }

    //At this point all re-selecting of units based off of the input has occured.
    //We definately want to act on the action the user took on the selected unit.

    if let Some(e) = have_moved.take() {
        let meta = e
            .the_move
            .clone()
            .into_attack(target_cell)
            .animate(selected_unit.team, game, world, doop)
            .await
            .apply(selected_unit.team, game, world);

        let effect = e.effect.combine(meta);

        LoopRes::EndTurn((
            moves::ActualMove {
                original: e.the_move.original,
                moveto: e.the_move.moveto,
                attackto: target_cell,
            },
            effect,
        ))
    } else {
        let p = unit.position;
        let this_unit = game
            .factions
            .relative_mut(selected_unit.team)
            .this_team
            .find_slow_mut(&p)
            .unwrap();
        let c = target_cell;
        let mut kk = this_unit.clone();
        kk.position = target_cell;

        let mp = move_build::MovePhase {
            original: p,
            moveto: target_cell,
        };

        let effect = mp
            .animate(selected_unit.team, game, world, doop)
            .await
            .apply(selected_unit.team, game);

        {
            *have_moved = Some(selection::HaveMoved {
                the_move: mp,
                effect,
            });
            selected_unit.coord = c;
            selected_unit.team = team;
            LoopRes::Select(selected_unit)
        }
    }
}

pub fn game_init(world: &board::MyWorld) -> GameState {
    let powerup = true;

    let cats = world
        .cat_start()
        .iter()
        .map(|&position| UnitData {
            position,
            typ: Type::Warrior { powerup },
            has_powerup: false,
        })
        .collect();

    //player
    let dogs = world
        .dog_start()
        .iter()
        .map(|&position| UnitData {
            position,
            typ: Type::Warrior { powerup },
            has_powerup: false,
        })
        .collect();

    let powerups = vec![]; //vec![[1, 1], [1, -2], [-2, 1]];

    let fog = world.get_game_cells().clone();

    let mut k = GameState {
        factions: Factions {
            dogs: Tribe { units: dogs },
            cats: Tribe { units: cats },
        },
        env: Environment {
            land: BitField::from_iter([]),
            forest: BitField::from_iter([]),
            fog,
            powerups: powerups.into_iter().map(Axial::from_arr).collect(),
        },
    };

    for a in k.factions.cats.iter().chain(k.factions.dogs.iter()) {
        move_build::compute_fog(a.position, &mut k.env).apply(a.position, &mut k.env);
    }

    k
}

pub mod share {
    // enum LoadError{
    //     Decode(base64::DecodeError),
    //     Decompres(miniz_oxide::inflate::DecompressError),
    //     Serialize(postcard::Error)
    // }
    // impl From<postcard::Error> for LoadError {
    //     fn from(e: postcard::Error) -> Self {
    //         LoadError::Serialize(e)
    //     }
    // }
    // impl From<base64::DecodeError> for LoadError {
    //     fn from(e: base64::DecodeError) -> Self {
    //         LoadError::Decode(e)
    //     }
    // }
    // impl From<miniz_oxide::inflate::DecompressError> for LoadError {
    //     fn from(e: miniz_oxide::inflate::DecompressError) -> Self {
    //         LoadError::Decompres(e)
    //     }
    // }

    pub struct LoadError;

    use super::*;
    pub fn load(s: &str) -> Result<selection::JustMoveLog, LoadError> {
        use base64::prelude::*;
        let k = BASE64_STANDARD_NO_PAD.decode(s).map_err(|_| LoadError)?;
        let k = miniz_oxide::inflate::decompress_to_vec(&k).map_err(|_| LoadError)?;
        Ok(postcard::from_bytes(&k).map_err(|_| LoadError)?)
    }
    pub fn save(game_history: &selection::JustMoveLog) -> String {
        use base64::prelude::*;

        let k = postcard::to_allocvec(game_history).unwrap();

        let k = miniz_oxide::deflate::compress_to_vec(&k, 10);
        BASE64_STANDARD_NO_PAD.encode(k)
    }
}

pub async fn replay(world: &board::MyWorld, mut doop: WorkerManager, just_logs: JustMoveLog) {
    let mut game = ace::game_init(world);

    let mut game_history = selection::MoveHistory::new();

    let mut team_gen = ActiveTeam::Dogs.iter();

    doop.send_command(ActiveTeam::Dogs, &mut game, Command::HideUndo)
        .await;

    for the_move in just_logs.inner {
        let team = team_gen.next().unwrap();

        if let Some(g) = game.game_is_over(world, team) {
            console_dbg!("Game over=", g);
            todo!();
        }

        let kk = the_move.as_move();

        let effect_m = kk
            .animate(team, &mut game, world, &mut doop)
            .await
            .apply(team, &mut game);

        let effect_a = kk
            .into_attack(the_move.attackto)
            .animate(team, &mut game, world, &mut doop)
            .await
            .apply(team, &mut game, world);

        game_history.push((the_move, effect_m.combine(effect_a)));
    }
}

pub async fn handle_player(
    game: &mut GameState,
    world: &board::MyWorld,
    doop: &mut WorkerManager,
    team: ActiveTeam,
    move_log: &mut selection::MoveHistory,
) -> (moves::ActualMove, move_build::CombinedEffect) {
    let undo = |move_log: &mut selection::MoveHistory, game: &mut GameState| {
        log!("undoing turn!!!");
        assert!(move_log.inner.len() >= 2, "Not enough moves to undo");

        let (a, e) = move_log.inner.pop().unwrap();
        a.as_extra().undo(&e.extra_effect, game);
        a.as_move().undo(team.not(), &e.move_effect, game);

        let (a, e) = move_log.inner.pop().unwrap();
        a.as_extra().undo(&e.extra_effect, game);
        a.as_move().undo(team, &e.move_effect, game);
    };

    let mut extra_attack = None;
    //Keep allowing the user to select units
    'outer: loop {
        if move_log.inner.len() >= 2 {
            doop.send_command(team, game, Command::ShowUndo).await;
        } else {
            doop.send_command(team, game, Command::HideUndo).await;
        }

        //Loop until the user clicks on a selectable unit in their team.
        let mut selected_unit = loop {
            let data = doop.get_mouse(team, game).await;

            let cell = match data {
                MouseEvent::Normal(a) => a,
                MouseEvent::Undo => {
                    assert!(extra_attack.is_none());

                    undo(move_log, game);

                    continue 'outer;
                }
            };

            if let Some(unit) = game.factions.relative(team).this_team.find_slow(&cell) {
                break SelectType {
                    coord: unit.position,
                    team,
                };
            }
            if let Some(unit) = game.factions.relative(team).that_team.find_slow(&cell) {
                break SelectType {
                    coord: unit.position,
                    team: team.not(),
                };
            }
        };

        //Keep showing the selected unit's options and keep handling the users selections
        //Until the unit is deselected.
        loop {
            if extra_attack.is_some() {
                doop.send_command(team, game, Command::HideUndo).await;
            }

            let res =
                reselect_loop(doop, game, world, team, &mut extra_attack, selected_unit).await;

            let a = match res {
                LoopRes::EndTurn(r) => {
                    return r;
                }
                LoopRes::Deselect => break,
                LoopRes::Select(a) => a,
                LoopRes::Undo => {
                    assert!(extra_attack.is_none());

                    undo(move_log, game);
                    continue 'outer;
                }
            };
            selected_unit = a;
        }
    }
}
