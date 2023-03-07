use crate::{
    animation::{self, Animation},
    grids::GridMatrix,
    movement::{self, GridCoord},
    CellSelection, Game, Tribe, Warrior, WarriorPointer,
};

pub struct GameWrap<'a, T> {
    pub game: &'a mut Game,
    pub team: usize,
    pub data: T,
}

pub struct GameWrapResponse<'a, T> {
    pub game: &'a mut Game,
    pub data: T,
}

#[derive(Debug)]
pub enum Command {
    Animate(Animation<WarriorPointer<Warrior>>),
    GetMouseInput(Option<CellSelection>),
    Nothing,
}
impl Command {
    pub fn take_animation(&mut self) -> Animation<WarriorPointer<Warrior>> {
        let mut a = Command::Nothing;
        std::mem::swap(self, &mut a);

        let Command::Animate(a)=a else{
            panic!();
        };

        a
    }

    pub fn take_cell(&mut self) -> Option<CellSelection> {
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
impl<T> Pototo<T> {
    fn unwrap(self) -> T {
        match self {
            Pototo::Normal(a) => a,
            Pototo::EndTurn => {
                unreachable!();
            }
        }
    }
}

#[derive(Debug)]
pub enum Response {
    Mouse(Option<CellSelection>, Pototo<GridCoord>), //TODO make grid coord
    AnimationFinish(Animation<WarriorPointer<Warrior>>),
}

use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};
use gloo::console::log;

pub struct Doop<'a> {
    sender: Sender<GameWrap<'a, Command>>,
    receiver: Receiver<GameWrapResponse<'a, Response>>,
}
impl<'a> Doop<'a> {
    async fn wait_animation<'c>(
        &mut self,
        animation: Animation<WarriorPointer<Warrior>>,
        game: &'c mut GameHolder<'a>,
    ) -> (GameView<'c>, Animation<WarriorPointer<Warrior>>) {
        self.sender
            .send(GameWrap {
                team: game.team_index,
                game: game.game.take().unwrap(),
                data: Command::Animate(animation),
            })
            .await
            .unwrap();

        let GameWrapResponse { game: gg, data } = self.receiver.next().await.unwrap();
        let Response::AnimationFinish(o)=data else{
            unreachable!();
        };

        game.game = Some(gg);
        (game.get_view(), o)
    }

    async fn get_mouse_no_selection<'c>(
        &mut self,
        game: &'c mut GameHolder<'a>,
    ) -> (GameView<'c>, Pototo<GridCoord>) {
        let (a, _, c) = self.get_mouse(None, game).await;
        (a, c)
    }
    async fn get_mouse_selection<'c>(
        &mut self,
        cell: CellSelection,
        game: &'c mut GameHolder<'a>,
    ) -> (GameView<'c>, CellSelection, Pototo<GridCoord>) {
        let (a, b, c) = self.get_mouse(Some(cell), game).await;
        (a, b.unwrap(), c)
    }

    async fn get_mouse<'c>(
        &mut self,
        cell: Option<CellSelection>,
        game: &'c mut GameHolder<'a>,
    ) -> (GameView<'c>, Option<CellSelection>, Pototo<GridCoord>) {
        self.sender
            .send(GameWrap {
                game: game.game.take().unwrap(),
                data: Command::GetMouseInput(cell),
                team: game.team_index,
            })
            .await
            .unwrap();

        let GameWrapResponse { game: gg, data } = self.receiver.next().await.unwrap();

        let Response::Mouse(cell,o)=data else{
            unreachable!();
        };

        game.game = Some(gg);
        (game.get_view(), cell, o)
    }
}

pub struct GameView<'a> {
    this_team: &'a mut Tribe,
    that_team: &'a mut Tribe,
}

pub struct GameHolder<'a> {
    game: Option<&'a mut Game>,
    team_index: usize,
}

impl<'a> GameHolder<'a> {
    fn get_view(&mut self) -> GameView {
        let gg = self.game.as_mut().unwrap();
        let (this_team, that_team) = if self.team_index == 0 {
            (&mut gg.cats, &mut gg.dogs)
        } else {
            (&mut gg.dogs, &mut gg.cats)
        };

        GameView {
            this_team,
            that_team,
        }
    }
}

