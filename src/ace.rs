use super::*;

use crate::{
    animation::Animation,
    grids::GridMatrix,
    movement::{self, Filter, GridCoord, MoveUnit, NoPath},
    terrain::{self},
    CellSelection, Game, UnitData, WarriorType,
};

pub struct GameWrap<'a, T> {
    pub game: &'a Game,
    pub team: ActiveTeam,
    pub data: T,
}

pub struct GameWrapResponse<'a, T> {
    pub game: &'a Game,
    pub data: T,
}

pub trait UnwrapMe {
    type Item;
    fn unwrapme(self, a: AnimationOptions) -> Self::Item;
}
pub struct Movement;
impl UnwrapMe for Movement {
    type Item = WarriorType<UnitData>;

    fn unwrapme(self, a: AnimationOptions) -> Self::Item {
        let AnimationOptions::Movement(a)=a else{
            unreachable!()
        };
        a
    }
}

pub struct Attack;
impl UnwrapMe for Attack {
    type Item = [WarriorType<UnitData>; 2];

    fn unwrapme(self, a: AnimationOptions) -> Self::Item {
        let AnimationOptions::Attack(a)=a else{
            unreachable!()
        };
        a
    }
}

pub struct CounterAttack;
impl UnwrapMe for CounterAttack {
    type Item = [WarriorType<UnitData>; 2];

    fn unwrapme(self, a: AnimationOptions) -> Self::Item {
        let AnimationOptions::CounterAttack(a)=a else{
            unreachable!()
        };
        a
    }
}

pub struct AnimationWrapper<K> {
    pub unwrapper: K,
    pub enu: AnimationOptions,
}

pub enum AnimationOptions {
    Movement(WarriorType<UnitData>),
    Attack([WarriorType<UnitData>; 2]),
    Heal([WarriorType<UnitData>; 2]),
    CounterAttack([WarriorType<UnitData>; 2]),
}
impl AnimationOptions {
    pub fn movement(a: WarriorType<UnitData>) -> AnimationWrapper<Movement> {
        AnimationWrapper {
            unwrapper: Movement,
            enu: AnimationOptions::Movement(a),
        }
    }

    pub fn attack(a: [WarriorType<UnitData>; 2]) -> AnimationWrapper<Attack> {
        AnimationWrapper {
            unwrapper: Attack,
            enu: AnimationOptions::Attack(a),
        }
    }

    pub fn counter_attack(a: [WarriorType<UnitData>; 2]) -> AnimationWrapper<CounterAttack> {
        AnimationWrapper {
            unwrapper: CounterAttack,
            enu: AnimationOptions::CounterAttack(a),
        }
    }
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
    Animate(Animation<AnimationOptions>),
    GetMouseInput(MousePrompt),
    Nothing,
}
impl ProcessedCommand {
    pub fn take_animation(&mut self) -> Animation<AnimationOptions> {
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
            Animate(a) => match a {
                AnimationCommand::Movement { unit, path } => {
                    let it = animation::movement(unit.position, path, grid);
                    let aa = AnimationOptions::Movement(unit);
                    let aa = animation::Animation::new(it, aa);
                    ProcessedCommand::Animate(aa)
                }
                AnimationCommand::Attack { attacker, defender } => {
                    let it = animation::attack(attacker.position, defender.position, grid);
                    let aa = AnimationOptions::Attack([attacker, defender]);
                    let aa = animation::Animation::new(it, aa);
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
    AnimationFinish(Animation<AnimationOptions>),
}

use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};

pub struct Doop<'a> {
    game: *mut Game,
    sender: Sender<GameWrap<'a, Command>>,
    receiver: Receiver<GameWrapResponse<'a, Response>>,
}
impl<'a> Doop<'a> {
    pub fn await_data<'b>(&'b mut self, team_index: ActiveTeam) -> AwaitData<'a, 'b> {
        AwaitData::new(self, team_index)
    }

