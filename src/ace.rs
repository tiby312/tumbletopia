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

#[derive(Debug)]
pub enum Command {
    Animate(Animation<WarriorType<UnitData>>),
    GetMouseInput(Option<CellSelection>),
    Nothing,
}
impl Command {
    pub fn take_animation(&mut self) -> Animation<WarriorType<UnitData>> {
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

#[derive(Debug)]
pub enum Response {
    Mouse(Option<CellSelection>, Pototo<GridCoord>), //TODO make grid coord
    AnimationFinish(Animation<WarriorType<UnitData>>),
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
        animation: Animation<WarriorType<UnitData>>,
        team_index: usize,
    ) -> Animation<WarriorType<UnitData>> {
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
        let (_, c) = self.get_mouse(None, team_index).await;
        c
    }
    async fn get_mouse_selection<'c>(
        &mut self,
        cell: CellSelection,
        team_index: usize,
    ) -> (CellSelection, Pototo<GridCoord>) {
        let (b, c) = self.get_mouse(Some(cell), team_index).await;
        (b.unwrap(), c)
    }

    async fn get_mouse<'c>(
        &mut self,
        cell: Option<CellSelection>,
        team_index: usize,
    ) -> (Option<CellSelection>, Pototo<GridCoord>) {
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

    let mut team_index = 0;
    //Loop over each team!
    loop {
        let (this_team, that_team) = if team_index == 0 {
            (&mut game.cats, &mut game.dogs)
        } else {
            (&mut game.dogs, &mut game.cats)
        };

        this_team.replenish_stamina();
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

                let Some(unit)= this_team.find_slow(&cell) else {
                    continue;
                };
                let pos = unit.slim();

                let ss = unit.calculate_selectable(this_team, that_team, grid_matrix);

                this_team.lookup_mut(&pos).selectable = ss;
                if !ss {
                    continue;
                }

                break pos;
            };

            let mut current_warrior_pos = current_unit;
            //Keep showing the selected unit's options and keep handling the users selections
            //Until the unit is deselected.
            loop {
                //let view = game.get_view();
                let unit = this_team.lookup(current_warrior_pos);

                let cc = generate_unit_possible_moves2(&unit, this_team, that_team, grid_matrix);

                let (cell, pototo) = doop.get_mouse_selection(cc, team_index).await;
                let mouse_world = match pototo {
                    Pototo::Normal(t) => t,
                    Pototo::EndTurn => {
                        //End the turn. Ok because we are not int he middle of anything.
                        break 'outer;
                    }
                };

                //This is the cell the user selected from the pool of available moves for the unit
                let target_cell = mouse_world;
                let (ss, attack_data, attack) = match cell {
                    CellSelection::MoveSelection(ss, attack_data, attack) => {
                        (ss, attack_data, attack)
                    }
                    _ => {
                        unreachable!()
                    }
                };

                let xx = this_team.lookup(current_warrior_pos).slim();

                let current_attack = this_team.lookup_mut(&xx).attacked;

                if let Some(target) = that_team.find_slow(&target_cell) {
                    let aaa = target.slim();

                    if !current_attack && movement::contains_coord(attack.iter(), &target_cell) {
                        let c = this_team.lookup_take(current_warrior_pos);
                        let d = that_team.lookup_take(aaa);

                        let j = doop
                            .await_data(grid_matrix, team_index)
                            .resolve_attack(c, d)
                            .await;
                        match j {
                            unit::Pair(Some(a), None) => {
                                current_warrior_pos = a.as_ref().slim();

                                this_team.add(a);
                            }
                            unit::Pair(None, Some(a)) => {
                                that_team.add(a);
                                //Deselect unit because it died.
                                break;
                            }
                            unit::Pair(Some(a), Some(b)) => {
                                current_warrior_pos = a.as_ref().slim();

                                this_team.add(a);
                                that_team.add(b)
                            }
                            unit::Pair(None, None) => {
                                unreachable!();
                            }
                        }
                    } else {
                        //Deselect
                        break;
                    }
                } else if movement::contains_coord(ss.iter_coords(), &target_cell) {
                    let (path, _) = ss.get_path_data(&target_cell).unwrap();
                    let this_unit = this_team.lookup_take(current_warrior_pos);

                    let this_unit = doop
                        .await_data(grid_matrix, team_index)
                        .resolve_movement(this_unit, path)
                        .await;

                    current_warrior_pos = this_unit.as_ref().slim();

                    this_team.add(this_unit);
                } else {
                    if let Some(a) = this_team.find_slow(&target_cell) {
                        let vv = a.calculate_selectable(this_team, that_team, grid_matrix);
                        let k = a.slim();

                        this_team.lookup_mut(&k).selectable = vv;

                        if vv && k != current_warrior_pos {
                            //Quick switch to another unit
                            current_warrior_pos = k;
                        } else {
                            //Deselect
                            break;
                        }
                    } else {
                        //Deselect
                        break;
                    }
                };

                //let view = game.get_view();

                let wwa = this_team.lookup(current_warrior_pos);
                let vv = wwa.calculate_selectable(this_team, that_team, grid_matrix);
                let mut wwa = this_team.lookup_mut(&current_warrior_pos);
                wwa.selectable = vv;

                if !vv {
                    //Deselect
                    break;
                }
            }

            //log!(format!("User selected!={:?}", mouse_world));
        }

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
    fn get_cat_move_attack_matrix(
        movement: (i8, i8),
        cat: &WarriorType<&UnitData>,
        cat_filter: impl Filter,
        roads: impl MoveCost,
        gg: &grids::GridMatrix,
        _moved: bool,
    ) -> CellSelection {
        let (_movement, attack) = movement;
        let mm = if !cat.attacked {
            //TODO make this more explicit somehow??
            // MoveUnit(if cat.stamina.0 % 2 ==0{
            //     cat.stamina.0-1
            // }else{
            //     cat.stamina.0
            // })
            MoveUnit(cat.stamina.0)
        } else {
            MoveUnit(0)
        };

        let mm = movement::PossibleMoves::new(
            &movement::WarriorMovement,
            &gg.filter().chain(cat_filter),
            &terrain::Grass.chain(roads),
            cat.position,
            mm,
        );

        let attack_range = if !cat.attacked { attack } else { 0 };

        //let attack_range=attack;

        let attack_coords = cat.get_attack_data().collect();

        // let attack = movement::PossibleMoves::new(
        //     &movement::WarriorMovement,
        //     &gg.filter().chain(SingleFilter { a: cat.get_pos() }),
        //     &terrain::Grass,
        //     cat.position,
        //     MoveUnit(attack_range),
        // );

        CellSelection::MoveSelection(mm, (), attack_coords)
    }

    let data = unit.get_movement_data();

    get_cat_move_attack_matrix(
        data,
        &unit,
        this_team.filter().chain(that_team.filter()),
        terrain::Grass,
        grid_matrix,
        true,
    )
}
