use engine::board::MyWorld;
use engine::main_logic as ace;
use engine::main_logic::GameWrap;
use engine::main_logic::MouseEvent;

use cgmath::Vector2;
use engine::HistoryOneMove;
use engine::MoveHistory;
use engine::Zobrist;
use engine::mesh;
use engine::mesh::small_mesh::SmallMesh;
use engine::move_build::NormalMove;
use engine::moves::SpokeInfo;
use glem::rotate_y;
use gloo::console::console_dbg;

use futures::{SinkExt, StreamExt};
use gloo::console::log;
use gui::shader_sys::ShaderSystem;
use hex::Axial;
use serde::{Deserialize, Serialize};

use shogo::utils;
use wasm_bindgen::prelude::*;

use engine::board;
use engine::moves;
use gui::dom;

use dom::DomToWorker;
use engine::unit;
use hex;

use unit::*;

#[wasm_bindgen]
pub async fn main_entry() {
    let (sender, mut receiver) = futures::channel::mpsc::unbounded();

    // let _foo = [gloop::EventListen::from_closure(&shogo::utils::get_by_id_elem("player_select_white"), "change", |_| {
    //     let se = sender.clone();

    //     //let val: web_sys::HtmlInputElement = shogo::utils::get_by_id_elem("player_select_white").dyn_into().unwrap();
    //     //console_dbg!("player1:", val.checked());
    //     //worker.post_message(DomToWorker::Button("player1".into()));
    //     se.unbounded_send("player_select_white").unwrap_throw();
    // }),
    // gloop::EventListen::from_closure(&shogo::utils::get_by_id_elem("ai_select_white"), "change", |_| {
    //     let se = sender.clone();

    //     //let val: web_sys::HtmlInputElement = shogo::utils::get_by_id_elem("player_select_white").dyn_into().unwrap();
    //     //console_dbg!("player1:", val.checked());
    //     //worker.post_message(DomToWorker::Button("player1".into()));
    //     se.unbounded_send("ai_select_white").unwrap_throw();
    // })];
    let _listeners1 = [
        "player_select_white",
        "ai_select_white",
        "player_select_black",
        "ai_select_black",
        "white_play_first",
    ]
    .map(|s| {
        let se = sender.clone();
        let undo = shogo::utils::get_by_id_elem(s);
        gloo::events::EventListener::new(&undo, "click", move |_event| {
            se.unbounded_send(s).unwrap_throw();
        })
    });

    let _listeners = ["single_b", "map_b"].map(|s| {
        let se = sender.clone();
        let undo = shogo::utils::get_by_id_elem(s);
        gloo::events::EventListener::new(&undo, "click", move |_event| {
            se.unbounded_send(s).unwrap_throw();
        })
    });

    let t: web_sys::HtmlTextAreaElement = gloo::utils::document()
        .get_element_by_id("textarea_m")
        .unwrap()
        .dyn_into()
        .unwrap();

    let map = "---------------------------------b------r----k---------------------------------------------";
    t.set_value(&map);

    let editor_elem = shogo::utils::get_by_id_elem("editor2");
    editor_elem.set_attribute("style", "display:none;").unwrap();

    let game_elem = shogo::utils::get_by_id_elem("game_b");
    game_elem.set_attribute("style", "display:none;").unwrap();

    let mut white = dom::Slot::Player;
    let mut black = dom::Slot::Ai;
    let mut starting_team = dom::Team::White;
    let command = loop {
        let Some(r) = receiver.next().await else {
            unreachable!()
        };
        let t: web_sys::HtmlTextAreaElement = gloo::utils::document()
            .get_element_by_id("textarea_m")
            .unwrap()
            .dyn_into()
            .unwrap();

        match r {
            "white_play_first" => {
                let val: web_sys::HtmlInputElement =
                    shogo::utils::get_by_id_elem("white_play_first")
                        .dyn_into()
                        .unwrap();
                if val.checked() {
                    starting_team = dom::Team::White;
                } else {
                    starting_team = dom::Team::Black;
                }
            }
            "player_select_white" => {
                let val: web_sys::HtmlInputElement =
                    shogo::utils::get_by_id_elem("player_select_white")
                        .dyn_into()
                        .unwrap();
                if val.checked() {
                    white = dom::Slot::Player;
                }
            }
            "player_select_black" => {
                let val: web_sys::HtmlInputElement =
                    shogo::utils::get_by_id_elem("player_select_black")
                        .dyn_into()
                        .unwrap();
                if val.checked() {
                    black = dom::Slot::Player;
                }
            }
            "ai_select_white" => {
                let val: web_sys::HtmlInputElement =
                    shogo::utils::get_by_id_elem("ai_select_white")
                        .dyn_into()
                        .unwrap();
                if val.checked() {
                    white = dom::Slot::Ai;
                }
            }
            "ai_select_black" => {
                let val: web_sys::HtmlInputElement =
                    shogo::utils::get_by_id_elem("ai_select_black")
                        .dyn_into()
                        .unwrap();
                if val.checked() {
                    black = dom::Slot::Ai;
                }
            }
            "single_b" => {
                let foo = if let Some(foo) = MyWorld::load_from_string(&t.value()) {
                } else {
                    t.set_value("Invalid game string or too big of a board (size 8 is the max)");
                    log!("Failed to prase string");
                    continue;
                };

                game_elem.set_attribute("style", "display:flex;").unwrap();
                break dom::GameType::Game(white, black, starting_team, t.value().into());

                //break dom::GameType::SinglePlayer(t.value().into());
            }
            // "pass_b" => {
            //     let foo = if let Some(foo) = MyWorld::load_from_string(&t.value()) {
            //     } else {
            //         t.set_value("Invalid game string or too big of a board (size 8 is the max)");
            //         log!("Failed to prase string");
            //         continue;
            //     };

            //     game_elem.set_attribute("style", "display:flex;").unwrap();
            //     break dom::GameType::PassPlay(t.value().into());
            // }
            // "ai_b" => {
            //     let foo = if let Some(foo) = MyWorld::load_from_string(&t.value()) {
            //     } else {
            //         t.set_value("Invalid game string or too big of a board (size 8 is the max)");
            //         log!("Failed to prase string");
            //         continue;
            //     };

            //     game_elem.set_attribute("style", "display:flex;").unwrap();

            //     break dom::GameType::AIBattle(t.value().into());
            // }
            "map_b" => {
                let foo = if let Some(foo) = MyWorld::load_from_string(&t.value()) {
                } else {
                    t.set_value("Invalid game string or too big of a board (size 8 is the max)");
                    log!("Failed to prase string");
                    continue;
                };
                game_elem.set_attribute("style", "display:flex;").unwrap();

                editor_elem.set_attribute("style", "display:flex;").unwrap();
                break dom::GameType::MapEditor(t.value().into());
            }
            "replaybutton" => {
                // let t: web_sys::HtmlTextAreaElement = gloo::utils::document()
                //     .get_element_by_id("textarea_r")
                //     .unwrap()
                //     .dyn_into()
                //     .unwrap();

                todo!();
                // let s: String = t.value().into();

                // // let Some(_) = unit::parse_replay_string(&s, &world) else {
                // //     console_dbg!("Could not part replay");
                // //     continue;
                // // };

                // //TODO this is the proper place to unhide elements. do this elsewhere
                // let elem = shogo::utils::get_by_id_elem("replay_b");
                // elem.set_attribute("style", "display:block;").unwrap();

                // break dom::GameType::Replay(t.value().into());
            }
            _ => {
                todo!()
            }
        }
    };

    //let main_ret = shogo::utils::get_by_id_elem("return-menu");
    //main_ret.set_attribute("style", "display:block;").unwrap();

    let elem = shogo::utils::get_by_id_elem("mainmenu");
    elem.set_attribute("style", "display:none;").unwrap();

    let prot = gloo::utils::window().location().protocol().unwrap();
    let host = gloo::utils::window().location().host().unwrap();

    let host = format!("{}//{}", prot, host);

    // console_dbg!("host", host);

    // let k = search.as_str();

    // let (a, k) = k.split_at(1);
    // console_dbg!(a, k);
    // assert_eq!(a, "?");

    // let res = querystring::querify(k);
    // console_dbg!("querystring:", res);

    // console_dbg!(search);

    // assert_eq!(res[1].0, "data");

    // let command = match res[0] {
    //     ("v", "singleplayer") => {
    //         //assert_eq!(res[1].0, "data");
    //         log!("singleplayer!!!");
    //         GameType::SinglePlayer(res[1].1.into())
    //     }
    //     ("v", "passplay") => {
    //         log!("passplay!!!");
    //         GameType::PassPlay(res[1].1.into())
    //     }
    //     ("v", "aibattle") => {
    //         log!("aibattle!!!");
    //         GameType::AIBattle(res[1].1.into())
    //     }
    //     ("v", "replay") => {
    //         //assert_eq!(res[1].0, "data");
    //         GameType::Replay(res[1].1.into())
    //     }
    //     ("v", "mapeditor") => {
    //         log!("map editor!!!");
    //         GameType::MapEditor(res[1].1.into())
    //     }
    //     _ => {
    //         unreachable!("unrecognized command");
    //     }
    // };

    log!("demo start");

    // let (sender, mut receiver) = futures::channel::mpsc::unbounded();

    // let start_button = utils::get_by_id_elem("startgame");

    // // Attach an event listener
    // let _listener = gloo::events::EventListener::new(&start_button, "click", move |_event| {
    //     log!("STARTING");
    //     sender.unbounded_send(()).unwrap_throw();
    // });

    // let e=receiver.next().await;
    log!("FOO");

    dom::start_game(command, &host).await;
}

