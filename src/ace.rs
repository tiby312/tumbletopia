use super::*;

use crate::{
    animation::Animation,
    grids::GridMatrix,
    movement::{self, Filter, GridCoord, MoveUnit},
    terrain::{self},
    CellSelection, Game, Tribe, UnitData, WarriorType,
};

pub struct GameWrap<'a, T> {
    pub game: &'a Game,
    pub team: usize,
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
    pub fn await_data<'b>(&'b mut self, team_index: usize) -> AwaitData<'a, 'b> {
        AwaitData::new(self, team_index)
    }

    pub async fn wait_animation<'c>(
        &mut self,
        animation: animation::AnimationCommand,
        team_index: usize,
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

    async fn get_mouse_no_selection<'c>(&mut self, team_index: usize) -> Pototo<GridCoord> {
        let (_, c) = self.get_mouse(MousePrompt::None, team_index).await;
        c
    }
    async fn get_mouse_selection<'c>(
        &mut self,
        cell: CellSelection,
        team_index: usize,
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
        team_index: usize,
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

    let mut team_index = 0;
    //Loop over each team!
    loop {
        let (this_team, that_team) = if team_index == 0 {
            (&mut game.cats, &mut game.dogs)
        } else {
            (&mut game.dogs, &mut game.cats)
        };

        //this_team.replenish_health();
        //that_team.replenish_health();

        //this_team.calculate_selectable_all(that_team, grid_matrix);

        let mut extra_attack = None;

        //Keep allowing the user to select units
        'outer: loop {
            //Loop until the user clicks on a selectable unit in their team.
            let current_unit = loop {
                let data = doop.get_mouse_no_selection(team_index).await;
                let cell = match data {
                    Pototo::Normal(a) => a,
                    Pototo::EndTurn => {
                        log!("End the turn!");
                        break 'outer;
                    }
                };

                if let Some(unit) = this_team.find_slow(&cell) {
                    break TeamType::ThisTeam(unit.slim());
                }
                if let Some(unit) = that_team.find_slow(&cell) {
                    break TeamType::ThatTeam(unit.slim());
                }

                //Else we just place a para and exit turn.
                // this_team.add(WarriorType {
                //     inner: UnitData::new(cell),
                //     val: Type::Para,
                // });
                // break 'outer;
            };

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

            let mut current_warrior_pos = current_unit;
            //Keep showing the selected unit's options and keep handling the users selections
            //Until the unit is deselected.
            loop {
                match current_warrior_pos {
                    TeamType::ThisTeam(a) => a,
                    TeamType::ThatTeam(curr_warrior_pos) => {
                        let unit = that_team.lookup(curr_warrior_pos);

                        let cc = generate_unit_possible_moves2(
                            &unit,
                            that_team,
                            this_team,
                            &game.world,
                            None,
                            movement::NoPath,
                        );
                        let cc = CellSelection::MoveSelection(cc);

                        let (_, pototo) = doop.get_mouse_selection(cc, team_index, true).await;
                        let target_cell = match pototo {
                            Pototo::Normal(t) => t,
                            Pototo::EndTurn => {
                                //End the turn. Ok because we are not int he middle of anything.
                                break 'outer;
                            }
                        };

                        if target_cell == *curr_warrior_pos {
                            break;
                        }

                        if let Some(target) = this_team.find_slow(&target_cell) {
                            current_warrior_pos = TeamType::ThisTeam(target.slim());
                            continue;
                        }
                        if let Some(target) = that_team.find_slow(&target_cell) {
                            current_warrior_pos = TeamType::ThatTeam(target.slim());
                            continue;
                        }

                        break;
                    }
                };

                let unit = this_team.lookup(current_warrior_pos.unwrap_this());

                let cc = generate_unit_possible_moves2(
                    &unit,
                    this_team,
                    that_team,
                    &game.world,
                    extra_attack,
                    movement::NoPath,
                );
                let cc = CellSelection::MoveSelection(cc);

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
                        break 'outer;
                    }
                };
                let target_cell = mouse_world;

                //This is the cell the user selected from the pool of available moves for the unit
                let CellSelection::MoveSelection(ss)=cell else{
                    unreachable!()
                };

                //RESELECT STAGE
                if let Some(a) = this_team.find_slow(&target_cell) {
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
                } else if let Some(target) = that_team.find_slow(&target_cell) {
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
                }

                //If we are in extra attack mode, forbid the user from moving
                //any unit other than the one that has already done a partial move.
                if let Some(k) = extra_attack {
                    if let TeamType::ThisTeam(a) = current_warrior_pos {
                        if a.inner != k {
                            break;
                        }
                    }
                }

                //Reconstruct path by creating all possible paths with path information this time.
                let path = get_path_from_move(
                    target_cell,
                    &unit,
                    this_team,
                    that_team,
                    &game.world,
                    extra_attack,
                );

                if let Some(target_coord) = that_team.find_slow(&target_cell).map(|a| a.slim()) {
                    let d = that_team.lookup_take(target_coord);
                    let c = this_team.lookup_take(current_warrior_pos.unwrap_this());

                    match doop
                        .await_data(team_index)
                        .resolve_attack(c, d, false, &path)
                        .await
                    {
                        unit::Pair(Some(a), None) => {
                            current_warrior_pos = TeamType::ThisTeam(a.as_ref().slim());

                            this_team.add(a);

                            let _ = doop
                                .await_data(1 - team_index)
                                .resolve_group_attack(target_cell.to_cube(), that_team, this_team)
                                .await;

                            //TODO is this possible?
                            for n in target_cell.to_cube().neighbours() {
                                doop.await_data(team_index)
                                    .resolve_group_attack(n, this_team, that_team)
                                    .await;
                            }

                            break 'outer;
                        }
                        _ => unreachable!(),
                    }
                } else {
                    let this_unit = this_team.lookup_take(current_warrior_pos.unwrap_this());

                    let this_unit = doop
                        .await_data(team_index)
                        .resolve_movement(this_unit, path)
                        .await;

                    current_warrior_pos = TeamType::ThisTeam(this_unit.as_ref().slim());

                    this_team.add(this_unit);

                    //TODO use an enum to team index
                    let k = doop
                        .await_data(1 - team_index)
                        .resolve_group_attack(target_cell.to_cube(), that_team, this_team)
                        .await;

                    //Need to add ourselves back so we can resolve and attacking groups
                    //only to remove ourselves again later.
                    let k = if let Some(k) = k {
                        let j = k.as_ref().slim();

                        this_team.add(k);

                        for n in target_cell.to_cube().neighbours() {
                            doop.await_data(team_index)
                                .resolve_group_attack(n, this_team, that_team)
                                .await;
                        }

                        Some(this_team.lookup_take(j))
                    } else {
                        for n in target_cell.to_cube().neighbours() {
                            doop.await_data(team_index)
                                .resolve_group_attack(n, this_team, that_team)
                                .await;
                        }
                        None
                    };

                    if let Some(mut k) = k {
                        k.stamina.0 = k.as_ref().get_movement_data();
                        extra_attack = Some(target_cell);

                        current_warrior_pos = TeamType::ThisTeam(k.as_ref().slim());
                        this_team.add(k);
                        //TODO allow the user to move this unit one more time jumping.
                        //So the user must move the unit, or it will die.
                    } else {
                        break 'outer;
                    }
                }
            }
        }

        for a in this_team.warriors.iter_mut() {
            for b in a.elem.iter_mut() {
                b.resting = 0.max(b.resting - 1);
            }
        }
        this_team.replenish_stamina();

        if team_index == 1 {
            team_index = 0;
        } else {
            team_index = 1;
        }
    }
}

