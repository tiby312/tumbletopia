use super::*;

mod ai;
pub mod selection;
use crate::{
    animation::Animation,
    grids::GridMatrix,
    movement::{self, Filter, GridCoord, MoveUnit},
    terrain::{self},
    CellSelection, GameState, UnitData,
};

pub struct GameWrap<'a, T> {
    pub game: &'a GameState,
    pub team: ActiveTeam,
    pub data: T,
}

pub struct GameWrapResponse<'a, T> {
    pub game: &'a GameState,
    pub data: T,
}

pub struct AnimationWrapper<K> {
    pub unwrapper: K,
    pub enu: animation::AnimationCommand,
}

#[derive(Debug)]
pub enum MousePrompt {
    Selection {
        selection: CellSelection,
        grey: bool,
    },
    None,
}

pub enum ProcessedCommand {
    Animate(Animation<animation::AnimationCommand>),
    GetMouseInput(MousePrompt),
    Nothing,
}
impl ProcessedCommand {
    pub fn take_animation(&mut self) -> Animation<animation::AnimationCommand> {
        let mut a = ProcessedCommand::Nothing;
        std::mem::swap(self, &mut a);

        let ProcessedCommand::Animate(a)=a else{
            panic!();
        };

        a
    }

    pub fn take_cell(&mut self) -> MousePrompt {
        let mut a = ProcessedCommand::Nothing;
        std::mem::swap(self, &mut a);

        let ProcessedCommand::GetMouseInput(a)=a else{
            panic!();
        };

        a
    }
}
#[derive(Debug)]
pub enum Command {
    Animate(animation::AnimationCommand),
    GetMouseInput(MousePrompt),
    Nothing,
}
impl Command {
    pub fn process(self, grid: &GridMatrix) -> ProcessedCommand {
        use animation::AnimationCommand;
        use Command::*;
        match self {
            Animate(a) => match a.clone() {
                AnimationCommand::Movement { unit, mesh, end } => {
                    let it = animation::movement(unit.position, mesh, end, grid);
                    //let aa = AnimationOptions::Movement(unit);
                    let aa = animation::Animation::new(it, a);
                    ProcessedCommand::Animate(aa)
                }
                AnimationCommand::Attack { attacker, defender } => {
                    let it = animation::attack(attacker.position, defender.position, grid);
                    //let aa = AnimationOptions::Attack([attacker, defender]);
                    let aa = animation::Animation::new(it, a);
                    ProcessedCommand::Animate(aa)
                }
            },
            GetMouseInput(a) => ProcessedCommand::GetMouseInput(a),
            Nothing => ProcessedCommand::Nothing,
        }
    }
}

#[derive(Debug)]
pub enum Pototo<T> {
    Normal(T),
    EndTurn,
}

#[derive(Debug)]
pub enum Response {
    Mouse(MousePrompt, Pototo<GridCoord>), //TODO make grid coord
    AnimationFinish(Animation<animation::AnimationCommand>),
}

use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};

pub struct WorkerManager<'a> {
    game: *mut GameState,
    sender: Sender<GameWrap<'a, Command>>,
    receiver: Receiver<GameWrapResponse<'a, Response>>,
}
impl<'a> WorkerManager<'a> {
    pub async fn wait_animation<'c>(
        &mut self,
        animation: animation::AnimationCommand,
        team_index: ActiveTeam,
    ) -> Animation<animation::AnimationCommand> {
        let game = unsafe { &*self.game };
        self.sender
            .send(GameWrap {
                team: team_index,
                game,
                data: Command::Animate(animation),
            })
            .await
            .unwrap();

        let GameWrapResponse { game: _gg, data } = self.receiver.next().await.unwrap();
        let Response::AnimationFinish(o)=data else{
            unreachable!();
        };

        o
    }

    async fn get_mouse_no_selection<'c>(&mut self, team_index: ActiveTeam) -> Pototo<GridCoord> {
        let (_, c) = self.get_mouse(MousePrompt::None, team_index).await;
        c
    }
    async fn get_mouse_selection<'c>(
        &mut self,
        cell: CellSelection,
        team_index: ActiveTeam,
        grey: bool,
    ) -> (CellSelection, Pototo<GridCoord>) {
        let (b, c) = self
            .get_mouse(
                MousePrompt::Selection {
                    selection: cell,
                    grey,
                },
                team_index,
            )
            .await;

        let MousePrompt::Selection{selection,grey:grey2}=b else{
            unreachable!()
        };
        assert_eq!(grey2, grey);

        (selection, c)
    }

    async fn get_mouse<'c>(
        &mut self,
        cell: MousePrompt,
        team_index: ActiveTeam,
    ) -> (MousePrompt, Pototo<GridCoord>) {
        let game = unsafe { &*self.game };

        self.sender
            .send(GameWrap {
                game,
                data: Command::GetMouseInput(cell),
                team: team_index,
            })
            .await
            .unwrap();

        let GameWrapResponse { game: _gg, data } = self.receiver.next().await.unwrap();

        let Response::Mouse(cell,o)=data else{
            unreachable!();
        };

        (cell, o)
    }
}

