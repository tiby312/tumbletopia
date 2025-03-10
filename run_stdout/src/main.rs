
fn main() {
    for _ in 0..30{
        test_run();
    }
}



pub fn test_run(){

    let world=&engine::board::MyWorld::load_from_string("bb-t-bbsrd-s----s--");

    let mut game_history=engine::MoveHistory::new();
    let mut game=world.starting_state.clone();

    let mut team_iter=engine::Team::White.iter();
    let foo=loop{
        let team=team_iter.next().unwrap();
        if let Some(foo)=game.tactical.game_is_over(world,team,&game_history){
            break foo;
        }
        let mut ai_state = game.tactical.bake_fog(&game.fog[team]);
        let m=engine::ai::calculate_move(&mut ai_state, &game.fog, world, team, &game_history);
        //println!("team {:?} made move {:?}",team,&world.format(&m));
        let effect=m.apply(team, &mut game.tactical, &game.fog[team], world);
        game_history.push((m,effect));

    };
    //
    let history:Vec<_>=game_history.inner.iter().map(|(x,_)|x.clone()).collect();

    let s=format!("{:?}",world.format(&history));
    assert_eq!(s,"[E3,D3,E4,B2,D2,C2,B1,D3,B2,D4,C2,C3,D3,D5,E3,C4,D2,B3,C1,C4,E4,C5,C2,D5,B1,C4,B2,B4,A1,A3,pp,pp,]");

    let engine::unit::GameOver::WhiteWon=foo else{
        panic!("Foo")
    };
    //println!("Result {:?},Game history {:?}",foo,world.format(&history));


}