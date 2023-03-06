use crate::{
    animation::Animation, grids::GridMatrix, movement::GridCoord, CellSelection, Game, Warrior,
};

pub struct GameWrap<'a, T> {
    pub game: &'a mut Game,
    pub data: T,
}

#[derive(Debug)]
pub enum Command {
    Animate(Animation<Warrior>),
    GetMouseInput,
    GetPlayerSelection(CellSelection),
    Nothing,
}
impl Command {
    pub fn take_animation(&mut self) -> Animation<Warrior> {
        todo!()
    }
}

#[derive(Debug)]
pub enum Response {
    Mouse([f32; 2]), //TODO make grid coord
    AnimationFinish(Animation<Warrior>),
    PlayerSelection([f32; 2]), //TODO make grid coord
}
pub struct RendererFacingEngine {}
impl RendererFacingEngine {
    pub async fn await_command(&mut self) -> Command {
        todo!();
    }
}

pub struct LogicFacingEngine {}
impl LogicFacingEngine {
    pub async fn animate<T>(&mut self, a: T, b: Animation<Warrior>) -> (T, Animation<Warrior>) {
        todo!()
    }
    pub async fn wait_mouse_input<T>(&mut self, a: T) -> (T, [f32; 2]) {
        todo!()
    }

    pub async fn wait_button_press<T>(&mut self, a: T) -> (T, [f32; 2]) {
        todo!()
    }
}

use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};
use gloo::console::log;

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

    *game=Some(gg);
    o
}

async fn get_user_selection<'a>(
    cell: CellSelection,
    game: &mut Option<&'a mut Game>,
    sender: &mut Sender<GameWrap<'a, Command>>,
    recv: &mut Receiver<GameWrap<'a, Response>>,
) -> [f32; 2] {
    sender
        .send(GameWrap {
            game: game.take().unwrap(),
            data: Command::GetPlayerSelection(cell),
        })
        .await
        .unwrap();

    let GameWrap { game: gg, data } = recv.next().await.unwrap();

    let Response::PlayerSelection(o)=data else{
        unreachable!();
    };
    *game=Some(gg);

    o
}

pub async fn main_logic<'a>(
    mut command_sender: Sender<GameWrap<'a, Command>>,
    mut response_recv: Receiver<GameWrap<'a, Response>>,
    game: &'a mut Game,
    grid_matrix: &GridMatrix,
) {
    let team_index = 0;
    let mut game = Some(game);

    let pos = loop {
        let mouse_world = get_mouse(
            &mut game,
            &mut command_sender,
            &mut response_recv,
        )
        .await;

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

    let unit = this_team.lookup(pos);
    let cc = crate::state::generate_unit_possible_moves2(&unit, this_team, that_team, grid_matrix);

    let mouse_world = get_user_selection(
        cc,
        &mut game,
        &mut command_sender,
        &mut response_recv,
    )
    .await;

    log!(format!("User selected!={:?}", mouse_world));
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