pub async fn main_logic<'a>(
    command_sender: Sender<GameWrap<'a, Command>>,
    response_recv: Receiver<GameWrapResponse<'a, Response>>,
    game: &'a mut Game,
    grid_matrix: &GridMatrix,
) {
    let mut doop = Doop {
        sender: command_sender,
        receiver: response_recv,
    };

    let mut game = GameHolder {
        game: Some(game),
        team_index: 0,
    };

    loop {
        game.get_view().this_team.replenish_stamina();

        //Keep allowing the user to select units
        'outer: loop {
            //Loop until the user clicks on a selectable unit in their team.
            let current_unit = loop {
                let (view, data) = doop.get_mouse_no_selection(&mut game).await;
                let cell = match data {
                    Pototo::Normal(a) => a,
                    Pototo::EndTurn => {
                        log!("End the turn!");
                        break 'outer;
                    }
                };

                let Some(unit)= view.this_team.find_slow(&cell) else {
                continue;
            };

                if !unit.selectable() {
                    continue;
                }

                let pos = unit.slim();
                break pos;
            };

            let mut current_warrior_pos = current_unit;
            //Keep showing the selected unit's options and keep handling the users selections
            //Until the unit is deselected.
            loop {
                let view = game.get_view();
                let unit = view.this_team.lookup(current_warrior_pos);

                let cc = crate::state::generate_unit_possible_moves2(
                    &unit,
                    view.this_team,
                    view.that_team,
                    grid_matrix,
                );

                let (view, cell, pototo) = doop.get_mouse_selection(cc, &mut game).await;
                let mouse_world = match pototo {
                    Pototo::Normal(t) => t,
                    Pototo::EndTurn => continue, //Ignore
                };

                //This is the cell the user selected from the pool of available moves for the unit
                let target_cell = mouse_world;
                let (ss, attack) = match cell {
                    CellSelection::MoveSelection(ss, attack) => (ss, attack),
                    _ => {
                        unreachable!()
                    }
                };

                let target_cat_pos = &target_cell;

                let xx = view.this_team.lookup(current_warrior_pos).slim();

                let current_attack = view.this_team.lookup_mut(&xx).attacked;

                if let Some(target) = view.that_team.find_slow(target_cat_pos) {
                    let aaa = target.slim();

                    if !current_attack
                        && movement::contains_coord(attack.iter_coords(), target_cat_pos)
                    {
                        //TODO attack aaa

                        //Only counter if non neg
                        // let counter_damage = if g1.this_team.lookup_mut(current).move_bank.0>=0{
                        //     5
                        // }else{
                        //     0
                        // };
                        // let damage = 5;
                        // let counter_damage = 5;
                        let damage = 5;
                        let counter_damage = 5;

                        let kill_self = view.this_team.lookup_mut(&current_warrior_pos).health
                            <= counter_damage;

                        let (path, _) = attack.get_path_data(target_cat_pos).unwrap();

                        //let attack_stamina_cost=2;
                        let total_cost = path.total_cost();
                        log!(format!("total_cost:{:?}", total_cost));
                        if target.health <= damage {
                            let c = view.this_team.lookup_take(current_warrior_pos);

                            let aa = animation::Animation::new(c.position, path, grid_matrix, c);

                            let (view, aa) = doop.wait_animation(aa, &mut game).await;

                            let mut this_unit = aa.into_data();

                            this_unit.position = view.that_team.lookup(aaa).position;

                            view.that_team.lookup_take(aaa);
                            view.this_team.add(this_unit);
                            let mut current_cat = view.this_team.lookup_mut(&aaa);

                            current_cat.attacked = true;
                            current_warrior_pos = current_cat.slim();
                        } else {
                            let c = view.this_team.lookup_take(current_warrior_pos);

                            let aa = animation::Animation::new(c.position, path, grid_matrix, c);
                            let (view, aa) = doop.wait_animation(aa, &mut game).await;

                            let this_unit = aa.into_data();
                            view.this_team.add(this_unit);
                            let mut target_cat = view.that_team.lookup_mut(&aaa);
                            target_cat.health -= damage;

                            let mut current_cat = view.this_team.lookup_mut(&current_warrior_pos);

                            if kill_self {
                                view.this_team.lookup_take(current_warrior_pos);
                            } else {
                                current_cat.attacked = true;
                                current_cat.health -= counter_damage;
                                current_cat.stamina.0 -= total_cost.0;
                                //current_cat.stamina.0 -= attack_stamina_cost;
                            }
                        }
                    } else {
                        //Deselect
                        break;
                    }
                } else if movement::contains_coord(ss.iter_coords(), &target_cell) {
                    let (dd, _) = ss.get_path_data(&target_cell).unwrap();
                    let start = view.this_team.lookup_take(current_warrior_pos);

                    let aa = animation::Animation::new(start.position, dd, grid_matrix, start);

                    let (view, aa) = doop.wait_animation(aa, &mut game).await;

                    let mut warrior = aa.into_data();
                    warrior.stamina.0 -= dd.total_cost().0;
                    warrior.position = target_cell;

                    current_warrior_pos = warrior.slim();

                    view.this_team.add(warrior);
                } else {
                    if let Some(a) = view
                        .this_team
                        .find_slow(&target_cell)
                        .filter(|a| a.selectable() && a.slim() != current_warrior_pos)
                    {
                        //Quick switch to another unit
                        current_warrior_pos = a.slim();
                    } else {
                        //Deselect
                        break;
                    }
                };
            }

            //log!(format!("User selected!={:?}", mouse_world));
        }

        game.get_view().this_team.reset_attacked();

        if game.team_index == 1 {
            game.team_index = 0;
        } else {
            game.team_index = 1;
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
