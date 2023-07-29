use super::*;

use crate::{
    animation::Animation,
    grids::GridMatrix,
    movement::{self, Filter, GridCoord, MoveUnit},
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

    // async fn get_mouse_selection_enemy<'c>(
    //     &mut self,
    //     cell: CellSelection,
    //     team_index: usize,
    // ) -> (CellSelection, Pototo<GridCoord>) {
    //     let (b, c) = self.get_mouse(MousePrompt::Enemy(cell), team_index).await;

    //     let MousePrompt::Enemy(b)=b else{
    //         unreachable!()
    //     };

    //     (b, c)
    // }

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

#[derive(Copy, Clone)]
pub enum ActiveTeam {
    Cats = 0,
    Dogs = 1,
}
impl ActiveTeam {
    // pub fn next(&mut self){
    //     *self=self.not()
    // }

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


#[derive(Copy, Clone, Debug)]
pub enum TeamType<A> {
    ThisTeam(A),
    ThatTeam(A),
}
impl<A> TeamType<A> {
    pub fn unwrap_this(self) -> A {
        let TeamType::ThisTeam(a)=self else{
            unreachable!()
        };
        a
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

    {
        game.cats.set_health();
        game.dogs.set_health();
        game.cats.replenish_stamina();
        game.dogs.replenish_stamina();
    }

    //Loop over each team!
    for team_index in ActiveTeam::Cats.iter() {
        let game = game.view(team_index);

        let mut extra_attack = None;

        //Keep allowing the user to select units
        'select: loop {
            //Loop until the user clicks on a selectable unit in their team.
            let mut current_warrior_pos = loop {
                let data = doop.get_mouse_no_selection(team_index).await;
                let cell = match data {
                    Pototo::Normal(a) => a,
                    Pototo::EndTurn => {
                        log!("End the turn!");
                        break 'select;
                    }
                };

                if let Some(unit) = game.this_team.find_slow(&cell) {
                    break TeamType::ThisTeam(unit.slim());
                }
                if let Some(unit) = game.that_team.find_slow(&cell) {
                    break TeamType::ThatTeam(unit.slim());
                }
            };

            //Keep showing the selected unit's options and keep handling the users selections
            //Until the unit is deselected.
            loop {
                if let TeamType::ThatTeam(curr_warrior_pos) = current_warrior_pos {
                    let unit = game.that_team.lookup(curr_warrior_pos);

                    let cc = generate_unit_possible_moves2(&unit, &game, None, movement::NoPath);
                    let cc = CellSelection::MoveSelection(cc);

                    let (_, pototo) = doop.get_mouse_selection(cc, team_index, true).await;
                    let target_cell = match pototo {
                        Pototo::Normal(t) => t,
                        Pototo::EndTurn => {
                            //End the turn. Ok because we are not int he middle of anything.
                            break 'select;
                        }
                    };

                    //we clicked on the selected enemy unit.
                    //deselect the unit
                    if target_cell == *curr_warrior_pos {
                        break;
                    }

                    //quick select a friendly unit.
                    if let Some(target) = game.this_team.find_slow(&target_cell) {
                        current_warrior_pos = TeamType::ThisTeam(target.slim());
                        continue;
                    }

                    //quick select a different enemy unit
                    if let Some(target) = game.that_team.find_slow(&target_cell) {
                        current_warrior_pos = TeamType::ThatTeam(target.slim());
                        continue;
                    }

                    //We clicked on an empty space, deselect.
                    break;
                }

                //At this point we know a friendly unit is currently selected.

                let unit = game.this_team.lookup(current_warrior_pos.unwrap_this());

                let cc =
                    generate_unit_possible_moves2(&unit, &game, extra_attack, movement::NoPath);
                let cc = CellSelection::MoveSelection(cc);

                //If we are in the middle of a extra attack move, make sure
                //no other friendly unit is selectable until we finish moving the
                //the unit that has been partially moved.
                let gg = if let Some(e) = extra_attack {
                    if let TeamType::ThisTeam(e2) = current_warrior_pos {
                        e != *e2
                    } else {
                        //TODO unreachble?
                        true
                    }
                } else {
                    false
                };

                let (cell, pototo) = doop.get_mouse_selection(cc, team_index, gg).await;
                let mouse_world = match pototo {
                    Pototo::Normal(t) => t,
                    Pototo::EndTurn => {
                        //End the turn. Ok because we are not int he middle of anything.
                        break 'select;
                    }
                };
                let target_cell = mouse_world;

                //This is the cell the user selected from the pool of available moves for the unit
                let CellSelection::MoveSelection(ss)=cell else{
                    unreachable!()
                };

                //RESELECT STAGE
                if let Some(a) = game.this_team.find_slow(&target_cell) {
                    let k = a.slim();
                    if k != current_warrior_pos.unwrap_this() {
                        //If we select a friendly unit thats not us.
                        //Quick switch to another unit
                        current_warrior_pos = TeamType::ThisTeam(k);
                        continue;
                    } else {
                        //if we click on the unit thats already selected, deselect.
                        //(might want to do somehting else? popup with info?)
                        //Deselect
                        break;
                    }
                } else if let Some(target) = game.that_team.find_slow(&target_cell) {
                    if !movement::contains_coord(ss.moves.iter().map(|x| &x.target), &target_cell) {
                        //If we select an enemy unit thats outside of our units range.
                        current_warrior_pos = TeamType::ThatTeam(target.slim());
                        continue;
                    }
                } else if !movement::contains_coord(
                    ss.moves.iter().map(|x| &x.target),
                    &target_cell,
                ) {
                    //If we select an empty cell outside of our units range.
                    break;
                } else if let Some(k) = extra_attack {
                    //If we are in extra attack mode, forbid the user from moving
                    //any unit other than the one that has already done a partial move.
                    if let TeamType::ThisTeam(a) = current_warrior_pos {
                        if a.inner != k {
                            break;
                        }
                    }
                }

                //At this point all re-selecting of units based off of the input has occured.
                //We definately want to act on the action the user took on the selected unit.

                //Reconstruct path by creating all possible paths with path information this time.
                let path = get_path_from_move(target_cell, &unit, &game, extra_attack);

                if let Some(target_coord) = game.that_team.find_slow_mut(&target_cell)
                {
                    
                    let target_coord=target_coord.as_ref().slim();
                    //If we are moving ontop of an enemy.
                
                    let d = game.that_team.lookup_take(target_coord);
                    let c = game
                        .this_team
                        .lookup_take(current_warrior_pos.unwrap_this());

                    match doop
                        .await_data(team_index)
                        .resolve_attack(c, d, false, &path)
                        .await
                    {
                        unit::Pair(Some(a), None) => {
                            
                            game.this_team.add(a);

                            let _ = doop
                                .await_data(team_index.not())
                                .resolve_group_attack(
                                    target_cell.to_cube(),
                                    game.that_team,
                                    game.this_team,
                                )
                                .await;

                            //TODO is this possible?
                            for n in target_cell.to_cube().neighbours() {
                                doop.await_data(team_index)
                                    .resolve_group_attack(n, game.this_team, game.that_team)
                                    .await;
                            }

                            //Finish this players turn.
                            break 'select;
                        }
                        _ => unreachable!(),
                    }
                } else {
                    //If we are moving to an empty square.

                    let this_unit = game
                        .this_team
                        .lookup_take(current_warrior_pos.unwrap_this());

                    let this_unit = doop
                        .await_data(team_index)
                        .resolve_movement(this_unit, path)
                        .await;

                    //current_warrior_pos = TeamType::ThisTeam(this_unit.as_ref().slim());

                    game.this_team.add(this_unit);

                    //TODO use an enum to team index
                    let k = doop
                        .await_data(team_index.not())
                        .resolve_group_attack(target_cell.to_cube(), game.that_team, game.this_team)
                        .await;

                    //Need to add ourselves back so we can resolve and attacking groups
                    //only to remove ourselves again later.
                    let k = if let Some(k) = k {
                        let j = k.as_ref().slim();

                        game.this_team.add(k);

                        for n in target_cell.to_cube().neighbours() {
                            doop.await_data(team_index)
                                .resolve_group_attack(n, game.this_team, game.that_team)
                                .await;
                        }

                        Some(game.this_team.lookup_take(j))
                    } else {
                        for n in target_cell.to_cube().neighbours() {
                            doop.await_data(team_index)
                                .resolve_group_attack(n, game.this_team, game.that_team)
                                .await;
                        }
                        None
                    };

                    if let Some(mut k) = k {
                        k.stamina.0 = k.as_ref().get_movement_data();
                        extra_attack = Some(target_cell);

                        current_warrior_pos = TeamType::ThisTeam(k.as_ref().slim());
                        game.this_team.add(k);
                        //TODO allow the user to move this unit one more time jumping.
                        //So the user must move the unit, or it will die.
                    } else {
                        //Finish this players turn.
                        break 'select;
                    }
                }
            }
        }

        for a in game.this_team.warriors.iter_mut() {
            for b in a.elem.iter_mut() {
                b.resting = 0.max(b.resting - 1);
            }
        }
        game.this_team.replenish_stamina();

        //team_index = team_index.not();
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

pub fn get_path_from_move(
    target_cell: GridCoord,
    unit: &WarriorType<&UnitData>,
    game: &GameView,
    extra_attack: Option<GridCoord>,
) -> movement::Path {
    //Reconstruct possible paths with path information this time.
    let ss = generate_unit_possible_moves2(&unit, game, extra_attack, movement::WithPath);

    let path = ss
        .moves
        .iter()
        .find(|a| a.target == target_cell)
        .map(|a| &a.path)
        .unwrap();

    *path
}
pub fn generate_unit_possible_moves2<P: movement::PathHave>(
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
        unit.stamina.0
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
