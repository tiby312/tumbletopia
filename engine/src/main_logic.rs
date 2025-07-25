use crate::{
    mesh::small_mesh::SmallMesh,
    move_build::{GenericMove, LighthouseMove, NormalMove},
    unit::LastSeenObjects,
};

use super::*;

#[derive(Debug, Clone)]
pub enum CellSelection {
    MoveSelection(
        Axial,
        mesh::small_mesh::SmallMesh,
        //mesh::small_mesh::SmallMesh,
        Option<HaveMoved>,
    ),
    BuildSelection(Axial),
}
impl Default for CellSelection {
    fn default() -> Self {
        CellSelection::BuildSelection(Axial::default())
    }
}

use board::MyWorld;
use futures::{
    SinkExt, StreamExt,
    channel::mpsc::{Receiver, Sender},
};
use hex::PASS_MOVE_INDEX;

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
    pub the_move: Coordinate,
    pub effect: move_build::NormalMoveEffect,
}

pub struct GameWrap<T> {
    pub game: unit::GameStateTotal,
    pub team: Team,
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
    RepaintUI(String),
}

#[derive(Debug)]
pub enum Response {
    //MouseWithSelection(CellSelection, MouseEvent<Axial>),
    Mouse(MouseEvent<Axial>),
    AnimationFinish,
    AiFinish(Coordinate),
    //ChangeGameState(String),
    Ack,
}

#[derive(Debug)]
pub enum MouseEvent<T> {
    Normal(T),
    Button(String),
}

pub async fn map_editor(mut doop: CommandSender, world: &board::MyWorld) -> unit::Map {
    let game = world.starting_state.clone();
    let last_seen = std::array::from_fn(|_| LastSeenObjects {
        state: game.clone(),
    });
    let mut game_total = unit::GameStateTotal {
        tactical: game,
        last_seen,
        history: MoveHistory::new(),
    };

    enum TT {
        Player1,
        Player2,
        Player3,
        Empty,
    }

    let mut tt = TT::Player1;
    use StackHeight::*;

    let mut curr_stack = Stack1;
    loop {
        let pos = doop.get_mouse(Team::White, &mut game_total).await;
        let pos = match pos {
            MouseEvent::Normal(pos) => pos,
            MouseEvent::Button(s) => {
                match s.as_str() {
                    "stack1" => {
                        curr_stack = Stack1;
                    }
                    "stack2" => {
                        curr_stack = Stack2;
                    }
                    "stack3" => {
                        curr_stack = Stack3;
                    }
                    "stack4" => {
                        curr_stack = Stack4;
                    }
                    "stack5" => {
                        curr_stack = Stack5;
                    }
                    "stack6" => {
                        curr_stack = Stack6;
                    }
                    "empty" => {
                        tt = TT::Empty;
                    }
                    "player1" => {
                        tt = TT::Player1;
                    }
                    "player2" => {
                        tt = TT::Player2;
                    }
                    "player3" => {
                        tt = TT::Player3;
                    }
                    _ => {
                        //gloo_console::console_dbg!("Received unsupported button input:", s);
                        continue;
                    }
                };

                continue;
            }
        };

        let game = &mut game_total.tactical;

        match tt {
            TT::Player1 => {
                game.factions.remove(pos);
                game.factions.add_cell(pos, curr_stack, Team::White, false);
            }
            TT::Player2 => {
                game.factions.remove(pos);
                game.factions.add_cell(pos, curr_stack, Team::Black, false);
            }
            TT::Player3 => {
                game.factions.remove(pos);
                game.factions
                    .add_cell(pos, curr_stack, Team::Neutral, false);
            }
            TT::Empty => {
                game.factions.remove(pos);
            }
        }

        let s = game.into_string(world);
        let s = format!("{}{}", "game string:", s);
        doop.repaint_ui(Team::Neutral, &mut game_total, s).await;
    }
}

//purpose of this trait is to keep as much game logic in this crate without addign more dependencies to this crate
pub trait AiInterface {
    fn wait_response(&mut self) -> impl std::future::Future<Output = ai::Res> + Send;
    fn send_command(
        &mut self,
        game: &GameState,
        fogs: &[mesh::small_mesh::SmallMesh; 2],
        world: &MyWorld,
        team: Team,
        history: &MoveHistory,
    );
    //fn interrupt_render_thread(&mut self);
    fn interrupt_render_thread(&mut self) -> impl std::future::Future<Output = ()>;
}

pub struct CommandSender {
    pub sender: Sender<GameWrap<Command>>,
    pub receiver: Receiver<GameWrap<Response>>,
}

