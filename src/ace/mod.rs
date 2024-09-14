use super::*;

pub mod selection;
use crate::{CellSelection, GameState};

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
    WaitAI,
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
    AiFinish(ActualMove),
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

    pub async fn wait_ai(&mut self, team: ActiveTeam, game: &mut GameState) -> ActualMove {
        let data = self.send_command(team, game, Command::WaitAI).await;

        let Response::AiFinish(the_move) = data else {
            unreachable!();
        };
        //console_dbg!("woke up");
        the_move
    }

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

#[derive(Debug)]
pub struct SelectType {
    coord: Axial,
    team: ActiveTeam,
}

#[derive(Debug)]
pub enum LoopRes<T> {
    EndTurn((moves::ActualMove, move_build::MoveEffect)),
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

    // assert!(game
    //     .factions
    //     .relative(selected_unit.team)
    //     .this_team
    //     .is_set(unwrapped_selected_unit));

    let grey = if selected_unit.team == team {
        //If we are in the middle of a extra attack move, make sure
        //no other friendly unit is selectable until we finish moving the
        //the unit that has been partially moved.
        if let Some(e) = have_moved {
            e.the_move.moveto != mesh::small_mesh::conv(selected_unit.coord)
        } else {
            false
        }
    } else {
        true
    };

    // let cca = if let Some(have_moved) = have_moved {
    //     (selected_unit.coord == have_moved.the_move.moveto).then(|| {
    //         game.generate_possible_moves_extra(
    //             world,
    //             &have_moved.the_move,
    //             &have_moved.effect,
    //             selected_unit.team,
    //         )
    //     })
    // } else {
    //     None
    // };

    // let cca = cca.unwrap_or_else(|| {
    //
    // });
    let (cca, _, _) = game.generate_possible_moves_movement(
        world,
        Some(unwrapped_selected_unit),
        selected_unit.team,
    );

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

    let contains = ss.is_set(target_cell);

    //If we just clicked on ourselves, just deselect.
    if target_cell == unwrapped_selected_unit && !contains {
        return LoopRes::Deselect;
    }

    //If we select a friendly unit quick swap

    if let Some((_, team2)) = game.factions.get_cell(target_cell) {
        if team2 == selected_unit.team {
            if !contains {
                //it should be impossible for a unit to move onto a friendly
                //assert!(!contains);
                selected_unit.coord = target_cell;
                return LoopRes::Select(selected_unit);
            }
        }
    }

    //If we select an enemy unit quick swap
    if let Some((_, team2)) = game.factions.get_cell(target_cell) {
        if team2 == selected_unit.team {
            if selected_unit.team != team || !contains {
                //If we select an enemy unit thats outside of our units range.
                selected_unit.coord = target_cell;
                selected_unit.team = selected_unit.team.not();
                return LoopRes::Select(selected_unit);
            }
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
        if mesh::small_mesh::conv(unwrapped_selected_unit) != e.the_move.moveto {
            return LoopRes::Deselect;
        }
    }

    //At this point all re-selecting of units based off of the input has occured.
    //We definately want to act on the action the user took on the selected unit.

    // if let Some(e) = have_moved.take() {
    //     let meta = e
    //         .the_move
    //         .clone()
    //         .into_attack(target_cell)
    //         .animate(selected_unit.team, game, world, doop)
    //         .await
    //         .apply(selected_unit.team, game, world, &e.effect);

    //     let effect = e.effect.combine(meta);

    //     LoopRes::EndTurn((
    //         moves::ActualMove {
    //             original: e.the_move.original,
    //             moveto: e.the_move.moveto,
    //             attackto: target_cell,
    //         },
    //         effect,
    //     ))
    // } else {
    // assert!(game
    //     .factions
    //     .relative_mut(selected_unit.team)
    //     .this_team
    //     .is_set(unwrapped_selected_unit));

    //let c = target_cell;

    let mp = ActualMove {
        //original: unwrapped_selected_unit,
        moveto: mesh::small_mesh::conv(target_cell),
    };

    let effect = animate_move(&mp, selected_unit.team, game, world, doop)
        .await
        .apply(selected_unit.team, game, world);

    {
        LoopRes::EndTurn((
            moves::ActualMove {
                //original: mp.original,
                moveto: mp.moveto,
                //attackto: target_cell,
            },
            effect,
        ))
        // *have_moved = Some(selection::HaveMoved {
        //     the_move: mp,
        //     effect,
        // });
        // selected_unit.coord = c;
        // selected_unit.team = team;
        // LoopRes::Select(selected_unit)
    }
    //}
}

pub async fn replay(
    world: &board::MyWorld,
    mut doop: WorkerManager,
    just_logs: engine::JustMoveLog,
) -> (GameOver, engine::MoveHistory) {
    let mut game = ace::game_init(world);

    let mut game_history = engine::MoveHistory::new();

    let start_team = ActiveTeam::White;
    let mut team_gen = start_team.iter();

    doop.send_command(start_team, &mut game, Command::HideUndo)
        .await;

    for the_move in just_logs.inner {
        let team = team_gen.next().unwrap();

        //let kk = the_move.as_move();

        let effect_m = animate_move(&the_move, team, &mut game, world, &mut doop)
            .await
            .apply(team, &mut game, world);

        // let effect_a = kk
        //     .into_attack(the_move.attackto)
        //     .animate(team, &mut game, world, &mut doop)
        //     .await
        //     .apply(team, &mut game, world, &effect_m);

        game_history.push((the_move, effect_m));
    }

    if let Some(g) = game.game_is_over(world, team_gen.next().unwrap()) {
        (g, game_history)
    } else {
        panic!("replay didnt end with game over state");
    }
}

pub async fn handle_player(
    game: &mut GameState,
    world: &board::MyWorld,
    doop: &mut WorkerManager,
    team: ActiveTeam,
    move_log: &mut engine::MoveHistory,
) -> (moves::ActualMove, move_build::MoveEffect) {
    let undo = |move_log: &mut engine::MoveHistory, game: &mut GameState| {
        log!("undoing turn!!!");
        assert!(move_log.inner.len() >= 2, "Not enough moves to undo");

        let (a, e) = move_log.inner.pop().unwrap();
        //a.as_extra().undo(&e.extra_effect, game);
        a.undo(team.not(), &e, game);

        let (a, e) = move_log.inner.pop().unwrap();
        //a.as_extra().undo(&e.extra_effect, game);
        a.undo(team, &e, game);
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

            if let Some((_, team2)) = game.factions.get_cell(cell) {
                break SelectType {
                    coord: cell,
                    team: team2,
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

            console_dbg!(res);
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
