use engine::{ai::Evaluator, board::MyWorld, moves::SpokeInfo, Zobrist};

//-r-sgg---wwg---wwwgg-swwwgg-wwwwgwwww

//B5,D6,E6,C6,D6,F7,C6,
fn doop() -> impl Iterator<Item = &'static str> {
    "
-r-sgg---wwg---wwwgg-swwwwg-wwwwgwwww
rrrrfrrrrdfrrfrrrrr
-----s--s-eb-ev-b--
-c-c-s-tct---cs--c-
tc-s-d-re-srces-s--
--brc----dc--r-sr-r
-r---s--rtd-bbb-c--
cs--cs----s---csc--
bs----s--d--c--s--c
ducd-uc-d-ub-dubd-u
test-est--erte-rte-
b-rbbr---k---ds-tds
c---rc-b-uc-r-s--sc
s--cbs---ds---d--bs
c--sc---s-e--ses-s-
rr-dr----d----rr-dr
ssetseteeessssettse
d--sd-d-sdd--s---s-
bcs-d-ss-e--sudtc--
bb---cs--d----s---r
-sr-se--se--se----r
sccrbs--ses--ses-sc
c-b--s-r-ts-ccd----
rc-rr----d----rc-rr
-r-e--rte--r-e--te-
rdsr-ds--dd-r-ds-ds
-bbrr----c-bs-rb-r-
s-s-ddssd-ds-dd-s-s
c-rse-rr-e-ss-e-s-c
t---tt-d---d--dttd-
d--d--t--d-t---dtt-
-t--d--t---t-d-dd-t
--t-td-t-t-dd--d---
-rrs-r---cb---bb---
s--c--s--t---cdbc--
r--c----cs-r--d--b-
s-cr-d---er-rtdt-c-
-se-se--ser-se-t---
--es-e--see--s-e-ss
c--ct-------c-t--cc
--ttd-tt-d---dtdd--
---d-c-s--d-rtst-dc
c--ctc-c-d---ssss--
tddt-t--dt---t-d-d-
t-dt---t-t-d--dtdd-
-tbc--b-s--c----b-t
bbbr-rc--dt----r---
--d---ss---s----bcb
bb-t-bbsrd-s----s--
dcc-surr-f-s--sd-sd
s---s-bdd-tct---b-b
-s--ds---cs-s-ds---
--ddttt------d-e---
-----ds--tdsc--er-t
sdt--ect-----e-sc-s
-tccd--t-t-ct--rdd-
--s---s-cttc-d-cr-c
cs--c----r--b----dr
-r--r-rbbcr---dr-bs
-r--r-r-bct---c--b-
-r--r-r-d-t-c-d--b-
duc----tt---e-b-td-
ccc-e-ss----s-----d
----sc-s-----ccssc-
"
    .trim()
    .split('\n')

    //TODO make sure first player wins in all these cases using the AI.
}

fn main() {
    //eval("dut-s-stt-dcedbbtd-");

    // let start = std::time::Instant::now();

    // for _ in 0..10{
    test_wins();
    // }

    // let elapsed = start.elapsed(); // Get the elapsed time

    // println!("Elapsed time: {:?}", elapsed);

    // test_pass("--t-td-t-t-dd--d---");
    //test_pass("c--ct-------c-t--cc");
    //test_pass("-----ds--tdsc--er-t");

    //play_large();
}

fn test_pass(game_s: &str) {
    let (gg, hist, world) = test_run(game_s);
    let engine::unit::GameOver::WhiteWon = gg else {
        let s = format!("{:?}", world.format(&hist));

        panic!("failed {} hist {:?}", game_s, s);
    };
}

fn eval(game_s: &str) {
    let world = engine::board::MyWorld::load_from_string(game_s);

    let mut game = world.starting_state.clone();

    let mut spoke_info = SpokeInfo::new(&game.tactical);
    engine::moves::update_spoke_info(&mut spoke_info, &world, &game.tactical);
    let eval = Evaluator::default().absolute_evaluate(&game.tactical, &world, &spoke_info, false);
    println!("eval for {} is {} from white perpsective", game_s, eval);
}

fn test_wins() {
    for g in doop() {
        test_pass(g);
        //println!("passed for {}",g);
    }
}

fn play_large() {
    let (gg, hist, world) =
        test_run("---------------------b----r---k------------------------------");
}

fn test_hard_one() {
    let (gg, hist, world) = test_run("c--ct-------c-t--cc");
    matches!(gg, engine::unit::GameOver::WhiteWon);

    let s = format!("{:?}", world.format(&hist));
    assert_eq!(s,"[E3,D3,E4,B2,D2,C2,B1,D3,B2,D4,C2,C3,D3,D5,E3,C4,D2,B3,C1,C4,E4,C5,C2,D5,B1,C4,B2,B4,A1,A3,pp,pp,]");
}

fn bench() {
    for _ in 0..30 {
        let (gg, hist, world) = test_run("bb-t-bbsrd-s----s--");
        matches!(gg, engine::unit::GameOver::WhiteWon);

        let s = format!("{:?}", world.format(&hist));
        assert_eq!(s,"[E3,D3,E4,B2,D2,C2,B1,D3,B2,D4,C2,C3,D3,D5,E3,C4,D2,B3,C1,C4,E4,C5,C2,D5,B1,C4,B2,B4,A1,A3,pp,pp,]");
    }
}

pub fn test_run(
    game: &str,
) -> (
    engine::unit::GameOver,
    Vec<engine::ActualMove>,
    engine::board::MyWorld,
) {
    let world = engine::board::MyWorld::load_from_string(game);

    let mut game_history = engine::MoveHistory::new();
    let mut game = world.starting_state.clone();
    let zobrist = Zobrist::new();
    let mut team_iter = engine::Team::White.iter();
    let foo = loop {
        let team = team_iter.next().unwrap();
        if let Some(foo) = game.tactical.game_is_over(&world, team, &game_history) {
            break foo;
        }
        let mut ai_state = game.tactical.bake_fog(&game.fog[team]);
        let m = engine::ai::calculate_move(
            &mut ai_state,
            &game.fog,
            &world,
            team,
            &game_history,
            &zobrist,
        );
        //panic!();

        //println!("team {:?} made move {:?}",team,&world.format(&m));
        let effect = m.apply(team, &mut game.tactical, &game.fog[team], &world, None);
        game_history.push((m, effect));
    };
    //
    let history: Vec<_> = game_history.inner.iter().map(|(x, _)| x.clone()).collect();

    // let s = format!("{:?}", world.format(&history));
    // assert_eq!(s,"[E3,D3,E4,B2,D2,C2,B1,D3,B2,D4,C2,C3,D3,D5,E3,C4,D2,B3,C1,C4,E4,C5,C2,D5,B1,C4,B2,B4,A1,A3,pp,pp,]");

    // // let engine::unit::GameOver::WhiteWon = foo else {
    // //     panic!("Foo")
    // // };
    //println!("Result {:?},Game history {:?}",foo,world.format(&history));
    (foo, history, world)
}