impl CommandSender {
    pub async fn wait_animation(
        &mut self,
        animation: AnimationCommand,
        team: Team,
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
        cell: &CellSelection,
        team: Team,
        game: &mut unit::GameStateTotal,
        grey: bool,
    ) -> MouseEvent<Axial> {
        //let selection = std::mem::take(cell);

        let b = self
            .send_command(
                team,
                game,
                Command::GetMouseInputSelection {
                    selection: cell.clone(),
                    grey,
                },
            )
            .await;

        let Response::Mouse(o) = b else {
            unreachable!();
        };

        //std::mem::swap(&mut cell2, cell);

        o
    }

    pub async fn get_mouse(
        &mut self,
        team: Team,
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
    pub async fn repaint_ui(&mut self, team: Team, game: &mut unit::GameStateTotal, foo: String) {
        let data = self.send_command(team, game, Command::RepaintUI(foo)).await;

        let Response::Ack = data else {
            unreachable!();
        };
        //console_db
    }

    pub async fn wait_forever(&mut self, team: Team, game: &mut unit::GameStateTotal) {
        let data = self.send_command(team, game, Command::Wait).await;

        let Response::AnimationFinish = data else {
            unreachable!();
        };
        //console_db
    }

    pub async fn wait_ai(&mut self, team: Team, game: &mut unit::GameStateTotal) -> Coordinate {
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
        team: Team,
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
    team: Team,
}

#[derive(Debug)]
pub enum LoopRes<T> {
    EndTurn(move_build::GenericMove<NormalMove, LighthouseMove>),
    Deselect,
    Undo,
    Pass,
    Select(T),
}

pub async fn reselect_loop(
    doop: &mut CommandSender,
    game: &mut unit::GameStateTotal,
    world: &board::MyWorld,
    team: Team,
    have_moved: &mut Option<HaveMoved>,
    mut selected_unit: SelectType,
    lighthouse_mode: bool,
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
            e.the_move.0 != selected_unit.coord.to_index()
        } else {
            false
        }
    } else {
        true
    };

    let game2 = game.tactical.convert_to_playable(world, selected_unit.team);
    let spoke_info = moves::SpokeInfo::new(&game2, world);

    let cca = {
        if lighthouse_mode {
            gloo_console::console_dbg!("Logging mesh thing");
            let mut cca = SmallMesh::from_iter_move(
                LighthouseMove::possible_moves(
                    &game2,
                    world,
                    selected_unit.team,
                    &spoke_info,
                    true,
                )
                .map(|x| x.coord),
            );

            let c2 = game
                .tactical
                .factions
                .filter_los(unwrapped_selected_unit.to_index(), world);

            cca.inner &= c2.inner;
            cca
        } else {
            let mut cca = SmallMesh::from_iter_move(
                NormalMove::possible_moves(&game2, world, selected_unit.team, &spoke_info, true)
                    .chain([NormalMove::new_pass()])
                    .map(|x| x.coord),
            );

            let c2 = game
                .tactical
                .factions
                .filter_los(unwrapped_selected_unit.to_index(), world);

            cca.inner &= c2.inner;
            cca
        }
    };

    let cell = CellSelection::MoveSelection(unwrapped_selected_unit, cca, have_moved.clone());

