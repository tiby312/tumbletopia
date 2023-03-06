use crate::{CellSelection, movement::GridCoord, animation::Animation, Warrior, Game};






pub struct GameWrap<'a,T>{
    pub game:&'a mut Game,
    pub data:T
}


#[derive(Debug)]
pub enum Command{
    Animate(Animation<Warrior>),
    GetMouseInput,
    GetPlayerSelection(CellSelection),
    Nothing
}
impl Command{
    pub fn take_animation(&mut self)->Animation<Warrior>{
        todo!()
    }
}

#[derive(Debug)]
pub enum Response{
    Mouse([f32;2]), //TODO make grid coord
    AnimationFinish(Animation<Warrior>),
    PlayerSelection([f32;2]) //TODO make grid coord
}
pub struct RendererFacingEngine{

}
impl RendererFacingEngine{
    pub async fn await_command(&mut self)->Command{
        todo!();
    }
}

pub struct LogicFacingEngine{

}
impl LogicFacingEngine{
    pub async fn animate<T>(&mut self,a:T,b:Animation<Warrior>)->(T,Animation<Warrior>){
        todo!()
    }
    pub async fn wait_mouse_input<T>(&mut self,a:T)->(T,[f32;2]){
    
        todo!()
    
    }

    pub async fn wait_button_press<T>(&mut self,a:T)->(T,[f32;2]){
    
        todo!()
    
    }
    
}

use futures::{channel::mpsc::{Sender,Receiver}, SinkExt, StreamExt};
use gloo::console::log;
pub async fn main_logic<'a>(mut command_sender:Sender<GameWrap<'a,Command>>,mut response_recv:Receiver<GameWrap<'a,Response>>,game:&'a mut Game){
    let mut game=Some(game);
    loop{
        command_sender.send(GameWrap{game:game.take().unwrap(),data:Command::GetMouseInput}).await.unwrap();
        let GameWrap{game:gg,data}=response_recv.next().await.unwrap();
        game=Some(gg);

        log!(format!("Got mouse input!={:?}",data));
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