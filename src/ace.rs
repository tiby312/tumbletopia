use super::*;

use crate::{
    animation::{self, Animation},
    grids::{self, GridMatrix},
    movement::{self, Filter, GridCoord, MoveUnit},
    terrain::{self, MoveCost},
    CellSelection, Game, HasPos, SingleFilter, Tribe, UnitData, WarriorType,
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
    Friendly(CellSelection),
    Enemy(CellSelection),
    None,
}

#[derive(Debug)]
pub enum Command {
    Animate(Animation<AnimationOptions>),
    GetMouseInput(MousePrompt),
    Nothing,
}
impl Command {
    pub fn take_animation(&mut self) -> Animation<AnimationOptions> {
        let mut a = Command::Nothing;
        std::mem::swap(self, &mut a);

        let Command::Animate(a)=a else{
            panic!();
        };

        a
    }

    pub fn take_cell(&mut self) -> MousePrompt {
        let mut a = Command::Nothing;
        std::mem::swap(self, &mut a);

        let Command::GetMouseInput(a)=a else{
            panic!();
        };

        a
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
use gloo::console::log;

pub struct Doop<'a> {
    game: *mut Game,
    sender: Sender<GameWrap<'a, Command>>,
    receiver: Receiver<GameWrapResponse<'a, Response>>,
}
impl<'a> Doop<'a> {
    pub fn await_data<'b>(
        &'b mut self,
        grid_matrix: &'b GridMatrix,
        team_index: usize,
    ) -> AwaitData<'a, 'b> {
        AwaitData::new(self, grid_matrix, team_index)
    }

    pub async fn wait_animation<'c>(
        &mut self,
        animation: Animation<AnimationOptions>,
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

        let GameWrapResponse { game: gg, data } = self.receiver.next().await.unwrap();
        let Response::AnimationFinish(o)=data else{
            unreachable!();
        };