    let pototo = doop
        .get_mouse_with_mesh(&cell, selected_unit.team, game, grey)
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
            } else if s == "lighthouse" {
                gloo_console::console_dbg!("Can't change unit type while already selected");
                return LoopRes::Deselect;
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

    match game.tactical.factions.get_cell(target_cell) {
        unit::GameCell::Piece(unit::Piece {
            height: stack_height,
            team: team2,
            ..
        }) => {
            if *team2 == selected_unit.team {
                if !contains {
                    //it should be impossible for a unit to move onto a friendly
                    //assert!(!contains);
                    selected_unit.coord = target_cell;
                    return LoopRes::Select(selected_unit);
                }
            }
        }
        unit::GameCell::Empty => {}
    }

    //If we select an enemy unit quick swap
    match game.tactical.factions.get_cell(target_cell) {
        unit::GameCell::Piece(unit::Piece {
            height: stack_height,
            team: team2,
            ..
        }) => {
            if *team2 == selected_unit.team {
                if selected_unit.team == !team || !contains {
                    //If we select an enemy unit thats outside of our units range.
                    selected_unit.coord = target_cell;
                    selected_unit.team = selected_unit.team.not();
                    return LoopRes::Select(selected_unit);
                }
            }
        }
        unit::GameCell::Empty => {}
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
        if unwrapped_selected_unit.to_index() != e.the_move.0 {
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

    let mp = Coordinate(target_cell.to_index());

    if lighthouse_mode {
        return LoopRes::EndTurn(GenericMove::Lighthouse(LighthouseMove { coord: mp }));
    } else {
        let norm = NormalMove {
            coord: mp,
            stack: mp.determine_stack_height(&game.tactical, world, team, None),
        };

        return LoopRes::EndTurn(GenericMove::Normal(norm));
    }
}

pub async fn replay(
    map: &unit::Map,
    history: &MoveHistory,
    world: &board::MyWorld,
    mut doop: CommandSender,
) -> unit::GameOver {
    todo!();
    //let map = unit::default_map(world);
    // let (mut game, starting_team) = unit::GameStateTotal::new(world, map);
    // //let mut game_history = MoveHistory::new();

    // //let team_gen = starting_team.iter();

    // doop.send_command(starting_team, &mut game, Command::HideUndo)
    //     .await;

    // let mut counter = 0;

    // let mut team_counter = starting_team;
    // loop {
    //     let pos = doop.get_mouse(Team::White, &mut game).await;
    //     match pos {
    //         MouseEvent::Normal(_) => continue,
    //         MouseEvent::Button(s) => {
    //             match s.as_str() {
    //                 "b_prev" => {
    //                     if counter > 0 {
    //                         counter -= 1;
    //                         team_counter = team_counter.not();

    //                         let (the_move, effect) = &history.inner[counter];

    //                         the_move.undo(team_counter, &effect, &mut game.tactical);
    //                     }
    //                 }
    //                 "b_next" => {
    //                     if counter < history.inner.len() {
    //                         let (the_move, _) = &history.inner[counter];

    //                         let _ =
    //                             animate_move(&the_move, team_counter, &mut game, world, &mut doop)
    //                                 .await
    //                                 .apply(
    //                                     team_counter,
    //                                     &mut game.tactical,
    //                                     &game.fog[team_counter.index()],
    //                                     world,
    //                                     None,
    //                                 );

    //                         counter += 1;
    //                         team_counter = team_counter.not();
    //                     }
    //                 }
    //                 _ => panic!("Not supported!"),
    //             };

    //             continue;
    //         }
    //     };
    // }

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

pub async fn handle_player(
    game: &mut unit::GameStateTotal,
    world: &board::MyWorld,
    doop: &mut CommandSender,
    team: Team,
) -> move_build::GenericMove<NormalMove, LighthouseMove> {
    let undo = async |doop: &mut CommandSender, game: &mut unit::GameStateTotal| {
        //log!("undoing turn!!!");
        assert!(game.history.inner.len() >= 2, "Not enough moves to undo");

        let mut mov = vec![];
        for _ in [(); 2] {
            match game.history.inner.pop().unwrap(){
                GenericMove::Normal((a, e)) => {
                    mov.push(a.coord);
                    a.undo( &e, &mut game.tactical);
                }
                GenericMove::Lighthouse((a, e)) => {
                    mov.push(a.coord);
                    a.undo(&e, &mut game.tactical);
                }
            }
        }

        let s = format!(
            "undoing moves {:?} and {:?}",
            world.format(&mov[0]),
            world.format(&mov[1])
        );
        doop.repaint_ui(team, game, s).await;
    };

    let mut extra_attack = None;
    let mut lighthouse_mode = false;

    //Keep allowing the user to select units
    'outer: loop {
        if game.history.inner.len() >= 2 {
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

                        undo(doop, game).await;

                        continue 'outer;
                    } else if s == "pass" {
                        let mp = NormalMove::new_pass();
                        // let me = mp.apply(
                        //     team,
                        //     &mut game.tactical,
                        //     &game.fog[team.index()],
                        //     world,
                        //     None,
                        // );
                        return GenericMove::Normal(mp);
                    } else if s == "lighthouse" {
                        lighthouse_mode = true;
                        gloo_console::console_dbg!("POTATO");
                        continue 'outer;
                    } else {
                        unreachable!();
                    }
                }
            };

            match game.tactical.factions.get_cell(cell) {
                unit::GameCell::Piece(unit::Piece { team: team2, .. }) => {
                    if *team2 == Team::Neutral {
                        continue 'outer;
                    }

                    break SelectType {
                        coord: cell,
                        team: *team2,
                    };
                }
                unit::GameCell::Empty => {}
            }
        };

        //TODO simplify this into one loop
        //Keep showing the selected unit's options and keep handling the users selections
        //Until the unit is deselected.
        loop {
            if extra_attack.is_some() {
                doop.send_command(team, game, Command::HideUndo).await;
            }

            let res = reselect_loop(
                doop,
                game,
                world,
                team,
                &mut extra_attack,
                selected_unit,
                lighthouse_mode,
            )
            .await;

            let a = match res {
                LoopRes::EndTurn(norm) => {
                    return norm;
                }

                LoopRes::Deselect => break,
                LoopRes::Select(a) => a,
                LoopRes::Undo => {
                    assert!(extra_attack.is_none());

                    undo(doop, game).await;
                    continue 'outer;
                }
                LoopRes::Pass => {
                    let mp = NormalMove::new_pass();
                    // let me = mp.apply(
                    //     team,
                    //     &mut game.tactical,
                    //     &game.fog[team.index()],
                    //     world,
                    //     None,
                    // );
                    return GenericMove::Normal(mp);
                }
            };
            selected_unit = a;
        }
    }
}