#[derive(Hash, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ActiveTeam {
    Cats = 0,
    Dogs = 1,
}
impl ActiveTeam {
    pub fn iter(&self) -> impl Iterator<Item = Self> {
        [*self, self.not()].into_iter().cycle()
    }
    pub fn not(&self) -> Self {
        match self {
            ActiveTeam::Cats => ActiveTeam::Dogs,
            ActiveTeam::Dogs => ActiveTeam::Cats,
        }
    }
}

pub struct SelectType {
    warrior: GridCoord,
    team: ActiveTeam,
}
impl SelectType {
    pub fn with(mut self, a: GridCoord) -> Self {
        self.warrior = a;
        self
    }
    pub fn not(mut self) -> Self {
        self.team = self.team.not();
        self
    }
    pub fn with_team(mut self, a: ActiveTeam) -> Self {
        self.team = a;
        self
    }
}

pub enum LoopRes<T> {
    EndTurn,
    Deselect,
    Select(T),
}

pub async fn reselect_loop(
    doop: &mut WorkerManager<'_>,
    game: &mut GameState,
    team_index: ActiveTeam,
    extra_attack: &mut Option<selection::PossibleExtra>,
    selected_unit: SelectType,
    game_history: &mut selection::MoveLog,
) -> LoopRes<SelectType> {
    console_dbg!(extra_attack.is_some());
    //At this point we know a friendly unit is currently selected.

    let mut relative_game_view = game.view_mut(selected_unit.team);

    let unwrapped_selected_unit = selected_unit.warrior;

    let unit = relative_game_view
        .this_team
        .find_slow(&unwrapped_selected_unit)
        .unwrap();

    let selection = if let Some(e) = extra_attack {
        if e.coord() == unwrapped_selected_unit {
            selection::SelectionType::Extra(e.select())
        } else {
            selection::SelectionType::Normal(selection::RegularSelection::new(unit))
        }
    } else {
        selection::SelectionType::Normal(selection::RegularSelection::new(unit))
    };

    let grey = if selected_unit.team == team_index {
        //If we are in the middle of a extra attack move, make sure
        //no other friendly unit is selectable until we finish moving the
        //the unit that has been partially moved.
        if let Some(e) = extra_attack {
            e.coord() != selected_unit.warrior
        } else {
            false
        }
    } else {
        true
    };

    let ccA = match &selection {
        selection::SelectionType::Normal(e) => e.generate(&relative_game_view),
        selection::SelectionType::Extra(e) => e.generate(&relative_game_view),
    };
    //let cc = relative_game_view.get_unit_possible_moves(&unit, extra_attack);
    let cc = CellSelection::MoveSelection(unwrapped_selected_unit, ccA.clone());

    let (cell, pototo) = doop.get_mouse_selection(cc, selected_unit.team, grey).await;

    let mouse_world = match pototo {
        Pototo::Normal(t) => t,
        Pototo::EndTurn => {
            //End the turn. Ok because we are not int he middle of anything.
            return LoopRes::EndTurn;
        }
    };
    let target_cell = mouse_world;

    //This is the cell the user selected from the pool of available moves for the unit
    let CellSelection::MoveSelection(_,ss)=cell else{
        unreachable!()
    };

    //If we just clicked on ourselves, just deselect.
    if target_cell == unwrapped_selected_unit {
        return LoopRes::Deselect;
    }

    let contains = movement::contains_coord(ss.iter_mesh(unwrapped_selected_unit), target_cell);

    //If we select a friendly unit quick swap
    if let Some(target) = relative_game_view.this_team.find_slow(&target_cell) {
        if !contains {
            //it should be impossible for a unit to move onto a friendly
            //assert!(!contains);
            return LoopRes::Select(selected_unit.with(target.position));
        }
    }

    //If we select an enemy unit quick swap
    if let Some(target) = relative_game_view.that_team.find_slow(&target_cell) {
        if selected_unit.team != team_index || !contains {
            //If we select an enemy unit thats outside of our units range.
            return LoopRes::Select(selected_unit.with(target.position).not());
        }
    }

    //If we selected an empty space, deselect.
    if !contains {
        return LoopRes::Deselect;
    }

    //If we are trying to move an enemy piece, deselect.
    if selected_unit.team != team_index {
        return LoopRes::Deselect;
    }

    // If we are trying to move a piece while in the middle of another
    // piece move, deselect.
    if let Some(e) = extra_attack {
        if unwrapped_selected_unit != e.coord() {
            return LoopRes::Deselect;
        }
    }

    //At this point all re-selecting of units based off of the input has occured.
    //We definately want to act on the action the user took on the selected unit.

    //Reconstruct path by creating all possible paths with path information this time.
    //let path = relative_game_view.get_path_from_move(target_cell, &unit, extra_attack);

    // let path = match selection {
    //     selection::SelectionType::Normal(e) => e
    //         .get_path_from_move(target_cell, &relative_game_view)
    //         .unwrap(),
    //     selection::SelectionType::Extra(e) => e
    //         .get_path_from_move(target_cell, &relative_game_view)
    //         .unwrap(),
    // };

    match selection {
        selection::SelectionType::Normal(n) => {
            match n
                .execute(
                    target_cell,
                    ccA.clone(),
                    &mut relative_game_view,
                    doop,
                    game_history,
                )
                .await
                .unwrap()
            {
                Some(n) => {
                    let c = n.coord();
                    *extra_attack = Some(n);
                    return LoopRes::Select(selected_unit.with(c).with_team(team_index));
                }
                None => {
                    //Finish this players turn.
                    return LoopRes::EndTurn;
                }
            }
        }
        selection::SelectionType::Extra(e) => {
            e.execute(
                target_cell,
                ccA.clone(),
                &mut relative_game_view,
                doop,
                game_history,
            )
            .await
            .unwrap();
            return LoopRes::EndTurn;
        }
    }

    // if let Some(_) = relative_game_view.that_team.find_slow_mut(&target_cell) {
    //     let iii = moves::Invade::new(selected_unit.warrior, path);

    //     let iii = iii
    //         .execute_with_animation(&mut relative_game_view, doop, |_| {})
    //         .await;

    //     if let Some(e) = extra_attack.take() {
    //         game_history.push(moves::ActualMove::ExtraMove(e.prev_move().clone(), iii));
    //     } else {
    //         game_history.push(moves::ActualMove::Invade(iii));
    //     }

    //     //Finish this players turn.
    //     return LoopRes::EndTurn;
    // } else {
    //     //If we are moving to an empty square.

    //     let pm = moves::PartialMove::new(selected_unit.warrior, path);
    //     let jjj = pm
    //         .clone()
    //         .execute_with_animation(&mut relative_game_view, doop, |_| {})
    //         .await;

    //     match jjj {
    //         (sigl, moves::ExtraMove::ExtraMove { pos }) => {
    //             *extra_attack = Some(selection::PossibleExtra::new(sigl, pos));
    //             return LoopRes::Select(selected_unit.with(pos).with_team(team_index));
    //         }
    //         (sigl, moves::ExtraMove::FinishMoving) => {
    //             game_history.push(moves::ActualMove::NormalMove(sigl));
    //             //console_dbg!();
    //             //Finish this players turn.
    //             return LoopRes::EndTurn;
    //         }
    //     }
    // }
}