#[wasm_bindgen]
pub async fn worker_entry2() {
    //let (mut worker, mut response) = gui::worker::Worker::<AiCommand, AiResponse>::new();
    use shogo::worker::*;
    let (_, tx, mut rx): (
        js_sys::ArrayBuffer,
        WorkerSender<AiResponse>,
        WorkerRecv<AiCommand, js_sys::ArrayBuffer>,
    ) = shogo::worker::create_worker().await;

    loop {
        //console_dbg!("worker:waiting22222");
        let mut res = rx.recv().next().await.unwrap();
        console_dbg!("worker:processing:", res.game.hash_me(), res.team);

        let res = engine::ai::calculate_move(
            &mut res.game,
            //&res.fogs,
            &res.world,
            res.team,
            &res.history,
            &res.zobrist,
        );
        //console_dbg!("worker:finished processing");
        tx.post_message(AiResponse { inner: res });
    }
}

// struct Doop3 {
//     pub ai_worker: WorkerInterface<AiCommand, AiResponse>,
//     pub ai_response: UnboundedReceiver<AiResponse>,
//     pub interrupt_sender: futures::channel::mpsc::Sender<ace::Response>,
// }
// impl Doop3 {
//     async fn interrupt_render_thread(&mut self) {
//         use futures::FutureExt;
//         self.interrupt_sender
//             .send(ace::Response::AnimationFinish)
//             .map(|_| ())
//             .await
//     }
//     async fn wait_response(&mut self) -> ActualMove {
//         use futures::FutureExt;
//         self.ai_response
//             .next()
//             .map(|x| {
//                 let k = x.unwrap();
//                 k.inner
//             })
//             .await
//     }
//     fn send_command(
//         &mut self,
//         game: &GameState,
//         fogs: &[mesh::small_mesh::SmallMesh; 2],
//         world: &board::MyWorld,
//         team: ActiveTeam,
//         history: &MoveHistory,
//     ) {
//         self.ai_worker.post_message(AiCommand {
//             game: game.clone(),
//             fogs: fogs.clone(),
//             world: world.clone(),
//             team,
//             history: history.clone(),
//         });
//     }
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AiCommand {
    game: GameState,
    //fogs: [mesh::small_mesh::SmallMesh; 2],
    world: board::MyWorld,
    team: Team,
    history: MoveHistory<HistoryOneMove>,
    zobrist: Zobrist,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AiResponse {
    inner: engine::move_build::NormalMove,
}

#[wasm_bindgen]
pub async fn worker_entry() {
    console_error_panic_hook::set_once();

    console_dbg!("num tiles={}", hex::Cube::new(0, 0).range(4).count());

    //let (mut wr, mut ss) = shogo::EngineWorker::new().await;
    let (canvas, mut sender, mut recv) = shogo::worker::create_worker().await;

    let k = recv.recv().next().await.unwrap();
    let DomToWorker::Start(game_type) = k else {
        unreachable!("worker:Didn't receive start")
    };
    sender.post_message(dom::WorkerToDom::Ack);

    console_dbg!("Found game thingy", game_type);

    //let mut frame_timer = shogo::FrameTimer::new(60, ss);
    let mut timer = shogo::Timer::new(60);

    let scroll_manager = gui::scroll::TouchController::new([0., 0.].into());

    // let (ai_worker, ai_response) =
    //     worker::WorkerInterface::<AiCommand, AiResponse>::new("./gridlock_worker2.js").await;

    let (interrupt_tx, mut interrupt_rx) = futures::channel::mpsc::channel(5);

    // let mut ai_int = Doop3 {
    //     ai_worker,
    //     ai_response,
    //     interrupt_sender,
    // };

    //let last_matrix = glam::f32::Mat4::IDENTITY;
    let ctx = &utils::get_context_webgl2_offscreen(&canvas);

    let grid_matrix = hex::HexConverter::new();

    let shader = gui::shader_sys::ShaderSystem::new(ctx).unwrap();

    let models = Models::new(&grid_matrix, &shader);

    let mut render = EngineStuff {
        grid_matrix,
        models,
        ctx: ctx.clone(),
        canvas,
        scroll_manager,
        //last_matrix,
        shader,
    };

    // let (seed, _) = if let dom::GameType::Replay(rr) = &game_type {
    //     let Ok(j) = engine::share::load(&rr) else {
    //         wr.post_message(dom::WorkerToDom::CantParseReplay);
    //         return;
    //     };

    //     (j.seed.clone(), Some(j))
    // } else {
    //     (board::WorldSeed::new(), None)
    // };

    //let world = board::MyWorld::new();

    //let map = unit::default_map(&world);
    //console_dbg!("ma", map.save(&world).unwrap());

    let (command_sender, mut command_recv) =
        futures::channel::mpsc::channel::<GameWrap<engine::main_logic::Command>>(5);

    let (mut response_sender, response_recv) = futures::channel::mpsc::channel(5);

    let game_type = match game_type {
        dom::GameType::SinglePlayer(s) => engine::GameType::SinglePlayer(s),
        dom::GameType::PassPlay(s) => engine::GameType::PassPlay(s),
        dom::GameType::AIBattle(s) => engine::GameType::AIBattle(s),
        dom::GameType::Replay(o) => engine::GameType::Replay(o),
        dom::GameType::MapEditor(s) => engine::GameType::MapEditor(s),
        dom::GameType::Game(a, b, team, s) => {
            let a = match a {
                dom::Slot::Player => engine::Slot::Player,
                dom::Slot::Ai => engine::Slot::Ai,
            };
            let b = match b {
                dom::Slot::Player => engine::Slot::Player,
                dom::Slot::Ai => engine::Slot::Ai,
            };

            let t = match team {
                dom::Team::White => engine::Team::White,
                dom::Team::Black => engine::Team::Black,
            };

            engine::GameType::Game(a, b, t, s)
        }
    };

    let world = match game_type.clone() {
        engine::GameType::MapEditor(s) => {
            //TODO handle this error better
            //let map = Map::load(&s, &world).unwrap();

            // let g = engine::main_logic::map_editor(doop, &world, map).await;
            // Finish::MapEditor(g)
            let world = board::MyWorld::load_from_string(&s).unwrap();
            world
        }
        engine::GameType::PassPlay(s)
        | engine::GameType::SinglePlayer(s)
        | engine::GameType::AIBattle(s)
        | engine::GameType::Game(_, _, _, s) => {
            let world = board::MyWorld::load_from_string(&s).unwrap();

            //let map = Map::load(&s, &world).unwrap();

            //TODO handle this error better
            // let res = engine::main_logic::game_play_thread(
            //     doop,
            //     &world,
            //     game_type,
            //     &mut ai_int,
            // )
            // .await;
            // Finish::GameFinish((res.0, res.1, world))
            world
        }
        engine::GameType::Replay(s) => {
            console_dbg!("got map=", s);
            todo!();
            // let (map, history) = unit::parse_replay_string(&s, &world).unwrap();

            // let res = engine::main_logic::replay(&map, &history, &world, doop).await;

            // Finish::GameFinish((res, history, map))
        }
    };

    let render_thead = async {
        while let Some(ace::GameWrap {
            mut game,
            data,
            team,
        }) = command_recv.next().await
        {
            let f1 = render_command(
                data.clone(),
                &mut game,
                team,
                &mut render,
                &world,
                &mut timer,
                &mut recv,
                &mut sender,
                &mut interrupt_rx,
            );

            // //TODO move this interrupt_recv into the render function.
            // if let engine::main_logic::Command::Wait = &data {
            //     let f2 = interrupt_recv.next().map(|x| x.unwrap());
            //     use futures::FutureExt;
            //     futures::select! {
            //         _= f1.fuse()=>{
            //             unreachable!()
            //             // response_sender
            //             // .send(ace::GameWrap { game, data, team })
            //             // .await
            //             // .unwrap();
            //         },
            //         _=f2.fuse()=>{
            //             //console_dbg!("render thread was interrupted!");
            //         }
            //     };
            // } else {
            let data = f1.await;
            response_sender
                .send(ace::GameWrap { game, data, team })
                .await
                .unwrap();
            //}
        }
    };

    let doop = ace::CommandSender {
        sender: command_sender,
        receiver: response_recv,
    };

    enum Finish {
        MapEditor(Map),
        GameFinish((GameOver, engine::MoveHistory<HistoryOneMove>)),
    }

    let gameplay_thread = async {
        match game_type.clone() {
            engine::GameType::MapEditor(_s) => {
                //TODO handle this error better
                // let map = Map::load(&s, &world).unwrap();

                let g = engine::main_logic::map_editor(doop, &world).await;
                Finish::MapEditor(g)
                //todo!();
            }
            engine::GameType::Game(white, black, t, s) => {
                let res = game_play_thread(doop, &world, t, [white, black], interrupt_tx).await;
                Finish::GameFinish((res.0, res.1))
            }
            engine::GameType::PassPlay(_s)
            | engine::GameType::SinglePlayer(_s)
            | engine::GameType::AIBattle(_s) => {
                //let world=board::MyWorld::load_from_string(&s);

                //let map = Map::load(&s, &world).unwrap();

                //TODO handle this error better
                //let res = game_play_thread(doop, &world, game_type, interrupt_tx).await;
                //Finish::GameFinish((res.0, res.1))
                todo!();
            }
            engine::GameType::Replay(s) => {
                console_dbg!("got map=", s);
                todo!();
                // let (map, history) = unit::parse_replay_string(&s, &world).unwrap();

                // let res = engine::main_logic::replay(&map, &history, &world, doop).await;

                // Finish::GameFinish((res, history, map))
            }
        }
    };

    console_dbg!("about to join");
    let (gg, ()) = futures::join!(gameplay_thread, render_thead);

    match gg {
        Finish::MapEditor(map) => {
            //sender.post_message(dom::WorkerToDom::ExportMap(map.save(&world).unwrap()));
            //console_dbg!("exported map", e.save(&world).unwrap());
        }
        Finish::GameFinish((result, g)) => {
            let result = match result {
                GameOver::WhiteWon => dom::GameOverGui::WhiteWon,
                GameOver::BlackWon => dom::GameOverGui::BlackWon,
                GameOver::Tie => dom::GameOverGui::Tie,
            };
            let replay_string = engine::unit::replay_string(&g, &world).unwrap();
            sender.post_message(dom::WorkerToDom::GameFinish {
                replay_string,
                result,
            });
        }
    }

    log!("Worker thread closin");
}

pub async fn game_play_thread(
    mut doop: ace::CommandSender,
    world: &board::MyWorld,
    team: Team,
    player_type: [engine::Slot; 2],
    mut interrupt_tx: futures::channel::mpsc::Sender<()>,
) -> (unit::GameOver, MoveHistory<HistoryOneMove>) {
    console_dbg!("gameplay thread start");

    let (ai_tx, mut ai_rx) = shogo::main::create_main::<AiCommand, AiResponse, _>(
        "./gridlock_worker2.js",
        js_sys::ArrayBuffer::new(0),
    )
    .await;

    console_dbg!("created ai worker");

    let game = world.starting_state.clone();
    let last_seen = LastSeenObjectsAll::new(&GameState::new());

    let mut game = GameStateTotal {
        tactical: game,
        last_seen,
        history: MoveHistory::new(),
    };

    let s = game.tactical.into_string(world);
    doop.repaint_ui(Team::Neutral, &mut game, format!("start:{:?}", s))
        .await;

    let mut team_gen = team.iter();

    let zobrist = Zobrist::new();

    //Loop over each team!
    let g = loop {
        let team = team_gen.next().unwrap();

        gloo::console::console!(format!(
            "Current game [{}]",
            game.tactical.into_string(world)
        ));

        //Write out the last move
        if let Some(gg) = game.history.inner.last() {
            use std::fmt::Write;
            let mut s = String::new();

            let mm = &gg.r;
            write!(
                &mut s,
                "{}:{:?}:n{:?}",
                game.history.inner.len(),
                team.not(),
                world.format(&mm.0.coord)
            )
            .unwrap();
            if mm.1.captured_unit(&mm.0, &game.tactical).is_some() {
                write!(&mut s, "x").unwrap();
            }

            doop.repaint_ui(team, &mut game, s).await;
        }

        if let Some(g) = game.tactical.game_is_over(&world, team, &game.history) {
            gloo::console::console!(format!("game over! {:?}", g));
            let ll = game.history.into_string(world);
            doop.repaint_ui(
                Team::Neutral,
                &mut game,
                format!("Game Over: {:?}! Full history:\"{}\"", g, ll),
            )
            .await;

            match player_type {
                [engine::Slot::Ai, engine::Slot::Ai] => break g,
                _ => {}
            }
        }

        console_dbg!("main thread iter");
        let r1 = match player_type[team.index()] {
            engine::Slot::Ai => {
                let mut ai_state = game.tactical.convert_to_playable(world, team);

                let the_move = if false {
                    ai_tx.post_message(AiCommand {
                        game: ai_state,

                        world: world.clone(),
                        team,
                        history: game.history.clone(),
                        zobrist: zobrist.clone(),
                    });

                    use futures::FutureExt;
                    let the_move = futures::select!(
                        _ = doop.wait_forever(team, &mut game).fuse()=>unreachable!(),
                        x = ai_rx.recv().next().fuse() => x
                    );

                    interrupt_tx.send(()).await.unwrap();

                    let k = doop.receiver.next().await;
                    matches!(k.unwrap().data, ace::Response::AnimationFinish);

                    //ai_int.interrupt_render_thread().await;

                    the_move.unwrap().inner
                } else {
                    engine::ai::calculate_move(&mut ai_state, &world, team, &game.history, &zobrist)
                };

                the_move
            }
            engine::Slot::Player => {
                engine::main_logic::handle_player(&mut game, &world, &mut doop, team).await
            }
        };

        let effect = r1.animate_move(team, &game, world, &mut doop).await.apply(
            team,
            &mut game.tactical,
            world,
            None,
        );

        let r = (r1, effect);

        let fe = game.last_seen.apply(&game.tactical, (&r.0, &r.1), world);

        game.history.inner.push(engine::HistoryOneMove { r, fe });

        let spoke_info = moves::SpokeInfo::new(&game.tactical, world);
        let curr_eval_player = engine::ai::Evaluator::default().absolute_evaluate(
            &game.tactical,
            world,
            &spoke_info,
            false,
        );
        console_dbg!(curr_eval_player);

        doop.wait_sometime(team, &mut game, 60).await;
    };

    //When ai vs ai finishes, allow player to look around.
    loop {
        let data = doop.get_mouse(Team::Neutral, &mut game).await;
    }
}

use gui::model_parse::*;
use gui::*;
use web_sys::OffscreenCanvas;
use web_sys::WebGl2RenderingContext;

pub struct EngineStuff {
    grid_matrix: hex::HexConverter,
    models: Models<gui::model_parse::Foo<TextureGpu, ModelGpu>>,
    //numm: Numm,
    ctx: WebGl2RenderingContext,
    canvas: OffscreenCanvas,
    scroll_manager: gui::scroll::TouchController,
    //last_matrix: glam::f32::Mat4,
    shader: ShaderSystem,
}

async fn render_command(
    command: ace::Command,
    game_total: &GameStateTotal,
    team: Team,
    e: &mut EngineStuff,
    world: &board::MyWorld,
    timer: &mut shogo::Timer,
    dom_messages: &mut shogo::worker::WorkerRecv<DomToWorker, web_sys::OffscreenCanvas>,
    engine_worker: &mut shogo::worker::WorkerSender<dom::WorkerToDom>,
    interrupt_rx: &mut futures::channel::mpsc::Receiver<()>,
) -> ace::Response {
    let game_total = game_total.clone();
    //let game = &game_total.tactical;
    let scroll_manager = &mut e.scroll_manager;
    //let last_matrix = &mut e.last_matrix;
    let ctx = &e.ctx;
    let canvas = &e.canvas;
    let grid_matrix = &e.grid_matrix;
    let models = &e.models;

    let draw_sys = &mut e.shader; //ctx.shader_system();

    let gl_width = canvas.width();
    let gl_height = canvas.height();
    ctx.viewport(0, 0, gl_width as i32, gl_height as i32);
    let mut viewport = [canvas.width() as f32, canvas.height() as f32];

    let drop_shadow = &models.drop_shadow;
    // let black_mouse = &models.black_mouse;
    // let white_mouse = &models.white_mouse;
    // let black_rabbit = &models.black_rabbit;
    // let white_rabbit = &models.white_rabbit;

    //let fog_asset = &models.fog;
    // let water = &models.token_neutral;
    // let grass = &models.grass;
    // let mountain_asset = &models.mountain;
    // let snow = &models.snow;
    let select_model = &models.select_model;
    // let attack_model = &models.attack;

    //First lets process the command. Break it down
    //into pieces that this thread understands.
    let mut get_mouse_input = None;
    let mut unit_animation = None;
    let mut terrain_animation = None;
    let mut poking = 0;
    let mut camera_moving_last = scroll::CameraMoving::Stopped;

    let mut show_hidden_units = false;
    let score_data = game_total.tactical.score(world);
    let score_data = dom::ScoreData {
        white: score_data.white,
        black: score_data.black,
        neutral: score_data.neutral,
    };

    //TODO remove
    let command_copy = command.clone();

    // let game_str = game.into_string(world);
    // let history_str = game_total.foo.into_string(world);

    let spoke = moves::SpokeInfo::new(&game_total.tactical, world);

    struct Foo {
        start: Axial,
        normal_moves: SmallMesh,
        suicidal_moves: SmallMesh,
        grey: bool,
    }

    let mut tick_counter = 0;
    let mut repaint_ui = None;
    let mut awaiting_popup_stop = None;
    //let mut waiting_engine_ack = false;
    //console_dbg!(command);
    match command {
        ace::Command::RepaintUI(foo) => {
            repaint_ui = Some(foo);
            // let k = update_text(world, grid_matrix, viewport, &cgmath::Matrix4::identity());
            // engine_worker.post_message(dom::WorkerToDom::TextUpdate(k, score_data.clone(), foo));
            // return ace::Response::Ack;
        }
        ace::Command::HideUndo => {
            engine_worker.post_message(dom::WorkerToDom::HideUndo);
            //waiting_engine_ack = true;
            return ace::Response::Ack;
        }
        ace::Command::ShowUndo => {
            engine_worker.post_message(dom::WorkerToDom::ShowUndo);
            //waiting_engine_ack = true;
            return ace::Response::Ack;
        }
        ace::Command::Animate(ak) => match ak {
            engine::main_logic::AnimationCommand::Movement { unit, end } => {
                // let ff = match data {
                //     move_build::PushInfo::PushedLand => {
                //         Some(animation::land_delta(unit, end, grid_matrix))
                //     }
                //     move_build::PushInfo::UpgradedLand => {
                //         todo!("BLAP");
                //     }
                //     move_build::PushInfo::PushedUnit => {
                //         todo!("BLAP");
                //     }

                //     move_build::PushInfo::None => None,
                // };

                let it = {
                    let a = grid_matrix.hex_axial_to_world(&unit);
                    let b = grid_matrix.hex_axial_to_world(&end);

                    (0..100).map(move |c| {
                        let counter = c as f32 / 100.0;
                        use cgmath::VectorSpace;
                        a.lerp(b, counter)
                    })
                };

                unit_animation = Some((Vector2::new(0.0, 0.0), it, unit));
            }
            engine::main_logic::AnimationCommand::Terrain {
                pos,
                terrain_type,
                dir,
            } => {
                let (a, b) = match dir {
                    engine::main_logic::AnimationDirection::Up => (-5., 0.),
                    engine::main_logic::AnimationDirection::Down => (0., -6.), //TODO 6 to make sure there is a frame with it gone
                };
                let it = gui::animation::terrain_create(a, b);
                terrain_animation = Some((0.0, it, pos, terrain_type));
            }
        },
        ace::Command::GetMouseInputSelection { selection, grey } => {
            match selection {
                ace::CellSelection::MoveSelection(axial, small_mesh, have_moved) => {
                    let game2 = game_total.tactical.convert_to_playable(world, team);
                    let spoke2 = SpokeInfo::new(&game2, world);
                    let mut suicidal_moves = mesh::small_mesh::SmallMesh::from_iter_move(
                        NormalMove::generate_suicidal(&game2, world, team, &spoke2),
                    );
                    suicidal_moves.inner &= small_mesh.inner;
                    //suicidal_moves.set_coord(axial,false);

                    //suicidal_moves
                    let mut normal_moves = small_mesh;
                    normal_moves.inner &= !suicidal_moves.inner;

                    let foo = Foo {
                        normal_moves,
                        suicidal_moves,
                        grey,
                        start: axial,
                    };

                    get_mouse_input = Some(Some(foo));
                }
                ace::CellSelection::BuildSelection(axial) => todo!(),
            };
            //let normal_moves

            //get_mouse_input = Some(Some((selection, grey)));
        }
        ace::Command::GetMouseInputNoSelect => get_mouse_input = Some(None),
        ace::Command::WaitAI => {}
        ace::Command::Wait(None) => {}
        ace::Command::Wait(Some(ticks)) => {
            tick_counter = ticks;
        }
        ace::Command::Popup(str) => {
            engine_worker.post_message(dom::WorkerToDom::ShowPopup(str.clone()));
            awaiting_popup_stop = Some(str);
        }
        ace::Command::Poke => {
            poking = 3;
        }
    };

    let game_total = if awaiting_popup_stop.is_some() {
        let game = GameState::new();
        let last_seen = LastSeenObjectsAll::new(&GameState::new());

        let game = GameStateTotal {
            tactical: game,
            last_seen,
            history: MoveHistory::new(),
        };

        game
    } else {
        game_total
    };

    let game = &game_total.tactical;

    let grid_snap = |c: Axial, cc| {
        let pos = grid_matrix.hex_axial_to_world(&c);
        let t = glem::translate(pos.x, pos.y, cc);
        glem::build(&t)
    };

    let mut water = mesh::small_mesh::SmallMesh::new();

    let land = world.land.inner & !water.inner; //& !game.factions.ice.inner;

    //TODO dont use this, also make sure to draw water tiles on the border that can be seen from the side?
    water.inner |= world.get_game_cells().inner;

    let team_perspective = team; //Team::White;

    let darkness = game.darkness(world, team_perspective);

    loop {
        if poking == 1 {
            console_dbg!("we poked!");
            return ace::Response::Ack;
        }
        poking = 0.max(poking - 1);

        let mut on_select = false;
        let mut button_pushed = None;

        let mut resize_text = false;
        use futures::FutureExt;

        let mut resize_canvas = None;

        let proj = gui::projection::projection(viewport);
        let view_proj = gui::projection::view_matrix(
            scroll_manager.camera(),
            scroll_manager.zoom(),
            scroll_manager.rot(),
        );

        use glem::prelude::*;

        let my_matrix = glem::build(&proj.chain(view_proj));

        loop {
            if repaint_ui.is_some() {
                break;
            }
            futures::select! {
                _ = interrupt_rx.next()=>{
                    matches!(command_copy,ace::Command::Wait(None));
                    return ace::Response::AnimationFinish;
                },
                () = timer.next().fuse() =>{
                    break;
                },
                k = dom_messages.recv().next() =>{
                    let k=k.unwrap();
                    let e=&k;
                    match e {

                        DomToWorker::Resize {
                            canvasx: _canvasx,
                            canvasy: _canvasy,
                            x,
                            y,
                        } => {
                            resize_canvas=Some((*x,*y));

                            resize_text = true;
                        }
                        DomToWorker::TouchMove { touches } => {
                            scroll_manager.on_touch_move(touches, &my_matrix, viewport);
                        }
                        DomToWorker::TouchDown { touches } => {
                            scroll_manager.on_new_touch(touches);
                        }
                        DomToWorker::TouchEnd { touches } => {
                            if let gui::scroll::MouseUp::Select = scroll_manager.on_touch_up(touches) {
                                on_select = true;
                            }
                        }
                        DomToWorker::CanvasMouseLeave => {
                            log!("mouse leaving!");
                            let _ = scroll_manager.on_mouse_up();
                        }
                        DomToWorker::CanvasMouseUp => {
                            if let gui::scroll::MouseUp::Select = scroll_manager.on_mouse_up() {
                                on_select = true;
                            }
                        }
                        DomToWorker::Button(s) => {
                            if let Some(oo)=&awaiting_popup_stop && s=="popup_ack"{
                                engine_worker.post_message(dom::WorkerToDom::HidePopup(oo.clone()));

                                return ace::Response::Ack;
                            }

                            button_pushed = Some(s.clone());

                            // match s.as_str(){
                            //     "undo"=>{
                            //         butt=true
                            //     },
                            //     "b_water"=>{
                            //         console_dbg!("clicked wattttrrrr");
                            //     },
                            //     _=>{
                            //         panic!("not supported yet");
                            //     }
                            // }
                        }
                        DomToWorker::Ack => {
                            //assert!(waiting_engine_ack);

                            // if waiting_engine_ack {
                            //     return ace::Response::Ack;
                            // }
                        }
                        DomToWorker::CanvasMouseMove { x, y } => {
                            scroll_manager.on_mouse_move([*x, *y], &my_matrix, viewport);
                        }

                        DomToWorker::CanvasMouseDown { x, y } => {
                            scroll_manager.on_mouse_down([*x, *y]);
                        }
                        DomToWorker::ButtonClick => {}
                        DomToWorker::ShutdownClick => todo!(),
                        DomToWorker::Start(_) => todo!(),
                    }
                }
            }
        }

        if tick_counter != 0 {
            tick_counter -= 1;
            if tick_counter == 0 {
                return ace::Response::Ack;
            }
        }
        if let Some((x, y)) = resize_canvas {
            let xx = x as u32;
            let yy = y as u32;
            canvas.set_width(xx);
            canvas.set_height(yy);
            ctx.viewport(0, 0, xx as i32, yy as i32);

            viewport = [xx as f32, yy as f32];
            log!(format!("updating viewport to be:{:?}", viewport));
        }
        let projjj = my_matrix.as_ref();

        let piece_scale: f32 = 0.8;

        let mouse_world =
            gui::scroll::mouse_to_world(scroll_manager.cursor_canvas(), &my_matrix, viewport);

        {
            if resize_text || repaint_ui.is_some() {
                let s = if let Some(foo) = &repaint_ui {
                    foo.clone()
                } else {
                    "".into()
                };

                let k = update_text(world, grid_matrix, viewport, &my_matrix);
                engine_worker.post_message(dom::WorkerToDom::TextUpdate(k, score_data.clone(), s));
            }

            if repaint_ui.is_some() {
                return ace::Response::Ack;
            }
        }

        if get_mouse_input.is_some() {
            if let Some(button) = button_pushed {
                return if let Some(foo) = get_mouse_input.unwrap() {
                    ace::Response::Mouse(MouseEvent::Button(button.clone()))
                } else {
                    ace::Response::Mouse(MouseEvent::Button(button.clone()))
                };
                //return ace::Response::Mouse(MouseEvent::Undo);
            } else if on_select {
                let mouse: Axial = grid_matrix.center_world_to_hex(mouse_world.into());
                log!(format!("pos:{:?}", mouse.to_cube()));

                if world.get_game_cells().is_set(mouse) {
                    // let mut s = String::new();
                    // ActualMove {
                    //     moveto: mouse.to_index(),
                    // }
                    // .as_text(&world, &mut s)
                    // .unwrap();

                    log!(format!(
                        "game pos:{:?}",
                        mouse.to_letter_coord(world.radius as i8)
                    ));

                    let data = if let Some(foo) = get_mouse_input.unwrap() {
                        ace::Response::Mouse(MouseEvent::Normal(mouse))
                    } else {
                        ace::Response::Mouse(MouseEvent::Normal(mouse))
                    };

                    return data;
                }
            }
        }

        if let Some((z, a, _, _)) = &mut terrain_animation {
            if let Some(zpos) = a.next() {
                *z = zpos;
            } else {
                return ace::Response::AnimationFinish;
            }
        }
        if let Some((lpos, a, _)) = &mut unit_animation {
            if let Some(pos) = a.next() {
                *lpos = pos;
            } else {
                return ace::Response::AnimationFinish;
            }
        }

        let camera_moving = scroll_manager.step();

        match (camera_moving, camera_moving_last) {
            (scroll::CameraMoving::Stopped, scroll::CameraMoving::Moving) => {
                let k = update_text(world, grid_matrix, viewport, &my_matrix);
                engine_worker.post_message(dom::WorkerToDom::TextUpdate(
                    k,
                    score_data.clone(),
                    "".to_string(),
                ));
            }
            (scroll::CameraMoving::Moving, scroll::CameraMoving::Stopped) => {
                engine_worker.post_message(dom::WorkerToDom::TextUpdate(
                    vec![],
                    score_data.clone(),
                    "".to_string(),
                ));
            }
            _ => {}
        }
        camera_moving_last = camera_moving;

        draw_sys.draw_clear([0.1, 0.1, 0.1, 0.0]);

        draw_sys
            .batch(
                land.iter_ones()
                    .map(|e| grid_snap(Axial::from_index(&e), -models.token_neutral.height)),
            )
            .build(&models.land, &projjj);

        draw_sys
            .batch(
                darkness.inner.iter_ones().map(|e| {
                    grid_snap(Axial::from_index(&e), 0.1).chain(glem::scale(2.3, 2.3, 2.3))
                }),
            )
            .build(&models.black_sigl, &projjj);

        let cell_height = models.token_neutral.height;

        // {
        //     //Draw grass
        //     let grass1 = game
        //         .env
        //         .terrain
        //         .land
        //         .iter_mesh()
        //         .map(|e| grid_snap(e, LAND_OFFSET));

        //     let ani_grass = if let Some((zpos, _, gpos, k)) = &terrain_animation {
        //         if let animation::TerrainType::Grass = k {
        //             let gpos = *gpos;

        //             let pos = grid_matrix.hex_axial_to_world(&gpos);

        //             let t = matrix::translation(pos.x, pos.y, LAND_OFFSET + *zpos);
        //             let m = my_matrix.chain(t).generate();
        //             Some(m)
        //         } else {
        //             None
        //         }
        //     } else {
        //         None
        //     };

        //     let push_grass = if let Some((pos, _, _unit, _, data)) = &unit_animation {
        //         if let Some(f) = data {
        //             let kk = pos + f;
        //             let m = my_matrix
        //                 .chain(matrix::translation(kk.x, kk.y, LAND_OFFSET))
        //                 .chain(matrix::scale(1.0, 1.0, 1.0))
        //                 .generate();
        //             Some(m)
        //         } else {
        //             None
        //         }
        //     } else {
        //         None
        //     };

        //     let all_grass = grass1
        //         .chain(ani_grass.into_iter())
        //         .chain(push_grass.into_iter());

        //     draw_sys.batch(all_grass).build(grass);
        // }

        // {
        //     //Draw forest
        //     let grass1 = game
        //         .env
        //         .terrain
        //         .forest
        //         .iter_mesh()
        //         .map(|e| grid_snap(e, LAND_OFFSET));

        //     let all_grass = grass1;

        //     draw_sys.batch(all_grass).build(mountain_asset);
        // }

        // {
        //     //Draw mountain
        //     let grass1 = game
        //         .env
        //         .terrain
        //         .mountain
        //         .iter_mesh()
        //         .map(|e| grid_snap(e, 0.0));

        //     let all_grass = grass1;

        //     draw_sys.batch(all_grass).build(mountain_asset);
        // }

        // {
        //     //Draw fog
        //     let fog1 = game.env.fog.iter_mesh().map(|e| grid_snap(e, LAND_OFFSET));

        //     let ani_fog = if let Some((zpos, _, gpos, k)) = &terrain_animation {
        //         if let animation::TerrainType::Fog = k {
        //             let gpos = *gpos;

        //             let pos = grid_matrix.hex_axial_to_world(&gpos);

        //             let t = matrix::translation(pos.x, pos.y, LAND_OFFSET + *zpos);
        //             let m = my_matrix.chain(t).generate();
        //             Some(m)
        //         } else {
        //             None
        //         }
        //     } else {
        //         None
        //     };

        //     let all_fog = fog1.chain(ani_fog.into_iter());

        //     draw_sys.batch(all_fog).build(snow);
        // }

        if let Some(a) = &get_mouse_input {
            if let Some(Foo {
                start,
                normal_moves,
                suicidal_moves,
                grey,
            }) = a
            {
                let cells = normal_moves.iter_mesh(Axial::zero()).map(|e| {
                    let zzzz = 0.0;

                    glem::build(&grid_snap(e, zzzz).chain(glem::scale(1.0, 1.0, 1.0)))
                });
                draw_sys
                    .batch(cells)
                    .no_lighting()
                    .grey(*grey)
                    .build(select_model, &projjj);

                let cells = suicidal_moves.iter_mesh(Axial::zero()).map(|e| {
                    let zzzz = 0.0;

                    glem::build(&grid_snap(e, zzzz).chain(glem::scale(1.0, 1.0, 1.0)))
                });
                draw_sys
                    .batch(cells)
                    .no_lighting()
                    .grey(*grey)
                    .build(&models.select_model_red, &projjj);

                // {
                //     let cells = loud.iter_mesh(Axial::zero()).map(|e| {
                //         let zzzz = 0.0;

                //         grid_snap(e, zzzz)
                //             .chain(matrix::scale(1.0, 1.0, 1.0))
                //             .generate()
                //     });
                //     draw_sys
                //         .batch(cells)
                //         .no_lighting()
                //         .grey(*grey)
                //         .build(&models.donut, &projjj);
                // }

                // if let Some(k) = hh {
                //     if k.the_move
                //         .original
                //         .to_cube()
                //         .dist(&k.the_move.moveto.to_cube())
                //         == 2
                //     {
                //         let a = k.the_move.original;
                //         let pos = grid_matrix.hex_axial_to_world(&a);
                //         let t = matrix::translation(pos.x, pos.y, 0.0);
                //         let m = my_matrix.chain(t).generate();
                //         draw_sys
                //             .batch([m])
                //             .no_lighting()
                //             .grey(*grey)
                //             .build(attack_model);
                //     }
                // }
            }
        }

        //let shown_team = team;
        let shown_team = Team::White;

        // let shown_fog = match shown_team {
        //     Team::White => &game_total.fog[0],
        //     Team::Black => &game_total.fog[1],
        //     Team::Neutral => todo!(),
        // };

        {
            let zzzz = 0.1;

            // Draw shadows
            let _d = DepthDisabler::new(ctx);

            let small_shadow = 0.6;
            let large_shadow = 0.8;
            let shadows = world
                .get_game_cells()
                .iter_mesh(Axial::zero())
                .filter_map(|a| {
                    match game.factions.get_cell(a) {
                        &GameCell::Piece(unit::Piece {
                            height: stack_height,
                            team: tt,
                            ..
                        }) => {
                            let val = stack_height.to_num();
                            if tt != team_perspective {
                                if !show_hidden_units && darkness.is_set(a) {
                                    return None;
                                }
                            }

                            let xx = if val == 6 && tt == Team::Neutral {
                                //1.3
                                return None;
                            } else {
                                match val {
                                    0 | 1 | 2 | 3 => small_shadow * piece_scale,
                                    4 | 5 | 6 => large_shadow * piece_scale,
                                    _ => unreachable!(),
                                }
                            };

                            Some(glem::build(
                                &grid_snap(a, zzzz).chain(glem::scale(xx, xx, 1.0)),
                            ))
                        }
                        GameCell::Empty => None,
                    }
                });

            let ani_drop_shadow = unit_animation
                .as_ref()
                .map(|a| {
                    let pos = a.0;
                    glem::build(&glem::translate(pos.x, pos.y, zzzz).chain(glem::scale(
                        small_shadow * piece_scale,
                        small_shadow * piece_scale,
                        1.0,
                    )))
                })
                .filter(|_| team == team_perspective);

            let all_shadows = shadows.chain(ani_drop_shadow.into_iter());

            draw_sys.batch(all_shadows).build(drop_shadow, &projjj);
        }

        //TODO pre-allocate
        let mut white_team_cells = vec![];
        let mut black_team_cells = vec![];
        let mut neutral_team_cells = vec![];
        //let mut mountains = vec![];

        {
            let radius = [0.4, 0.6, 0.8];

            if team == team_perspective {
                if let Some((pos, ..)) = &unit_animation {
                    let ss = radius[0];
                    //Draw it a bit lower then static ones so there is no flickering
                    let first = glem::build(
                        &glem::translate(pos.x, pos.y, 1.0)
                            .chain(glem::scale(ss, ss, 1.0))
                            .chain(glem::scale(piece_scale, piece_scale, piece_scale)),
                    );

                    match team {
                        Team::White => {
                            white_team_cells.push(first);
                        }
                        Team::Black => {
                            black_team_cells.push(first);
                        }
                        Team::Neutral => {
                            unreachable!();
                            //neutral_team_cells.push(first);
                        }
                    }
                }
            }

            let visible_cells =
                game.factions
                    .cells
                    .iter()
                    .enumerate()
                    .filter(|(index, d)| match d {
                        GameCell::Piece(d) => {
                            if d.team != team_perspective {
                                !darkness.inner[*index]
                            } else {
                                true
                            }
                        }
                        GameCell::Empty => false,
                    });

            let last_seen_cells = game_total.last_seen.fog[team]
                .state
                .factions
                .cells
                .iter()
                .enumerate()
                .filter(|(i, x)| if darkness.inner[*i] { true } else { false });

            for (index, pp) in visible_cells
                .chain(last_seen_cells)
                .filter_map(|(index, x)| match x {
                    GameCell::Piece(p) => Some((index, p)),
                    GameCell::Empty => None,
                })
            {
                let a = Axial::from_index(&index);
                let inner_stack = pp.height.to_num().min(3);
                let mid_stack = pp.height.to_num().max(3).min(6) - 3;

                let arr = match pp.team {
                    Team::White => &mut white_team_cells,
                    Team::Black => &mut black_team_cells,
                    Team::Neutral => &mut neutral_team_cells,
                };

                if pp.height == StackHeight::Stack0 {
                    assert!(pp.has_lighthouse);
                    let radius = radius[0];
                    arr.push(glem::build(
                        &grid_snap(a, 5. as f32 * cell_height * piece_scale)
                            .chain(rotate_y(1.0))
                            .chain(glem::scale(radius, radius, 1.0))
                            .chain(glem::scale(piece_scale, piece_scale, piece_scale)),
                    ));
                } else {
                    for (stack, radius) in [inner_stack, mid_stack].iter().zip(radius) {
                        for k in 0..*stack {
                            arr.push(glem::build(
                                &grid_snap(a, k as f32 * cell_height * piece_scale)
                                    .chain(glem::scale(radius, radius, 1.0))
                                    .chain(glem::scale(piece_scale, piece_scale, piece_scale)),
                            ));
                        }
                    }
                }
            }
        }

        draw_sys
            .batch(white_team_cells)
            .build(&models.token_white, &projjj);
        draw_sys
            .batch(black_team_cells)
            .build(&models.token_black, &projjj);
        draw_sys
            .batch(neutral_team_cells)
            .build(&models.token_neutral, &projjj);

        let visible_lighthouses =
            game.factions
                .cells
                .iter()
                .enumerate()
                .filter_map(|(index, k)| match k {
                    GameCell::Piece(e) => {
                        if !e.has_lighthouse {
                            return None;
                        }
                        if e.team != team_perspective {
                            if !show_hidden_units && darkness.inner[index] {
                                return None;
                            }
                        }
                        Some((index, k))
                    }
                    GameCell::Empty => None,
                });

        let last_seen_lighthouses = game_total.last_seen.fog[team]
            .state
            .factions
            .cells
            .iter()
            .enumerate()
            .filter(|(i, x)| {
                if !darkness.inner[*i] {
                    return false;
                }
                match x {
                    GameCell::Piece(o) => o.has_lighthouse,
                    GameCell::Empty => false,
                }
            });

        let mut lighthouses = vec![];
        for (index, pp) in visible_lighthouses.chain(last_seen_lighthouses) {
            let e = Axial::from_index(&index);
            let k = glem::build(&grid_snap(e, 0.0).chain(glem::scale(1.0, 1.0, 1.0)));

            lighthouses.push(k);
        }

        draw_sys
            .batch(lighthouses)
            .build(&models.lighthouse, &projjj);

        let mut water_pos = vec![];
        for a in water.iter_mesh(Axial::zero()) {
            water_pos.push(grid_snap(a, -models.land.height));
        }
        draw_sys.batch(water_pos).build(&models.water, &projjj);

        // let mut ice_pos = vec![];
        // for pos in game.factions.ice.iter_mesh(Axial::zero()) {
        //     ice_pos.push(grid_snap(pos, -models.land.height));
        // }
        // draw_sys.batch(ice_pos).build(&models.snow, &projjj);

        //let mut fog_pos = vec![];
        // let fogg = match team {
        //     ActiveTeam::White => &game_total.fog[0],
        //     ActiveTeam::Black => &game_total.fog[1],
        //     ActiveTeam::Neutral => todo!(),
        // };

        // for pos in shown_fog.iter_mesh(Axial::zero()) {
        //     fog_pos.push(grid_snap(pos, 0.0));
        // }
        // draw_sys.batch(fog_pos).build(&models.fog, &projjj);

        let mut label_arrows = vec![];
        for (pos, hdir) in label_arrow_points(world) {
            let pos = grid_matrix.hex_axial_to_world(&pos);
            let t = glem::translate(pos.x, pos.y, -5.0);
            let r =
                glem::rotate_z((((hdir as usize) + 2) % 6) as f32 * (std::f32::consts::TAU / 6.0));

            let m = glem::build(&t.chain(r));

            label_arrows.push(m);
        }
        draw_sys
            .batch(label_arrows)
            .no_lighting()
            .build(&models.label_arrow, &projjj);

        // let mut white_cells=vec!();
        // let mut black_cells=vec!();

        // for &fo in world.land_as_vec.iter(){
        //     if let Some(fo)=game.factions.get_cell_inner(fo){

        //     }else{
        //         let foo=get_num_attack(&spoke, fo);

        //         let radius=1.0;
        //         let foo2=grid_snap(Axial::from_index(&fo), 0.0 as f32 * cell_height * piece_scale)
        //         .chain(matrix::scale(radius, radius, 1.0))
        //         .chain(matrix::scale(piece_scale, piece_scale, piece_scale))
        //         .generate();

        //         if foo[Team::White]>foo[Team::Black]{
        //             white_cells.push(foo2);
        //         }else if foo[Team::Black]>foo[Team::White]{
        //             black_cells.push(foo2);
        //         }

        //     }
        // }

        // draw_sys
        //     .batch(white_cells)
        //     .build(&models.white_sigl, &projjj);
        // draw_sys
        //     .batch(black_cells)
        //     .build(&models.black_sigl, &projjj);

        ctx.flush();
    }
}

pub fn label_arrow_points(world: &board::MyWorld) -> impl Iterator<Item = (hex::Cube, hex::HDir)> {
    let rr = world.radius as i8 - 1;
    let a1 = Axial { q: 0, r: -rr };
    let a2 = Axial { q: rr, r: -rr };
    let a3 = Axial { q: rr, r: 0 };

    let first = anchor_points2(a1, a2, a3).map(|x| {
        let x = x.add(Axial { q: 1, r: -1 });

        (x.to_cube(), hex::HDir::BottomRight)
    });

    let a1 = Axial { q: 0, r: -rr };
    let a2 = Axial { q: -rr, r: 0 };
    let a3 = Axial { q: -rr, r: rr };

    let second = anchor_points2(a1, a2, a3).map(|x| {
        let x = x.add(Axial { q: -1, r: 0 });

        (x.to_cube(), hex::HDir::BottomLeft)
    });

    //first.chain(second)
    //std::iter::empty()
    first.chain(second)
}

fn update_text(
    world: &board::MyWorld,
    grid_matrix: &hex::HexConverter,
    viewport: [f32; 2],
    my_matrix: &glam::f32::Mat4,
) -> Vec<dom::Text> {
    let make_text = |point: hex::Cube, text: String| {
        let pos = grid_matrix.hex_axial_to_world(&point);
        let pos = scroll::world_to_mouse([pos.x, pos.y, -5.0], viewport, &my_matrix);
        dom::Text { text, pos }
    };

    let radius = world.radius as i8;

    let mut k = Vec::new();
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

    let rr = radius - 1;

    let aaa = alphabet.chars().nth(0).unwrap();
    let bbb = alphabet.chars().nth(rr as usize).unwrap();
    let ccc = alphabet.chars().nth((rr * 2) as usize).unwrap();
    console_dbg!(aaa, bbb, ccc);
    let a11 = Axial::from_letter_coord(aaa, 1, world.radius as i8);
    let a22 = Axial::from_letter_coord(bbb, 1, world.radius as i8);
    let a33 = Axial::from_letter_coord(ccc, 1 + rr, world.radius as i8);

    let a1 = Axial { q: 0, r: -rr };
    let a2 = Axial { q: rr, r: -rr };
    let a3 = Axial { q: rr, r: 0 };

    assert_eq!(a1, a11);
    assert_eq!(a2, a22);
    assert_eq!(a3, a33);

    for (a, letter) in anchor_points2(a1, a2, a3).zip(alphabet.chars()) {
        let a = a.add(Axial { q: 1, r: -1 });
        k.push(make_text(a.into(), letter.to_uppercase().to_string()))
    }

    let a11 = Axial::from_letter_coord(aaa, 1, world.radius as i8);
    let a22 = Axial::from_letter_coord(aaa, 1 + rr, world.radius as i8);
    let a33 = Axial::from_letter_coord(bbb, 1 + rr + rr, world.radius as i8);

    let rr = radius - 1;
    let a1 = Axial { q: 0, r: -rr };
    let a2 = Axial { q: -rr, r: 0 };
    let a3 = Axial { q: -rr, r: rr };

    assert_eq!(a1, a11);
    assert_eq!(a2, a22);
    assert_eq!(a3, a33);

    for (a, num) in anchor_points2(a1, a2, a3).zip(1..) {
        let a = a.add(Axial { q: -1, r: 0 });
        k.push(make_text(a.into(), num.to_string()))
    }

    k
}

fn anchor_points2(start: Axial, bend_point: Axial, end: Axial) -> impl Iterator<Item = hex::Axial> {
    let offset = bend_point.sub(&start).to_cube();

    let unit = Axial {
        q: offset.q.clamp(-1, 1),
        r: offset.r.clamp(-1, 1),
    };

    let dis = (offset.q.abs() + offset.r.abs() + offset.s.abs()) / 2;
    //console_dbg!(dis);
    let first = (0..dis).map(move |x| start.add(unit.mul(x)));

    let offset = end.sub(&bend_point);
    let unit = Axial {
        q: offset.q.clamp(-1, 1),
        r: offset.r.clamp(-1, 1),
    };

    let second = (0..dis + 1).map(move |x| bend_point.add(unit.mul(x)));

    first.chain(second)
}
