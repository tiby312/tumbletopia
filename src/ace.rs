use crate::{
    animation::{self, Animation},
    grids::GridMatrix,
    movement::{self, GridCoord},
    CellSelection, Game, Warrior, WarriorPointer,
};

pub struct GameWrap<'a, T> {
    pub game: &'a mut Game,
    pub data: T,
}

#[derive(Debug)]
pub enum Command {
    Animate(Animation<WarriorPointer<Warrior>>),
    GetMouseInput,
    GetPlayerSelection(CellSelection),
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
    pub fn take_selection(&mut self) -> CellSelection {
        let mut a = Command::Nothing;
        std::mem::swap(self, &mut a);

        let Command::GetPlayerSelection(a)=a else{
            panic!();
        };

        a
    }
}

#[derive(Debug)]
pub enum Response {
    Mouse([f32; 2]), //TODO make grid coord
    AnimationFinish(Animation<WarriorPointer<Warrior>>),
    PlayerSelection(CellSelection, [f32; 2]), //TODO make grid coord
}

use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};
use gloo::console::log;

async fn wait_animation<'a>(
    animation: Animation<WarriorPointer<Warrior>>,
    game: &mut Option<&'a mut Game>,
    sender: &mut Sender<GameWrap<'a, Command>>,
    recv: &mut Receiver<GameWrap<'a, Response>>,
) -> Animation<WarriorPointer<Warrior>> {
    sender
        .send(GameWrap {
            game: game.take().unwrap(),
            data: Command::Animate(animation),
        })
        .await
        .unwrap();

    let GameWrap { game: gg, data } = recv.next().await.unwrap();

    let Response::AnimationFinish(o)=data else{
        unreachable!();
    };

    *game = Some(gg);
    o
}

async fn get_mouse<'a>(
    game: &mut Option<&'a mut Game>,
    sender: &mut Sender<GameWrap<'a, Command>>,
    recv: &mut Receiver<GameWrap<'a, Response>>,
) -> [f32; 2] {
    sender
        .send(GameWrap {
            game: game.take().unwrap(),
            data: Command::GetMouseInput,
        })
        .await
        .unwrap();

    let GameWrap { game: gg, data } = recv.next().await.unwrap();

    let Response::Mouse(o)=data else{
        unreachable!();
    };

    *game = Some(gg);
    o
}

async fn get_user_selection<'a>(
    cell: CellSelection,
    game: &mut Option<&'a mut Game>,
    sender: &mut Sender<GameWrap<'a, Command>>,
    recv: &mut Receiver<GameWrap<'a, Response>>,
) -> (CellSelection, [f32; 2]) {
    sender
        .send(GameWrap {
            game: game.take().unwrap(),
            data: Command::GetPlayerSelection(cell),
        })
        .await
        .unwrap();

    let GameWrap { game: gg, data } = recv.next().await.unwrap();

    let Response::PlayerSelection(c,o)=data else{
        unreachable!();
    };
    *game = Some(gg);

    (c, o)
}

pub async fn main_logic<'a>(
    mut command_sender: Sender<GameWrap<'a, Command>>,
    mut response_recv: Receiver<GameWrap<'a, Response>>,
    game: &'a mut Game,
    grid_matrix: &GridMatrix,
) {
    for _ in 0..4 {
        game.cats.replenish_stamina();
        game.dogs.replenish_stamina();
    }
    let team_index = 0;
    let mut game = Some(game);

    loop {
        let current_unit = loop {
            let mouse_world = get_mouse(&mut game, &mut command_sender, &mut response_recv).await;

            let gg = game.as_mut().unwrap();

            log!(format!("Got mouse input!={:?}", mouse_world));

            let this_team = if team_index == 0 {
                &mut gg.cats
            } else {
                &mut gg.dogs
            };

            let cell: GridCoord = GridCoord(grid_matrix.to_grid((mouse_world).into()).into());

            let Some(unit)= this_team.find_slow(&cell) else {
            continue;
        };

            if !unit.selectable() {
                continue;
            }

            let pos = unit.slim();
            break pos;
        };

        let gg = game.as_mut().unwrap();

        let (this_team, that_team) = if team_index == 0 {
            (&mut gg.cats, &mut gg.dogs)
        } else {
            (&mut gg.dogs, &mut gg.cats)
        };

        let mut unit = this_team.lookup(current_unit);

        let cc =
            crate::state::generate_unit_possible_moves2(&unit, this_team, that_team, grid_matrix);

        let (cell, mouse_world) =
            get_user_selection(cc, &mut game, &mut command_sender, &mut response_recv).await;

        let gg = game.as_mut().unwrap();

        let (this_team, that_team) = if team_index == 0 {
            (&mut gg.cats, &mut gg.dogs)
        } else {
            (&mut gg.dogs, &mut gg.cats)
        };

        let (ss, attack) = match cell {
            CellSelection::MoveSelection(ss, attack) => (ss, attack),
            _ => {
                unreachable!()
            }
        };

        //This is the cell the user selected from the pool of available moves for the unit
        let target_cell: GridCoord = GridCoord(grid_matrix.to_grid((mouse_world).into()).into());

        let target_cat_pos = &target_cell;

        let xx = this_team.lookup(current_unit).slim();

        let current_attack = this_team.lookup_mut(&xx).attacked;

        let aa = if let Some(aaa) = that_team.find_slow(target_cat_pos) {
            let aaa = aaa.slim();

            if !current_attack && movement::contains_coord(attack.iter_coords(), target_cat_pos) {
                //TODO attack aaa
            } else {
                //TODO
            }
        } else if movement::contains_coord(ss.iter_coords(), &target_cell) {
            let (dd, _) = ss.get_path_data(&target_cell).unwrap();
            let start = this_team.lookup_take(current_unit);

            let aa = animation::Animation::new(start.position, dd, grid_matrix, start);

            let aa = wait_animation(aa, &mut game, &mut command_sender, &mut response_recv).await;
            let mut warrior = aa.into_data();
            warrior.stamina.0 -= dd.total_cost().0;
            warrior.position = target_cell;

            //Add it back!

            let gg = game.as_mut().unwrap();

            let (this_team, that_team) = if team_index == 0 {
                (&mut gg.cats, &mut gg.dogs)
            } else {
                (&mut gg.dogs, &mut gg.cats)
            };
            this_team.add(warrior);
        } else {
            let va = this_team.find_slow(&target_cell).and_then(|a| {
                if a.selectable() && a.slim() != current_unit {
                    //TODO quick switch to another unit!!!!!
                    //Some(a)
                    Some(a)
                } else {
                    None
                    //None
                }
            });
            //Deselect?
        };

        log!(format!("User selected!={:?}", mouse_world));
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