// pub struct Selector<'a,'b>{
//     game:&'a mut GameView<'b>
// }

// impl<'a,'b> Selector<'a,'b>{
//     pub fn iter_selections(&mut self)->impl Iterator<Item=selection::PossibleMovesNormal>{

//     }
// }

// #[derive(Debug)]
// pub struct ParseErr;
// pub async fn replay<'a>(
//     command_sender: Sender<GameWrap<'a, Command>>,
//     response_recv: Receiver<GameWrapResponse<'a, Response>>,
//     game: &'a mut Game,
// ) -> Result<(), ParseErr> {
//     let mut doop = WorkerManager {
//         game: game as *mut _,
//         sender: command_sender,
//         receiver: response_recv,
//     };

//     //let s="N-1:1:0:-1,N2:-1:0:0,N-1:2:-1:0,N2:-2:2:-1,E-1:0:0:-1:0:-1:1:-1,I0:0:-2:2,S,N-2:2:-3:3,S,S,S,";
//     let s = "N-1:1:-1:-1,N2:-1:0:0,N-1:2:0:2,I0:0:-2:2,FB";
//     let mut k = moves::from_foo(s).map_err(|_| ParseErr)?.into_iter();

//     for team in ActiveTeam::Cats.iter() {
//         let Some(n)=k.next() else{
//             break;
//         };
//         console_dbg!(&n);
//         let mut game_view = game.view_mut(team);
//         match n {
//             moves::ActualMove::Invade(i) => {
//                 let un = game_view.this_team.find_slow(&i.unit).ok_or(ParseErr)?;

