use crate::{
    animation::{self, Animation},
    grids::GridMatrix,
    movement::{self, GridCoord},
    CellSelection, Game, Tribe, Warrior, WarriorPointer,
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

pub struct Doop<'a> {
    sender: Sender<GameWrap<'a, Command>>,
    receiver: Receiver<GameWrap<'a, Response>>,
}
impl<'a> Doop<'a> {
    async fn wait_animation(
        &mut self,
        animation: Animation<WarriorPointer<Warrior>>,
        game: &mut GameHolder<'a>,
    ) -> Animation<WarriorPointer<Warrior>> {
        self.sender
            .send(GameWrap {
                game: game.game.take().unwrap(),
                data: Command::Animate(animation),
            })
            .await
            .unwrap();

        let GameWrap { game: gg, data } = self.receiver.next().await.unwrap();

        let Response::AnimationFinish(o)=data else{
            unreachable!();
        };

        game.game = Some(gg);
        o
    }
    async fn get_mouse(&mut self, game: &mut GameHolder<'a>) -> [f32; 2] {
        self.sender
            .send(GameWrap {
                game: game.game.take().unwrap(),
                data: Command::GetMouseInput,
            })
            .await
            .unwrap();

        let GameWrap { game: gg, data } = self.receiver.next().await.unwrap();

        let Response::Mouse(o)=data else{
            unreachable!();
        };

        game.game = Some(gg);
        o
    }
    async fn get_user_selection(
        &mut self,
        cell: CellSelection,
        game: &mut GameHolder<'a>,
    ) -> (CellSelection, [f32; 2]) {
        self.sender
            .send(GameWrap {
                game: game.game.take().unwrap(),
                data: Command::GetPlayerSelection(cell),
            })
            .await
            .unwrap();

        let GameWrap { game: gg, data } = self.receiver.next().await.unwrap();

        let Response::PlayerSelection(c,o)=data else{
            unreachable!();
        };
        game.game = Some(gg);

        (c, o)
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
    response_recv: Receiver<GameWrap<'a, Response>>,
    game: &'a mut Game,
    grid_matrix: &GridMatrix,
) {
    let mut doop = Doop {
        sender: command_sender,
        receiver: response_recv,
    };

    for _ in 0..4 {
        game.cats.replenish_stamina();
        game.dogs.replenish_stamina();
    }

    let mut game = GameHolder {
        game: Some(game),
        team_index: 0,
    };

    loop {
        let (current_unit,view) = loop {
            let mouse_world = doop.get_mouse(&mut game).await;
            let view = game.get_view();

            let cell: GridCoord = GridCoord(grid_matrix.to_grid((mouse_world).into()).into());

            let Some(unit)= view.this_team.find_slow(&cell) else {
                continue;
            };

            if !unit.selectable() {
                continue;
            }

            let pos = unit.slim();
            break (pos,view);
        };

        
        let unit = view.this_team.lookup(current_unit);

        let cc = crate::state::generate_unit_possible_moves2(
            &unit,
            view.this_team,
            view.that_team,
            grid_matrix,
        );

        let (cell, mouse_world) = doop.get_user_selection(cc, &mut game).await;
        let view = game.get_view();

        let (ss, attack) = match cell {
            CellSelection::MoveSelection(ss, attack) => (ss, attack),
            _ => {
                unreachable!()
            }
        };

        //This is the cell the user selected from the pool of available moves for the unit
        let target_cell: GridCoord = GridCoord(grid_matrix.to_grid((mouse_world).into()).into());

        let target_cat_pos = &target_cell;

        let xx = view.this_team.lookup(current_unit).slim();

        let current_attack = view.this_team.lookup_mut(&xx).attacked;

        let aa = if let Some(aaa) = view.that_team.find_slow(target_cat_pos) {
            let aaa = aaa.slim();

            if !current_attack && movement::contains_coord(attack.iter_coords(), target_cat_pos) {
                //TODO attack aaa
            } else {
                //TODO
            }
        } else if movement::contains_coord(ss.iter_coords(), &target_cell) {
            let (dd, _) = ss.get_path_data(&target_cell).unwrap();
            let start = view.this_team.lookup_take(current_unit);

            let aa = animation::Animation::new(start.position, dd, grid_matrix, start);

            let aa = doop.wait_animation(aa, &mut game).await;
            let view = game.get_view();

            let mut warrior = aa.into_data();
            warrior.stamina.0 -= dd.total_cost().0;
            warrior.position = target_cell;

            //Add it back!
            
            view.this_team.add(warrior);
        } else {
            let va = view.this_team.find_slow(&target_cell).and_then(|a| {
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
