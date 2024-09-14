use ace::MouseEvent;
use cgmath::{InnerSpace, Matrix4, Transform, Vector2};
use gloo::console::console_dbg;

use futures::{SinkExt, StreamExt};
use gloo::console::log;
use gui::shader_sys::ShaderSystem;
use hex::Axial;
use model::matrix::{self, MyMatrix};
use moves::ActualMove;
use serde::{Deserialize, Serialize};

use shogo::utils;
use wasm_bindgen::prelude::*;

use engine::board;
use engine::grids;
use engine::mesh;
use engine::move_build;
use engine::moves;
use gui::dom;

use dom::DomToWorker;
pub mod ace;
use engine::unit;
use hex;

use unit::*;

#[wasm_bindgen]
pub async fn main_entry() {
    dom::main_entry().await
}

#[wasm_bindgen]
pub async fn worker_entry2() {
    let (mut worker, mut response) = gui::worker::Worker::<AiCommand, AiResponse>::new();

    loop {
        //console_dbg!("worker:waiting22222");
        let mut res = response.next().await.unwrap();
        //console_dbg!("worker:processing:", res.game.hash_me(), res.team);
        let the_move = engine::ai::iterative_deepening(&mut res.game, &res.world, res.team);
        //console_dbg!("worker:finished processing");
        worker.post_message(AiResponse { the_move });
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AiCommand {
    game: GameState,
    world: board::MyWorld,
    team: ActiveTeam,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AiResponse {
    the_move: ActualMove,
}

#[wasm_bindgen]
pub async fn worker_entry() {
    console_error_panic_hook::set_once();

    console_dbg!("num tiles={}", hex::Cube::new(0, 0).range(4).count());

    let (mut wr, mut ss) = shogo::EngineWorker::new().await;

    let k = ss.next().await.unwrap();
    let DomToWorker::Start(game_type) = k else {
        unreachable!("worker:Didn't receive start")
    };
    wr.post_message(dom::WorkerToDom::Ack);

    console_dbg!("Found game thingy", game_type);

    let mut frame_timer = shogo::FrameTimer::new(60, ss);

    let scroll_manager = gui::scroll::TouchController::new([0., 0.].into());
    use cgmath::SquareMatrix;

    let (mut ai_worker, mut ai_response) =
        worker::WorkerInterface::<AiCommand, AiResponse>::new("./gridlock_worker2.js").await;
    console_dbg!("created ai worker");

    let last_matrix = cgmath::Matrix4::identity();
    let ctx = &utils::get_context_webgl2_offscreen(&wr.canvas());

    let grid_matrix = hex::HexConverter::new();

    let shader = gui::shader_sys::ShaderSystem::new(ctx).unwrap();

    let models = Models::new(&grid_matrix, &shader);

    let mut render = EngineStuff {
        grid_matrix,
        models,
        ctx: ctx.clone(),
        canvas: wr.canvas(),
        scroll_manager,
        last_matrix,
        shader,
    };

    let (seed, o) = if let dom::GameType::Replay(rr) = &game_type {
        let Ok(j) = engine::share::load(&rr) else {
            wr.post_message(dom::WorkerToDom::CantParseReplay);
            return;
        };

        (j.seed.clone(), Some(j))
    } else {
        (board::WorldSeed::new(), None)
    };

    let world = board::MyWorld::new(seed);

    let (command_sender, mut command_recv) = futures::channel::mpsc::channel(5);
    let (mut response_sender, response_recv) = futures::channel::mpsc::channel(5);

    let render_thead = async {
        while let Some(ace::GameWrap {
            mut game,
            data,
            team,
        }) = command_recv.next().await
        {
            //if let Command::
            let data = if let ace::Command::WaitAI = data {
                console_dbg!("render:sending ai");
                //send ai worker game
                ai_worker.post_message(AiCommand {
                    game: game.clone(),
                    world: world.clone(),
                    team,
                });
                //select on both
                use futures::FutureExt;

                let aaa = async {
                    render_command(
                        data,
                        &mut game,
                        team,
                        &mut render,
                        &world,
                        &mut frame_timer,
                        &mut wr,
                    )
                    .await
                };
                let k = futures::select!(
                    _ = aaa.fuse()=>unreachable!(),
                    x = ai_response.next() => x
                );
                //console_dbg!("render:finished ai");
                ace::Response::AiFinish(k.unwrap().the_move)
            } else {
                render_command(
                    data,
                    &mut game,
                    team,
                    &mut render,
                    &world,
                    &mut frame_timer,
                    &mut wr,
                )
                .await
            };

            response_sender
                .send(ace::GameWrap { game, data, team })
                .await
                .unwrap();
            //console_dbg!("send response!");
        }
    };

    let gameplay_thread = async {
        let mut doop = ace::WorkerManager {
            sender: command_sender,
            receiver: response_recv,
        };

        if let Some(fff) = o {
            let game_history = ace::replay(&world, doop, fff).await;

            return game_history;
        }

        let mut game = unit::game_init(&world);

        let mut game_history = engine::MoveHistory::new();

        let mut team_gen = ActiveTeam::Black.iter();

        //doop.send_command(ActiveTeam::Dogs, &mut game, Command::HideUndo).await;

        //Loop over each team!
        loop {
            let team = team_gen.next().unwrap();

            if let Some(g) = game.game_is_over(&world, team) {
                console_dbg!("Game over=", g);
                break (g, game_history);
                //break 'game_loop;
            }

            //Add AIIIIII.
            let foo = match game_type {
                dom::GameType::SinglePlayer => team == ActiveTeam::Black,
                dom::GameType::PassPlay => false,
                dom::GameType::AIBattle => true,
                dom::GameType::Replay(_) => todo!(),
            };

            if foo {
                //console_dbg!("original game dbg=", game.hash_me(), team);
                //console_dbg!("game:Sending ai command");
                let the_move = doop.wait_ai(team, &mut game).await;
                //console_dbg!("game:finished");

                //let the_move = ace::ai::iterative_deepening(&mut game.clone(), &world, team);
                //assert_eq!(the_move,the_move2);

                //let kk = the_move;

                let effect_m = animate_move(&the_move, team, &mut game, &world, &mut doop)
                    .await
                    .apply(team, &mut game, &world);

                // let effect_a = kk
                //     .into_attack(the_move.attackto)
                //     .animate(team, &mut game, &world, &mut doop)
                //     .await
                //     .apply(team, &mut game, &world, &effect_m);

                game_history.push((the_move, effect_m));

                let mut e = engine::ai::Evaluator::default();
                console_dbg!(
                    "Game after ai move:",
                    game.hash_me(),
                    e.absolute_evaluate(&mut game, &world, true)
                );

                continue;
            }

            let r = ace::handle_player(&mut game, &world, &mut doop, team, &mut game_history).await;
            game_history.push(r);

            let stest = serde_json::to_string(&game).unwrap();

            let mut e = engine::ai::Evaluator::default();
            console_dbg!(
                "Game after player move:",
                stest,
                game.hash_me(),
                e.absolute_evaluate(&mut game, &world, true)
            );

            // console_dbg!(
            //     "current position2:",
            //     e.absolute_evaluate(&mut game, &world, true)
            // );
        }
    };

    let ((result, game), ()) = futures::join!(gameplay_thread, render_thead);

    let result = match result {
        GameOver::WhiteWon => dom::GameOverGui::WhiteWon,
        GameOver::BlackWon => dom::GameOverGui::BlackWon,
        GameOver::Tie => dom::GameOverGui::Tie,
    };

    wr.post_message(dom::WorkerToDom::GameFinish {
        replay_string: engine::share::save(&game.into_just_move(world.seed)),
        result,
    });

    log!("Worker thread closin");
}

#[derive(Debug, Clone)]
pub enum CellSelection {
    MoveSelection(Axial, mesh::small_mesh::SmallMesh, Option<ace::HaveMoved>),
    BuildSelection(Axial),
}
impl Default for CellSelection {
    fn default() -> Self {
        CellSelection::BuildSelection(Axial::default())
    }
}

pub async fn animate_move<'a>(
    aa: &'a ActualMove,
    team: ActiveTeam,
    state: &GameState,
    world: &board::MyWorld,
    data: &mut ace::WorkerManager,
) -> &'a ActualMove {
    let end_points = state.factions.iter_end_points(world, aa.moveto);

    let mut ss = state.clone();

    let mut stack = 0;
    for (i, (dis, rest)) in end_points.into_iter().enumerate() {
        let Some((_, team2)) = rest else {
            continue;
        };

        if team2 != team {
            continue;
        }

        let unit = mesh::small_mesh::inverse(aa.moveto)
            .add(hex::Cube::from_arr(hex::OFFSETS[i]).ax.mul(dis as i8));

        data.wait_animation(
            gui::animation::AnimationCommand::Movement {
                unit,
                end: mesh::small_mesh::inverse(aa.moveto),
            },
            team,
            &mut ss,
        )
        .await;

        stack += 1;
        if let Some(_) = state.factions.get_cell_inner(aa.moveto) {
            ss.factions.remove_inner(aa.moveto);
        }
        ss.factions.add_cell_inner(aa.moveto, stack, team);
    }

    aa
}

use gui::model_parse::*;
use gui::*;
use web_sys::OffscreenCanvas;
use web_sys::WebGl2RenderingContext;
pub struct EngineStuff {
    grid_matrix: hex::HexConverter,
    models: Models<Foo<TextureGpu, ModelGpu>>,
    //numm: Numm,
    ctx: WebGl2RenderingContext,
    canvas: OffscreenCanvas,
    scroll_manager: gui::scroll::TouchController,
    last_matrix: cgmath::Matrix4<f32>,
    shader: ShaderSystem,
}

async fn render_command(
    command: ace::Command,
    game: &GameState,
    team: ActiveTeam,
    e: &mut EngineStuff,
    world: &board::MyWorld,
    frame_timer: &mut shogo::FrameTimer<
        DomToWorker,
        futures::channel::mpsc::UnboundedReceiver<DomToWorker>,
    >,
    engine_worker: &mut shogo::EngineWorker<dom::DomToWorker, dom::WorkerToDom>,
) -> ace::Response {
    //let mut x = 0.0;
    let scroll_manager = &mut e.scroll_manager;
    let last_matrix = &mut e.last_matrix;
    let ctx = &e.ctx;
    let canvas = &e.canvas;
    let grid_matrix = &e.grid_matrix;
    let models = &e.models;
    //let numm = &e.numm;

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
    let water = &models.water;
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

    let mut waiting_engine_ack = false;
    //console_dbg!(command);
    match command {
        ace::Command::HideUndo => {
            engine_worker.post_message(dom::WorkerToDom::HideUndo);
            waiting_engine_ack = true;
            //return ace::Response::Ack;
        }
        ace::Command::ShowUndo => {
            engine_worker.post_message(dom::WorkerToDom::ShowUndo);
            waiting_engine_ack = true;
            //return ace::Response::Ack;
        }
        ace::Command::Animate(ak) => match ak {
            gui::animation::AnimationCommand::Movement { unit, end } => {
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
            gui::animation::AnimationCommand::Terrain {
                pos,
                terrain_type,
                dir,
            } => {
                let (a, b) = match dir {
                    gui::animation::AnimationDirection::Up => (-5., 0.),
                    gui::animation::AnimationDirection::Down => (0., -6.), //TODO 6 to make sure there is a frame with it gone
                };
                let it = gui::animation::terrain_create(a, b);
                terrain_animation = Some((0.0, it, pos, terrain_type));
            }
        },
        ace::Command::GetMouseInputSelection { selection, grey } => {
            get_mouse_input = Some(Some((selection, grey)));
        }
        ace::Command::GetMouseInputNoSelect => get_mouse_input = Some(None),
        ace::Command::WaitAI => {}
        ace::Command::Popup(_str) => {
            todo!();
            // if str.is_empty() {
            //     engine_worker.post_message(UiButton::HidePopup);
            // } else {
            //     engine_worker.post_message(UiButton::ShowPopup(str));
            // }

            // return ace::Response::Ack;
        }
        ace::Command::Poke => {
            poking = 3;
        }
    };

    loop {
        if poking == 1 {
            console_dbg!("we poked!");
            return ace::Response::Ack;
        }
        poking = 0.max(poking - 1);

        let mut on_select = false;
        //let mut end_turn = false;
        let mut on_undo = false;
        let res = frame_timer.next().await;

        for e in res {
            match e {
                DomToWorker::Resize {
                    canvasx: _canvasx,
                    canvasy: _canvasy,
                    x,
                    y,
                } => {
                    let xx = *x as u32;
                    let yy = *y as u32;
                    canvas.set_width(xx);
                    canvas.set_height(yy);
                    ctx.viewport(0, 0, xx as i32, yy as i32);

                    viewport = [xx as f32, yy as f32];
                    log!(format!("updating viewport to be:{:?}", viewport));
                }
                DomToWorker::TouchMove { touches } => {
                    scroll_manager.on_touch_move(touches, last_matrix, viewport);
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
                DomToWorker::Undo => {
                    on_undo = true;
                }
                DomToWorker::Ack => {
                    if waiting_engine_ack {
                        return ace::Response::Ack;
                    }
                }
                DomToWorker::CanvasMouseMove { x, y } => {
                    scroll_manager.on_mouse_move([*x, *y], last_matrix, viewport);
                }

                DomToWorker::CanvasMouseDown { x, y } => {
                    scroll_manager.on_mouse_down([*x, *y]);
                }
                DomToWorker::ButtonClick => {}
                DomToWorker::ShutdownClick => todo!(),
                DomToWorker::Start(_) => todo!(),
            }
        }

        let proj = gui::projection::projection(viewport).generate();
        let view_proj = gui::projection::view_matrix(
            scroll_manager.camera(),
            scroll_manager.zoom(),
            scroll_manager.rot(),
        );

        let my_matrix = proj.chain(view_proj).generate();

        *last_matrix = my_matrix;

        let lll = my_matrix.generate(); //matrix::scale(0.0, 0.0, 0.0).generate();
        let projjj = lll.as_ref();

        let mouse_world =
            gui::scroll::mouse_to_world(scroll_manager.cursor_canvas(), &my_matrix, viewport);

        if get_mouse_input.is_some() {
            if on_undo {
                return if let Some((selection, _grey)) = get_mouse_input.unwrap() {
                    ace::Response::MouseWithSelection(selection, MouseEvent::Undo)
                } else {
                    ace::Response::Mouse(MouseEvent::Undo)
                };
                //return ace::Response::Mouse(MouseEvent::Undo);
            } else if on_select {
                let mouse: Axial = grid_matrix.center_world_to_hex(mouse_world.into());
                log!(format!("pos:{:?}", mouse));

                let data = if let Some((selection, _grey)) = get_mouse_input.unwrap() {
                    ace::Response::MouseWithSelection(selection, MouseEvent::Normal(mouse))
                } else {
                    ace::Response::Mouse(MouseEvent::Normal(mouse))
                };

                return data;
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

        scroll_manager.step();

        draw_sys.draw_clear([0.4, 0.1, 0.1, 0.0]);

        let grid_snap = |c: Axial, cc| {
            let pos = grid_matrix.hex_axial_to_world(&c);
            let t = matrix::translation(pos.x, pos.y, cc);
            t.generate()
        };

        let cell_height = models.water.height;

        draw_sys
            .batch(
                world
                    .land
                    .iter_mesh(hex::Axial::zero())
                    .map(|e| grid_snap(e, -models.water.height)),
            )
            .build(water, &projjj);

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
            if let Some((selection, grey)) = a {
                match selection {
                    CellSelection::MoveSelection(_, mesh, _) => {
                        //console_dbg!("doo=",mesh);
                        let cells = mesh.iter_mesh(Axial::zero()).map(|e| {
                            let zzzz = 0.0;

                            grid_snap(e, zzzz)
                                .chain(matrix::scale(1.0, 1.0, 1.0))
                                .generate()
                        });
                        draw_sys
                            .batch(cells)
                            .no_lighting()
                            .grey(*grey)
                            .build(select_model, &projjj);

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
                    CellSelection::BuildSelection(_) => {}
                }
            }
        }

        {
            let zzzz = 0.1;

            // Draw shadows
            let _d = DepthDisabler::new(ctx);

            let shadows = world
                .get_game_cells()
                .iter_mesh(Axial::zero())
                .filter_map(|a| {
                    if let Some((val, _)) = game.factions.get_cell(a) {
                        let xx = match val {
                            1 | 2 => 0.6,
                            3 | 4 => 0.8,
                            5 | 6 => 1.2,
                            _ => unreachable!(),
                        };
                        Some(
                            grid_snap(a, zzzz)
                                .chain(matrix::scale(xx, xx, 1.0))
                                .generate(),
                        )
                    } else {
                        None
                    }
                });

            let ani_drop_shadow = unit_animation.as_ref().map(|a| {
                let pos = a.0;
                matrix::translation(pos.x, pos.y, zzzz)
                    .chain(matrix::scale(0.6, 0.6, 1.0))
                    .generate()
            });

            let all_shadows = shadows.chain(ani_drop_shadow.into_iter());

            draw_sys.batch(all_shadows).build(drop_shadow, &projjj);
        }

        //TODO pre-allocate
        let mut white_team_cells = vec![];
        let mut black_team_cells = vec![];
        let mut neutral_team_cells = vec![];

        if let Some((pos, ..)) = &unit_animation {
            let ss = 0.4;
            //Draw it a bit lower then static ones so there is no flickering
            let first = matrix::translation(pos.x, pos.y, 1.0)
                .chain(matrix::scale(ss, ss, 1.0))
                .generate();

            match team {
                ActiveTeam::White => {
                    white_team_cells.push(first);
                }
                ActiveTeam::Black => {
                    black_team_cells.push(first);
                }
                ActiveTeam::Neutral => {
                    neutral_team_cells.push(first);
                }
            }
        }

        //x += 0.1;
        for a in world.get_game_cells().iter_mesh(Axial::zero()) {
            if let Some((val, team2)) = game.factions.get_cell(a) {
                let inner_stack = val.min(2);
                let mid_stack = val.max(2).min(4) - 2;
                let outer_stack = val.max(4) - 4;
                let arr = match team2 {
                    ActiveTeam::White => &mut white_team_cells,
                    ActiveTeam::Black => &mut black_team_cells,
                    ActiveTeam::Neutral => &mut neutral_team_cells,
                };

                let radius = [0.4, 0.6, 0.8];

                for (stack, radius) in [inner_stack, mid_stack, outer_stack].iter().zip(radius) {
                    for k in 0..*stack {
                        arr.push(
                            grid_snap(a, k as f32 * cell_height)
                                .chain(matrix::scale(radius, radius, 1.0))
                                .generate(),
                        );
                    }
                }
                // for k in 0..inner_stack {
                //     arr.push(
                //         grid_snap(a, k as f32 * cell_height)
                //             .chain(matrix::scale(0.5, 0.5, 1.0))
                //             .generate(),
                //     );
                // }

                // for k in 0..outer_stack {
                //     arr.push(
                //         grid_snap(a, k as f32 * cell_height)
                //             .chain(matrix::scale(0.8, 0.8, 1.0))
                //             .generate(),
                //     );
                // }
            }
        }

        draw_sys
            .batch(white_team_cells)
            .build(&models.snow, &projjj);
        draw_sys
            .batch(black_team_cells)
            .build(&models.grass, &projjj);

        draw_sys
            .batch(neutral_team_cells)
            .build(&models.water, &projjj);

        // draw_unit_type(
        //     UnitType::Mouse,
        //     ActiveTeam::White,
        //     &game.factions.white.mouse,
        //     &models.grass,
        // );

        // draw_unit_type(
        //     UnitType::Mouse,
        //     ActiveTeam::Black,
        //     &game.factions.black.mouse,
        //     &models.snow,
        // );

        // let d = DepthDisabler::new(ctx);

        // draw_health_text(
        //     game.factions
        //         .cats
        //         .iter()
        //         .map(|x| (x.position, x.typ.type_index() as i8))
        //         .chain(
        //             game.factions
        //                 .dogs
        //                 .iter()
        //                 .map(|x| (x.position, x.typ.type_index() as i8)),
        //         ),
        //     grid_matrix,
        //     &numm.health_numbers,
        //     &view_proj,
        //     &proj,
        //     &mut draw_sys,
        //     &numm.text_texture,
        // );
        // drop(d);

        ctx.flush();
    }
}