//                 let path = selection::PossibleMovesNormal::new(un)
//                     .get_path_from_move(i.moveto, &game_view)
//                     .map_err(|_| ParseErr)?;

//                 moves::Invade::new(i.unit, path)
//                     .execute_with_animation(&mut game_view, &mut doop, |_| {})
//                     .await;
//             }
//             moves::ActualMove::NormalMove(i) => {
//                 let un = game_view.this_team.find_slow(&i.unit).ok_or(ParseErr)?;

//                 let path = selection::PossibleMovesNormal::new(un)
//                     .get_path_from_move(i.moveto, &game_view)
//                     .map_err(|_| ParseErr)?;

//                 moves::PartialMove::new(i.unit, path)
//                     .execute_with_animation(&mut game_view, &mut doop, |_| {})
//                     .await;
//             }
//             moves::ActualMove::ExtraMove(i, j) => {
//                 let un = game_view.this_team.find_slow(&i.unit).ok_or(ParseErr)?;
//                 let path = selection::PossibleMovesNormal::new(un)
//                     .get_path_from_move(i.moveto, &game_view)
//                     .map_err(|_| ParseErr)?;
//                 let k = moves::PartialMove::new(i.unit, path)
//                     .execute_with_animation(&mut game_view, &mut doop, |_| {})
//                     .await;

//                 let moves::ExtraMove::ExtraMove{pos}=k.1 else{
//                     return Err(ParseErr);
//                 };

//                 let sel = selection::PossibleExtra::new(k.0, pos);

//                 let un = game_view.this_team.find_slow(&j.unit).ok_or(ParseErr)?;
//                 let path = sel
//                     .select(un)
//                     .get_path_from_move(j.moveto, &game_view)
//                     .map_err(|_| ParseErr)?;

//                 moves::Invade::new(j.unit, path)
//                     .execute_with_animation(&mut game_view, &mut doop, |_| {})
//                     .await;
//             }
//             moves::ActualMove::SkipTurn => {}
//             moves::ActualMove::GameEnd(g) => match g {
//                 moves::GameEnding::Win(win_team) => {
//                     if win_team == team {
//                         console_dbg!("This team won=", win_team);
//                     } else {
//                         return Err(ParseErr);
//                     }
//                 }
//                 moves::GameEnding::Draw => {
//                     console_dbg!("It was a draw!");
//                 }
//             },
//         }
//     }
//     Ok(())
// }

