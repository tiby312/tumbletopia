use super::*;

#[derive(Debug, Clone)]
pub enum CellSelection {
    MoveSelection(Axial, mesh::small_mesh::SmallMesh, Option<HaveMoved>),
    BuildSelection(Axial),
}
impl Default for CellSelection {
    fn default() -> Self {
        CellSelection::BuildSelection(Axial::default())
    }
}

use board::MyWorld;
use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};
use gloo_console::console_dbg;

#[derive(Debug, Clone)]
pub enum AnimationCommand {
    Movement {
        unit: Axial,
        end: Axial,
    },
    Terrain {
        pos: Axial,
        terrain_type: TerrainType,
        dir: AnimationDirection,
    },
}

#[derive(Debug, Clone)]
pub enum AnimationDirection {
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub enum TerrainType {
    Grass,
    Fog,
}

#[derive(Clone, Debug)]
pub struct HaveMoved {
    pub the_move: ActualMove,
    pub effect: move_build::MoveEffect,
}

pub struct GameWrap<T> {
    pub game: unit::GameStateTotal,
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

#[derive(Debug, Clone)]
pub enum Command {
    Animate(AnimationCommand),
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
    Wait,
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
    Button(String),
}

pub async fn map_editor(
    mut doop: WorkerManager,
    world: &board::MyWorld,
    map: unit::Map,
) -> unit::Map {
    let (mut game_total, _starting_team) = unit::GameStateTotal::new(&world, &map);

    enum TT {
        Ice,
        Land,
        Water,
        Forest,
        Start1,
        Start2,
    }

    let mut tt = TT::Water;

    loop {
        let pos = doop.get_mouse(ActiveTeam::White, &mut game_total).await;
        let pos = match pos {
            MouseEvent::Normal(pos) => pos,
            MouseEvent::Button(s) => {
                console_dbg!("map editor received", s);
                tt = match s.as_str() {
                    "b_ice" => TT::Ice,
                    "b_land" => TT::Land,
                    "b_water" => TT::Water,
                    "b_forest" => TT::Forest,
                    "b_start1" => TT::Start1,
                    "b_start2" => TT::Start2,
                    "b_export" => {
                        if let Some(m) = unit::Map::from_game_state(&game_total.tactical, world) {
                            return m;
                        }
                        continue;
                    }
                    _ => panic!("Not supported!"),
                };

                continue;
            }
        };

        let game = &mut game_total.tactical;

        match tt {
            TT::Ice => {
                game.factions.remove(pos);
                game.factions.ice.set_coord(pos, true)
            }
            TT::Land => {
                game.factions.ice.set_coord(pos, false);
                game.factions.remove(pos);
            }
            TT::Water => {
                game.factions.remove(pos);
                game.factions.ice.set_coord(pos, false);
                game.factions.add_cell(pos, 6, ActiveTeam::Neutral);
            }
            TT::Forest => {
                game.factions.ice.set_coord(pos, false);
                game.factions.remove(pos);
                game.factions.add_cell(pos, 1, ActiveTeam::Neutral);
            }
            TT::Start1 => {
                game.factions.remove(pos);
                game.factions.ice.set_coord(pos, false);

                // for a in world.get_game_cells().inner.iter_ones() {
                //     if let Some((_, t)) = game.factions.get_cell_inner(a) {
                //         if t == ActiveTeam::White {
                //             game.factions.remove_inner(a);
                //         }
                //     }
                // }
                game.factions.add_cell(pos, 1, ActiveTeam::White);
            }
            TT::Start2 => {
                game.factions.remove(pos);
                game.factions.ice.set_coord(pos, false);

                // for a in world.get_game_cells().inner.iter_ones() {
                //     if let Some((_, t)) = game.factions.get_cell_inner(a) {
                //         if t == ActiveTeam::Black {
                //             game.factions.remove_inner(a);
                //         }
                //     }
                // }
                game.factions.add_cell(pos, 1, ActiveTeam::Black);
            }
        }
    }
}

pub trait AiInterface {
    fn wait_response(&mut self) -> impl std::future::Future<Output = (ActualMove, i64)> + Send;
    fn send_command(
        &mut self,
        game: &GameState,
        fogs: &[mesh::small_mesh::SmallMesh; 2],
        world: &MyWorld,
        team: ActiveTeam,
        history: &MoveHistory,
    );
    //fn interrupt_render_thread(&mut self);
    fn interrupt_render_thread(&mut self) -> impl std::future::Future<Output = ()>;
}

pub async fn game_play_thread(
    mut doop: WorkerManager,
    world: &board::MyWorld,
    game_type: GameType,
    ai_int: &mut impl AiInterface,
) -> (unit::GameOver, MoveHistory) {
    console_dbg!("gameplay thread start");

    //let (mut game, start_team) = unit::GameStateTotal::new(&world, &map);
    let mut game = world.starting_state.clone();

    let mut game_history = MoveHistory::new();

    let mut team_gen = world.starting_team.iter();

    //Loop over each team!
    loop {
        let team = team_gen.next().unwrap();

        if let Some(g) = game.tactical.game_is_over(&world, team, &game_history) {
            break (g, game_history);
        }

        //Add AIIIIII.
        let foo = match game_type {
            GameType::SinglePlayer(_) => team == ActiveTeam::Black,
            GameType::PassPlay(_) => false,
            GameType::AIBattle(_) => true,
            GameType::MapEditor(_) => unreachable!(),
            GameType::Replay(_) => unreachable!(),
        };

        console_dbg!("main thread iter");
        if foo {
            let the_move = {
                let ai_state = game.tactical.bake_fog(&game.fog[team.index()]);

                //ai::iterative_deepening(&ai_state, &world, team, &game_history)

                ai_int.send_command(&ai_state, &game.fog, &world, team, &game_history);

                use futures::FutureExt;
                let the_move = futures::select!(
                    _ = doop.wait_forever(team, &mut game).fuse()=>unreachable!(),
                    x = ai_int.wait_response().fuse() => x
                );

                ai_int.interrupt_render_thread().await;
                the_move
            };

            let the_move = the_move.0;

            let the_move = if ai::should_pass(&the_move, team, &mut game.tactical, world) {
                console_dbg!("Choosing to pass!");
                ActualMove {
                    moveto: moves::PASS_MOVE_INDEX,
                }
            } else {
                the_move
            };

            console_dbg!("gmae thread has interrupted render thread");

            let effect_m = animate_move(&the_move, team, &mut game, &world, &mut doop)
                .await
                .apply(team, &mut game.tactical, &game.fog[team.index()], &world);

            game.update_fog(world, team);
            game_history.push((the_move, effect_m));

            continue;
        }

        let r = handle_player(&mut game, &world, &mut doop, team, &mut game_history).await;

        game.update_fog(world, team);
        game_history.push(r);
    }
}

pub struct WorkerManager {
    pub sender: Sender<GameWrap<Command>>,
    pub receiver: Receiver<GameWrap<Response>>,
}

impl WorkerManager {
    pub async fn wait_animation(
        &mut self,
        animation: AnimationCommand,
        team: ActiveTeam,
        game: &mut unit::GameStateTotal,
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
        game: &mut unit::GameStateTotal,
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

    async fn get_mouse(
        &mut self,
        team: ActiveTeam,
        game: &mut unit::GameStateTotal,
    ) -> MouseEvent<Axial> {
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

    pub async fn wait_forever(&mut self, team: ActiveTeam, game: &mut unit::GameStateTotal) {
        let data = self.send_command(team, game, Command::Wait).await;

        let Response::AnimationFinish = data else {
            unreachable!();
        };
        //console_db
    }

    pub async fn wait_ai(
        &mut self,
        team: ActiveTeam,
        game: &mut unit::GameStateTotal,
    ) -> ActualMove {
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
        game1: &mut unit::GameStateTotal,
        co: Command,
    ) -> Response {
        //let game2 = std::mem::take(game1);
        self.sender
            .send(GameWrap {
                game: game1.clone(),
                data: co,
                team,
            })
            .await
            .unwrap();

        let GameWrap { data, .. } = self.receiver.next().await.unwrap();

        //std::mem::swap(&mut game, game1);

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
    Pass,
    Select(T),
}

pub async fn reselect_loop(
    doop: &mut WorkerManager,
    game: &mut unit::GameStateTotal,
    world: &board::MyWorld,
    team: ActiveTeam,
    have_moved: &mut Option<HaveMoved>,
    mut selected_unit: SelectType,
) -> LoopRes<SelectType> {
    //console_dbg!(have_moved.is_some());
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
    //create_ai_state(team)
    let (mut cca, _, _) = game
        .tactical
        .bake_fog(&game.fog[team.index()])
        .generate_possible_moves_movement(
            world,
            Some(unwrapped_selected_unit),
            selected_unit.team,
            true,
            false,
            true, //TODO should this be true?
            &game.fog[team.index()],
        );

    let c2 = game
        .tactical
        //.bake_fog(&game.fog[team.index()])
        .factions
        .doop(mesh::small_mesh::conv(unwrapped_selected_unit), world);

    cca.inner &= c2.inner;

    let mut cell = CellSelection::MoveSelection(unwrapped_selected_unit, cca, have_moved.clone());

    let pototo = doop
        .get_mouse_with_mesh(&mut cell, selected_unit.team, game, grey)
        .await;

    let mouse_world = match pototo {
        MouseEvent::Normal(t) => t,
        MouseEvent::Button(s) => {
            if s == "undo" {
                //End the turn. Ok because we are not int he middle of anything.
                //return LoopRes::EndTurn;
                //unreachable!();
                return LoopRes::Undo;
            } else if s == "pass" {
                return LoopRes::Pass;
            } else {
                unreachable!();
            }
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

    if let Some((_, team2)) = game.tactical.factions.get_cell(target_cell) {
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
    if let Some((_, team2)) = game.tactical.factions.get_cell(target_cell) {
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
        .apply(
            selected_unit.team,
            &mut game.tactical,
            &game.fog[team.index()],
            world,
        );

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
    map: &unit::Map,
    history: &MoveHistory,
    world: &board::MyWorld,
    mut doop: WorkerManager,
) -> unit::GameOver {
    //let map = unit::default_map(world);
    let (mut game, starting_team) = unit::GameStateTotal::new(world, map);
    //let mut game_history = MoveHistory::new();

    //let team_gen = starting_team.iter();

    doop.send_command(starting_team, &mut game, Command::HideUndo)
        .await;

    let mut counter = 0;

    let mut team_counter = starting_team;
    loop {
        let pos = doop.get_mouse(ActiveTeam::White, &mut game).await;
        match pos {
            MouseEvent::Normal(_) => continue,
            MouseEvent::Button(s) => {
                match s.as_str() {
                    "b_prev" => {
                        if counter > 0 {
                            counter -= 1;
                            team_counter = team_counter.not();

                            let (the_move, effect) = &history.inner[counter];

                            the_move.undo(team_counter, &effect, &mut game.tactical);
                        }
                    }
                    "b_next" => {
                        if counter < history.inner.len() {
                            let (the_move, _) = &history.inner[counter];

                            let _ =
                                animate_move(&the_move, team_counter, &mut game, world, &mut doop)
                                    .await
                                    .apply(
                                        team_counter,
                                        &mut game.tactical,
                                        &game.fog[team_counter.index()],
                                        world,
                                    );

                            counter += 1;
                            team_counter = team_counter.not();
                        }
                    }
                    _ => panic!("Not supported!"),
                };

                continue;
            }
        };
    }

    // for (the_move, effect) in history.inner.iter() {
    //     let team = team_gen.next().unwrap();

    //     //let kk = the_move.as_move();

    //     let effect_m = animate_move(&the_move, team, &mut game, world, &mut doop)
    //         .await
    //         .apply(team, &mut game, world);

    //     // let effect_a = kk
    //     //     .into_attack(the_move.attackto)
    //     //     .animate(team, &mut game, world, &mut doop)
    //     //     .await
    //     //     .apply(team, &mut game, world, &effect_m);

    //     //game_history.push((the_move, effect_m));
    // }

    // if let Some(g) = game.game_is_over(world, team_gen.next().unwrap(), &history) {
    //     g
    // } else {
    //     panic!("replay didnt end with game over state");
    // }
}

pub async fn animate_move<'a>(
    aa: &'a ActualMove,
    team: ActiveTeam,
    state: &unit::GameStateTotal,
    world: &board::MyWorld,
    data: &mut WorkerManager,
) -> &'a ActualMove {
    let end_points = state
        .tactical
        .bake_fog(&state.fog[team.index()])
        .factions
        .iter_end_points(world, aa.moveto);

    let mut ss = state.clone();

    let mut stack = 0;
    for (i, (dis, rest)) in end_points.into_iter().enumerate() {
        let Some((_, team2)) = rest else {
            continue;
        };

        if team2 != team {
            continue;
        }

        let unit = mesh::small_mesh::inverse(aa.moveto)
            .add(hex::Cube::from_arr(hex::OFFSETS[i]).ax.mul(dis as i8));

        data.wait_animation(
            AnimationCommand::Movement {
                unit,
                end: mesh::small_mesh::inverse(aa.moveto),
            },
            team,
            &mut ss,
        )
        .await;

        stack += 1;
        if let Some(_) = state.tactical.factions.get_cell_inner(aa.moveto) {
            ss.tactical.factions.remove_inner(aa.moveto);
        }
        ss.tactical.factions.add_cell_inner(aa.moveto, stack, team);
    }

    aa
}

pub async fn handle_player(
    game: &mut unit::GameStateTotal,
    world: &board::MyWorld,
    doop: &mut WorkerManager,
    team: ActiveTeam,
    move_log: &mut MoveHistory,
) -> (moves::ActualMove, move_build::MoveEffect) {
    let undo = |move_log: &mut MoveHistory, game: &mut GameState| {
        //log!("undoing turn!!!");
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
                MouseEvent::Button(s) => {
                    if s == "undo" {
                        assert!(extra_attack.is_none());

                        undo(move_log, &mut game.tactical);

                        continue 'outer;
                    } else if s == "pass" {
                        let mp = ActualMove {
                            moveto: moves::PASS_MOVE_INDEX,
                        };

                        let me = mp.apply(team, &mut game.tactical, &game.fog[team.index()], world);
                        return (mp, me);
                    } else {
                        unreachable!();
                    }
                }
            };

            if let Some((_, team2)) = game.tactical.factions.get_cell(cell) {
                break SelectType {
                    coord: cell,
                    team: team2,
                };
            }
        };

        //TODO simplify this into one loop
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

                    undo(move_log, &mut game.tactical);
                    continue 'outer;
                }
                LoopRes::Pass => {
                    let mp = ActualMove {
                        moveto: moves::PASS_MOVE_INDEX,
                    };

                    let me = mp.apply(team, &mut game.tactical, &game.fog[team.index()], world);
                    return (mp, me);
                }
            };
            selected_unit = a;
        }
    }
}