        o
    }

    async fn get_mouse_no_selection<'c>(&mut self, team_index: usize) -> Pototo<GridCoord> {
        let (_, c) = self.get_mouse(MousePrompt::None, team_index).await;
        c
    }
    async fn get_mouse_selection_friendly<'c>(
        &mut self,
        cell: CellSelection,
        team_index: usize,
    ) -> (CellSelection, Pototo<GridCoord>) {
        let (b, c) = self
            .get_mouse(MousePrompt::Friendly(cell), team_index)
            .await;

        let MousePrompt::Friendly(b)=b else{
            unreachable!()
        };

        (b, c)
    }

    async fn get_mouse_selection_enemy<'c>(
        &mut self,
        cell: CellSelection,
        team_index: usize,
    ) -> (CellSelection, Pototo<GridCoord>) {
        let (b, c) = self.get_mouse(MousePrompt::Enemy(cell), team_index).await;

        let MousePrompt::Enemy(b)=b else{
            unreachable!()
        };

        (b, c)
    }

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

        let GameWrapResponse { game: gg, data } = self.receiver.next().await.unwrap();

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
    grid_matrix: &GridMatrix,
) {
    let mut doop = Doop {
        game: game as *mut _,
        sender: command_sender,
        receiver: response_recv,
    };

    {
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

        this_team.calculate_selectable_all(that_team, grid_matrix);

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

                        let cc =
                            generate_unit_possible_moves2(&unit, that_team, this_team, grid_matrix);

                        let (_, pototo) = doop.get_mouse_selection_enemy(cc, team_index).await;
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

                //let view = game.get_view();
                let unit = this_team.lookup(current_warrior_pos.unwrap_this());

                {
                    //TODO only need to calculate when we mutate?
                    let k = unit.calculate_selectable(this_team, that_team, grid_matrix);
                    if !k {
                        this_team
                            .lookup_mut(&current_warrior_pos.unwrap_this())
                            .selectable = k;
                        //Deselect not selecftable!!!
                        break;
                    }
                }

                let cc = generate_unit_possible_moves2(&unit, this_team, that_team, grid_matrix);

                let (cell, pototo) = doop.get_mouse_selection_friendly(cc, team_index).await;
                let mouse_world = match pototo {
                    Pototo::Normal(t) => t,
                    Pototo::EndTurn => {
                        //End the turn. Ok because we are not int he middle of anything.
                        break 'outer;
                    }
                };

                //This is the cell the user selected from the pool of available moves for the unit
                let target_cell = mouse_world;
                let (ss, friendly, attack) = match cell {
                    CellSelection::MoveSelection(ss, friendly, attack) => (ss, friendly, attack),
                    _ => {
                        unreachable!()
                    }
                };

                let xx = this_team.lookup(current_warrior_pos.unwrap_this()).slim();

                let current_attack = this_team.lookup_mut(&xx).attacked;

                if let Some(target) = that_team.find_slow(&target_cell) {
                    let target_coord = target.slim();

                    if !current_attack && movement::contains_coord(attack.iter(), &target_cell) {
                        let d = that_team.lookup_take(target_coord);

                        //Attack using supports.
                        // let d = {
                        //     let v: Vec<_> = this_team
                        //         .other_units_in_range_of_target(target_cell, grid_matrix)
                        //         .map(|x| x.slim())
                        //         .filter(|&f| f != xx)
                        //         .collect();

                        //     let mut d = Some(d);
                        //     for a in v {
                        //         let damage = 1;
                        //         if d.as_ref().unwrap().health <= damage {
                        //             break;
                        //         }
                        //         let warrir = this_team.lookup_take(a);

                        //         match doop
                        //             .await_data(grid_matrix, team_index)
                        //             .resolve_attack(warrir, d.take().unwrap(), true)
                        //             .await
                        //         {
                        //             unit::Pair(Some(a), Some(b)) => {
                        //                 this_team.add(a);
                        //                 d = Some(b);
                        //             }
                        //             _ => {
                        //                 unreachable!()
                        //             }
                        //         }
                        //     }
                        //     d.unwrap()
                        // };

                        let c = this_team.lookup_take(current_warrior_pos.unwrap_this());

                        match doop
                            .await_data(grid_matrix, team_index)
                            .resolve_attack(c, d, false)
                            .await
                        {
                            unit::Pair(Some(a), None) => {
                                current_warrior_pos = TeamType::ThisTeam(a.as_ref().slim());

                                this_team.add(a);
                            }
                            unit::Pair(None, Some(a)) => {
                                that_team.add(a);
                                //Deselect unit because it died.
                                break;
                            }
                            unit::Pair(Some(a), Some(b)) => {
                                current_warrior_pos = TeamType::ThisTeam(a.as_ref().slim());

                                this_team.add(a);
                                that_team.add(b)
                            }
                            unit::Pair(None, None) => {
                                unreachable!();
                            }
                        }
                    } else {
                        current_warrior_pos = TeamType::ThatTeam(target_coord);
                        continue;
                    }
                } else if movement::contains_coord(ss.iter_coords(), &target_cell) {
                    let (path, _) = ss.get_path_data(&target_cell).unwrap();
                    let this_unit = this_team.lookup_take(current_warrior_pos.unwrap_this());

                    let this_unit = doop
                        .await_data(grid_matrix, team_index)
                        .resolve_movement(this_unit, path)
                        .await;

                    current_warrior_pos = TeamType::ThisTeam(this_unit.as_ref().slim());

                    this_team.add(this_unit);
                } else if let Some(a) = this_team.find_slow(&target_cell) {
                    // if !current_attack && movement::contains_coord(friendly.iter(), &target_cell) {
                    //     let target_coord = a.slim();

                    //     let c = this_team.lookup_take(current_warrior_pos);

                    //     let d = this_team.lookup_take(target_coord);

                    //     let unit::Pair(Some(a),Some(b))= doop.await_data(grid_matrix,team_index).resolve_heal(c, d).await else{
                    //             unreachable!()
                    //         };

                    //     this_team.add(a);
                    //     this_team.add(b)
                    // }
                    // else
                    {
                        let vv = a.calculate_selectable(this_team, that_team, grid_matrix);
                        let k = a.slim();

                        this_team.lookup_mut(&k).selectable = vv;

                        if vv && k != current_warrior_pos.unwrap_this() {
                            //Quick switch to another unit
                            current_warrior_pos = TeamType::ThisTeam(k);
                        } else {
                            //Deselect
                            break;
                        }
                    }
                } else {
                    //Deselect
                    break;
                }

                //let view = game.get_view();

                let wwa = this_team.lookup(current_warrior_pos.unwrap_this());
                let vv = wwa.calculate_selectable(this_team, that_team, grid_matrix);
                let mut wwa = this_team.lookup_mut(&current_warrior_pos.unwrap_this());
                wwa.selectable = vv;

                if !vv {
                    //FInish turn
                    break 'outer;
                }
            }

            //log!(format!("User selected!={:?}", mouse_world));
        }

        //Loop through healers and apply healing.
        let mages: Vec<_> = this_team.warriors[Type::Mage.type_index()]
            .elem
            .iter()
            .map(|x| WarriorType {
                inner: x.position,
                val: Type::Mage,
            })
            .collect();

        for unit in mages.iter() {
            let mut a = this_team.lookup_mut(unit);

            if a.health < 2 {
                a.health = 2.min(a.health + 1)
            }
        }
        for unit in mages.iter() {
            let unit = this_team.lookup(*unit);

            let cc = generate_unit_possible_moves2(&unit, this_team, that_team, grid_matrix);
            let (_, friendly, attack) = match cc {
                CellSelection::MoveSelection(ss, friendly, attack) => (ss, friendly, attack),
                _ => {
                    unreachable!()
                }
            };

            assert!(attack.is_empty());

            for a in friendly {
                if let Some(mut a) = this_team.find_slow_mut(&a) {
                    if a.health < 2 {
                        a.health = 2.min(a.health + 1)
                    }
                }
            }
        }
        this_team.replenish_stamina();
        this_team.reset_attacked();

        if team_index == 1 {
            team_index = 0;
        } else {
            team_index = 1;
        }
    }
}

// async fn attack_enimate<'a>(game:&'a mut Game,engine:&mut LogicFacingEngine)->&'a mut Game{
//     let (a,b)=engine.animate(game,Animation).await;
//     //Do something here with warrior!!!
//     a
// }
// async fn doop(engine:&mut LogicFacingEngine){
//     if warrior.health<5{
//         e
//         killanimator.await
//     }else{
//         moveanimator.await
//     }
// }

pub fn generate_unit_possible_moves2(
    unit: &WarriorType<&UnitData>,
    this_team: &Tribe,
    that_team: &Tribe,
    grid_matrix: &GridMatrix,
) -> CellSelection {
    let mm = MoveUnit(unit.stamina.0);

    let mm = movement::PossibleMoves::new(
        &movement::WarriorMovement,
        &grid_matrix
            .filter()
            .chain(this_team.filter())
            .chain(that_team.filter()),
        &terrain::Grass,
        unit.position,
        mm,
    );

    let friendly_coords = unit
        .get_friendly_data()
        .filter(|a| that_team.filter().chain(grid_matrix.filter()).filter(a))
        .collect();

    //TODO don't collect.
    let attack_coords = unit
        .get_attack_data(&this_team.filter().chain(grid_matrix.filter()))
        .collect();

    CellSelection::MoveSelection(mm, friendly_coords, attack_coords)
}