pub async fn main_logic<'a>(
    command_sender: Sender<GameWrap<'a, Command>>,
    response_recv: Receiver<GameWrapResponse<'a, Response>>,
    game: &'a mut GameState,
) {
    //replay(command_sender, response_recv, game).await.unwrap();
    //return;
    //todo!();
    let mut game_history = selection::MoveLog::new();

    let mut doop = WorkerManager {
        game: game as *mut _,
        sender: command_sender,
        receiver: response_recv,
    };

    //Loop over each team!
    'game_loop: for team_index in ActiveTeam::Dogs.iter() {
        //check if we lost.
        'check_end: {
            let game = game.view_mut(team_index);
            let our_king_dead = game
                .this_team
                .units
                .iter()
                .find(|a| a.typ == Type::King)
                .is_none();
            let their_king_dead = game
                .that_team
                .units
                .iter()
                .find(|a| a.typ == Type::King)
                .is_none();

            let g = match (our_king_dead, their_king_dead) {
                (true, true) => moves::GameEnding::Draw,
                (true, false) => moves::GameEnding::Win(team_index.not()),
                (false, true) => moves::GameEnding::Win(team_index),
                (false, false) => {
                    break 'check_end;
                }
            };
            game_history.push(moves::ActualMove::GameEnd(g));
            break 'game_loop;
        }

        //Add AIIIIII.
        //if team_index == ActiveTeam::Cats {
        //{
        if false {
            let the_move = ai::iterative_deepening(game, team_index);

            let mut game = game.view_mut(team_index);

            match the_move {
                moves::ActualMove::NormalMove(o) => {
                    let unit = game.this_team.find_slow(&o.unit).unwrap();

                    let mesh = selection::generate_unit_possible_moves_inner(unit, &game, None);

                    let r = selection::RegularSelection::new(unit);
                    let r = r
                        .execute(o.moveto, mesh, &mut game, &mut doop, &mut game_history)
                        .await
                        .unwrap();
                    assert!(r.is_none());
                }
                moves::ActualMove::ExtraMove(o, e) => {
                    let unit = game.this_team.find_slow(&o.unit).unwrap().clone();

                    let mesh = selection::generate_unit_possible_moves_inner(&unit, &game, None);

                    let r = selection::RegularSelection::new(&unit);
                    let r = r
                        .execute(o.moveto, mesh, &mut game, &mut doop, &mut game_history)
                        .await
                        .unwrap();
                    //console_dbg!("WOOO");

                    //let unit = game.this_team.find_slow(&o.unit).unwrap().clone();

                    // let mesh =
                    //     selection::generate_unit_possible_moves_inner(&unit, &game, Some(e.unit));

                    let rr = r.unwrap();

                    let rr = rr.select();
                    let mesh = rr.generate(&game);

                    rr.execute(e.moveto, mesh, &mut game, &mut doop, &mut game_history)
                        .await
                        .unwrap();
                }
                moves::ActualMove::SkipTurn => {}
                moves::ActualMove::GameEnd(_) => todo!(),
            }

            continue;
        }

        let mut extra_attack = None;
        //Keep allowing the user to select units

        'select_loop: loop {
            //Loop until the user clicks on a selectable unit in their team.
            let mut selected_unit = loop {
                let data = doop.get_mouse_no_selection(team_index).await;
                let cell = match data {
                    Pototo::Normal(a) => a,
                    Pototo::EndTurn => {
                        log!("End the turn!");
                        game_history.push(moves::ActualMove::SkipTurn);

                        break 'select_loop;
                    }
                };
                let game = game.view_mut(team_index);

                if let Some(unit) = game.this_team.find_slow(&cell) {
                    break SelectType {
                        warrior: unit.position,
                        team: team_index,
                    };
                }
                if let Some(unit) = game.that_team.find_slow(&cell) {
                    break SelectType {
                        warrior: unit.position,
                        team: team_index.not(),
                    };
                }
            };

            //Keep showing the selected unit's options and keep handling the users selections
            //Until the unit is deselected.
            loop {
                let a = match reselect_loop(
                    &mut doop,
                    game,
                    team_index,
                    &mut extra_attack,
                    selected_unit,
                    &mut game_history,
                )
                .await
                {
                    LoopRes::EndTurn => break 'select_loop,
                    LoopRes::Deselect => break,
                    LoopRes::Select(a) => a,
                };
                selected_unit = a;
            }
        }
    }
    //TODO
    // let mut s = String::new();
    // moves::to_foo(&game_history, &mut s).unwrap();
    // console_dbg!(s);
}

// pub struct GameState {}
// pub struct Engine {}
// impl Engine {
//     fn play_move(&mut self, a: Move) {}
//     fn get_state(&self) -> &GameState {
//         todo!()
//     }
//     fn get_valid_moves(&self, a: GridCoord) -> impl Iterator<Item = Move> {
//         std::iter::empty()
//     }
// }

//TODO use this!
pub enum Move {
    Warrior {
        from: GridCoord,
        to: GridCoord,
        extra: Option<GridCoord>,
    },
    King {
        from: GridCoord,
        to: GridCoord,
    },
}

// impl<'a> GameViewMut<'a> {
//     pub fn get_path_from_move(
//         &self,
//         target_cell: GridCoord,
//         unit: &UnitData,
//         extra_attack: &Option<(moves::PartialMove, GridCoord)>,
//     ) -> movement::Path {
//         //Reconstruct possible paths with path information this time.
//         let ss = generate_unit_possible_moves_inner(&unit, self, extra_attack, movement::WithPath);

//         let path = ss
//             .moves
//             .iter()
//             .find(|a| a.target == target_cell)
//             .map(|a| &a.path)
//             .unwrap();

//         *path
//     }

//     pub fn get_unit_possible_moves(
//         &self,
//         unit: &UnitData,
//         extra_attack: &Option<(moves::PartialMove, GridCoord)>,
//     ) -> movement::PossibleMoves2<()> {
//         generate_unit_possible_moves_inner(unit, self, extra_attack, NoPath)
//     }
// }