pub struct GameState {}
pub struct Engine {}
impl Engine {
    fn play_move(&mut self, a: Move) {}
    fn get_state(&self) -> &GameState {
        todo!()
    }
    fn get_valid_moves(&self, a: GridCoord) -> impl Iterator<Item = Move> {
        std::iter::empty()
    }
}

// pub enum HexDir{

// }

// pub struct WarriorMoveSet{
//     position:GridCoord
// }
// impl Iterator for WarriorMoveSet{
//     type Item=(movement::Path,Option<HexDir>);
// }

// pub enum MoveSet{
//     Warrior{
//         path:movement::Path,
//         extra:Option<HexDir>,
//     },
//     King{
//         path:HexDir
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
    this_team: &Tribe,
    that_team: &Tribe,
    grid_matrix: &board::World,
    extra_attack: Option<GridCoord>,
) -> movement::Path {
    //Reconstruct possible paths with path information this time.
    let ss = generate_unit_possible_moves2(
        &unit,
        this_team,
        that_team,
        grid_matrix,
        extra_attack,
        movement::WithPath,
    );

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
    this_team: &Tribe,
    that_team: &Tribe,
    grid_matrix: &board::World,
    extra_attack: Option<GridCoord>,
    ph: P,
) -> movement::PossibleMoves2<P::Foo> {
    // If there is an enemy near by restrict movement.

    let j = if let Some(_) = unit
        .position
        .to_cube()
        .ring(1)
        .map(|s| that_team.find_slow(&s.to_axial()).is_some())
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
            &grid_matrix.filter().and(that_team.filter()),
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
            &grid_matrix
                .filter()
                .and(that_team.warriors[0].filter().not()),
            &this_team.filter().not(),
            &terrain::Grass,
            unit.position,
            mm,
            true,
            ph,
        )
    };
    mm
}