    pub async fn wait_animation<'c>(
        &mut self,
        animation: animation::AnimationCommand,
        team_index: ActiveTeam,
    ) -> Animation<AnimationOptions> {
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

#[derive(Copy, Clone, Eq, PartialEq)]
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
    warrior: WarriorType<GridCoord>,
    team: ActiveTeam,
}
impl SelectType {
    pub fn with(mut self, a: WarriorType<GridCoord>) -> Self {
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
    doop: &mut Doop<'_>,
    game: &mut Game,
    team_index: ActiveTeam,
    extra_attack: &mut Option<GridCoord>,
    selected_unit: SelectType,
) -> LoopRes<SelectType> {
    //At this point we know a friendly unit is currently selected.

    let mut relative_game_view = game.view(selected_unit.team);

    let unwrapped_selected_unit = selected_unit.warrior;

    let unit = relative_game_view.this_team.lookup(unwrapped_selected_unit);

    let cc = relative_game_view.get_unit_possible_moves(&unit, *extra_attack);
    let cc = CellSelection::MoveSelection(cc);

    let grey = if selected_unit.team == team_index {
        //If we are in the middle of a extra attack move, make sure
        //no other friendly unit is selectable until we finish moving the
        //the unit that has been partially moved.
        if let Some(e) = *extra_attack {
            e != *selected_unit.warrior
        } else {
            false
        }
    } else {
        true
    };

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
    let CellSelection::MoveSelection(ss)=cell else{
        unreachable!()
    };

    //If we just clicked on ourselves, just deselect.
    if target_cell == unwrapped_selected_unit.inner {
        return LoopRes::Deselect;
    }

    let contains = movement::contains_coord(ss.moves.iter().map(|x| &x.target), &target_cell);

    //If we select a friendly unit quick swap
    if let Some(target) = relative_game_view.this_team.find_slow(&target_cell) {
        //it should be impossible for a unit to move onto a friendly
        assert!(!contains);
        return LoopRes::Select(selected_unit.with(target.slim()));
    }

    //If we select an enemy unit quick swap
    if let Some(target) = relative_game_view.that_team.find_slow(&target_cell) {
        if selected_unit.team != team_index || !contains {
            //If we select an enemy unit thats outside of our units range.
            return LoopRes::Select(selected_unit.with(target.slim()).not());
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
    if let Some(e) = *extra_attack {
        if unwrapped_selected_unit.inner != e {
            return LoopRes::Deselect;
        }
    }

    //At this point all re-selecting of units based off of the input has occured.
    //We definately want to act on the action the user took on the selected unit.

    //Reconstruct path by creating all possible paths with path information this time.
    let path = relative_game_view.get_path_from_move(target_cell, &unit, *extra_attack);

    if let Some(target_coord) = relative_game_view.that_team.find_slow_mut(&target_cell) {
        let target_coord = target_coord.as_ref().slim();

        doop.await_data(team_index)
            .resolve_attack(
                selected_unit.warrior,
                target_coord,
                &mut relative_game_view,
                false,
                &path,
            )
            .await;

        let _ = doop
            .await_data(team_index.not())
            .resolve_group_attack(target_cell.to_cube(), &mut relative_game_view.not())
            .await;

        //TODO is this possible?
        for n in target_cell.to_cube().neighbours() {
            doop.await_data(team_index)
                .resolve_group_attack(n, &mut relative_game_view)
                .await;
        }

        //Finish this players turn.
        return LoopRes::EndTurn;
    } else {
        //If we are moving to an empty square.

        let this_unit = relative_game_view
            .this_team
            .lookup_take(selected_unit.warrior);

        let this_unit = doop
            .await_data(team_index)
            .resolve_movement(this_unit, path)
            .await;

        relative_game_view.this_team.add(this_unit);

        let k = doop
            .await_data(team_index.not())
            .resolve_group_attack(target_cell.to_cube(), &mut relative_game_view.not())
            .await;

        //Need to add ourselves back so we can resolve and attacking groups
        //only to remove ourselves again later.
        let k = if let Some(k) = k {
            let j = k.as_ref().slim();

            relative_game_view.this_team.add(k);

            for n in target_cell.to_cube().neighbours() {
                doop.await_data(team_index)
                    .resolve_group_attack(n, &mut relative_game_view)
                    .await;
            }

            Some(relative_game_view.this_team.lookup_take(j))
        } else {
            for n in target_cell.to_cube().neighbours() {
                doop.await_data(team_index)
                    .resolve_group_attack(n, &mut relative_game_view)
                    .await;
            }
            None
        };

        if let Some(k) = k {
            let b = k.as_ref().slim();
            *extra_attack = Some(target_cell);
            relative_game_view.this_team.add(k);
            return LoopRes::Select(selected_unit.with(b).with_team(team_index));
        } else {
            //Finish this players turn.
            return LoopRes::EndTurn;
        }
    }
}

pub async fn main_logic<'a>(
    command_sender: Sender<GameWrap<'a, Command>>,
    response_recv: Receiver<GameWrapResponse<'a, Response>>,
    game: &'a mut Game,
) {
    let mut doop = Doop {
        game: game as *mut _,
        sender: command_sender,
        receiver: response_recv,
    };

    //Loop over each team!
    for team_index in ActiveTeam::Cats.iter() {
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
                        break 'select_loop;
                    }
                };
                let game = game.view(team_index);

                if let Some(unit) = game.this_team.find_slow(&cell) {
                    break SelectType {
                        warrior: unit.slim(),
                        team: team_index,
                    };
                }
                if let Some(unit) = game.that_team.find_slow(&cell) {
                    break SelectType {
                        warrior: unit.slim(),
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

impl<'a> GameView<'a> {
    pub fn get_path_from_move(
        &self,
        target_cell: GridCoord,
        unit: &WarriorType<&UnitData>,
        extra_attack: Option<GridCoord>,
    ) -> movement::Path {
        //Reconstruct possible paths with path information this time.
        let ss = generate_unit_possible_moves_inner(&unit, self, extra_attack, movement::WithPath);

        let path = ss
            .moves
            .iter()
            .find(|a| a.target == target_cell)
            .map(|a| &a.path)
            .unwrap();

        *path
    }

    pub fn get_unit_possible_moves(
        &self,
        unit: &WarriorType<&UnitData>,
        extra_attack: Option<GridCoord>,
    ) -> movement::PossibleMoves2<()> {
        generate_unit_possible_moves_inner(unit, self, extra_attack, NoPath)
    }
}

fn generate_unit_possible_moves_inner<P: movement::PathHave>(
    unit: &WarriorType<&UnitData>,
    game: &GameView,
    extra_attack: Option<GridCoord>,
    ph: P,
) -> movement::PossibleMoves2<P::Foo> {
    // If there is an enemy near by restrict movement.

    let j = if let Some(_) = unit
        .position
        .to_cube()
        .ring(1)
        .map(|s| game.that_team.find_slow(&s.to_axial()).is_some())
        .find(|a| *a)
    {
        1
    } else {
        match unit.val {
            Type::Warrior => 2,
            Type::Para => 1,
            _ => todo!(),
        }
    };

    let mm = MoveUnit(j);

    let mm = if let Some(_) = extra_attack.filter(|&aaa| aaa == unit.position) {
        movement::compute_moves(
            &movement::WarriorMovement,
            &game.world.filter().and(game.that_team.filter()),
            &movement::NoFilter,
            &terrain::Grass,
            unit.position,
            MoveUnit(1),
            false,
            ph,
        )
    } else {
        movement::compute_moves(
            &movement::WarriorMovement,
            &game
                .world
                .filter()
                .and(game.that_team.warriors[0].filter().not()),
            &game.this_team.filter().not(),
            &terrain::Grass,
            unit.position,
            mm,
            true,
            ph,
        )
    };
    mm
}
