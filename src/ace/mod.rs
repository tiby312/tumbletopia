use super::*;

mod ai;
pub mod selection;
use crate::{
    animation::Animation,
    grids::GridMatrix,
    movement::{self, Filter, GridCoord},
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

        let ProcessedCommand::Animate(a) = a else {
            panic!();
        };

        a
    }

    pub fn take_cell(&mut self) -> MousePrompt {
        let mut a = ProcessedCommand::Nothing;
        std::mem::swap(self, &mut a);

        let ProcessedCommand::GetMouseInput(a) = a else {
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
                AnimationCommand::Movement {
                    unit,
                    mesh,
                    walls,
                    end,
                } => {
                    let it = animation::movement(unit.position, mesh, walls, end, grid);
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
        let Response::AnimationFinish(o) = data else {
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

        let MousePrompt::Selection {
            selection,
            grey: grey2,
        } = b
        else {
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

        let Response::Mouse(cell, o) = data else {
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
    _game_history: &mut selection::MoveLog,
) -> LoopRes<SelectType> {
    console_dbg!(extra_attack.is_some());
    //At this point we know a friendly unit is currently selected.

    //let mut relative_game_view = game.view_mut(selected_unit.team);

    let unwrapped_selected_unit = selected_unit.warrior;

    let unit = game
        .factions
        .relative(selected_unit.team)
        .this_team
        .find_slow(&unwrapped_selected_unit)
        .unwrap();

    // let selection = if let Some(e) = extra_attack {
    //     if e.coord() == unwrapped_selected_unit {
    //         selection::SelectionType::Extra(e.select())
    //     } else {
    //         selection::SelectionType::Normal(selection::RegularSelection::new(unit))
    //     }
    // } else {
    //     selection::SelectionType::Normal(selection::RegularSelection::new(unit))
    // };

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

    let cca = moves::partial_move::generate_unit_possible_moves_inner(
        &unit.position,
        unit.typ,
        game,
        selected_unit.team,
        extra_attack.is_some(),
    );

    //let cc = relative_game_view.get_unit_possible_moves(&unit, extra_attack);
    let cc = CellSelection::MoveSelection(unwrapped_selected_unit, cca.clone());

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
    let CellSelection::MoveSelection(_, ss) = cell else {
        unreachable!()
    };

    //If we just clicked on ourselves, just deselect.
    if target_cell == unwrapped_selected_unit {
        return LoopRes::Deselect;
    }

    let contains = movement::contains_coord(ss.iter_mesh(unwrapped_selected_unit), target_cell);

    //If we select a friendly unit quick swap
    if let Some(target) = game
        .factions
        .relative(selected_unit.team)
        .this_team
        .find_slow(&target_cell)
    {
        if !contains {
            //it should be impossible for a unit to move onto a friendly
            //assert!(!contains);
            return LoopRes::Select(selected_unit.with(target.position));
        }
    }

    //If we select an enemy unit quick swap
    if let Some(target) = game
        .factions
        .relative(selected_unit.team)
        .that_team
        .find_slow(&target_cell)
    {
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

    {
        if let Some(e) = extra_attack {
            let iii = moves::partial_move::PartialMove {
                selected_unit: e.coord(),
                typ: unit.typ,
                end: target_cell,
                is_extra: true,
            };

            iii.execute_with_animation(game, selected_unit.team, doop, cca.clone())
                .await;

            return LoopRes::EndTurn;
        } else {
            let iii = moves::PartialMove {
                selected_unit: unit.position,
                typ: unit.typ,
                end: target_cell,
                is_extra: false,
            };
            let c = target_cell;
            let mut kk = unit.clone();
            kk.position = target_cell;

            let iii = iii
                .execute_with_animation(game, selected_unit.team, doop, cca.clone())
                .await;

            {
                *extra_attack = Some(selection::PossibleExtra::new(iii, kk));
                return LoopRes::Select(selected_unit.with(c).with_team(team_index));
            }
        }
    }
}

pub async fn main_logic<'a>(
    command_sender: Sender<GameWrap<'a, Command>>,
    response_recv: Receiver<GameWrapResponse<'a, Response>>,
    game: &'a mut GameState,
) {
    let mut game_history = selection::MoveLog::new();

    let mut doop = WorkerManager {
        game: game as *mut _,
        sender: command_sender,
        receiver: response_recv,
    };

    //Loop over each team!
    'game_loop: for team_index in ActiveTeam::Dogs.iter() {
        if let Some(g) = moves::partial_move::game_is_over(game, team_index) {
            console_dbg!("Game over=", g);
            break 'game_loop;
        }

        //Add AIIIIII.
        if team_index == ActiveTeam::Cats {
            //{
            //if false {
            let the_move = ai::iterative_deepening(game, team_index);

            the_move.execute_move_ani(game, team_index, &mut doop).await;

            console_dbg!(ai::absolute_evaluate(game, true));

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
                        log!("cant end turn!");
                        //game_history.push(moves::ActualMove::SkipTurn);

                        break 'select_loop;
                    }
                };
                //let game = game.view_mut(team_index);

                if let Some(unit) = game
                    .factions
                    .relative(team_index)
                    .this_team
                    .find_slow(&cell)
                {
                    break SelectType {
                        warrior: unit.position,
                        team: team_index,
                    };
                }
                if let Some(unit) = game
                    .factions
                    .relative(team_index)
                    .that_team
                    .find_slow(&cell)
                {
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
}

//TODO use this!
#[derive(Copy, Clone)]
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
