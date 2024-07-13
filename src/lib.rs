use cgmath::{InnerSpace, Matrix4, Transform, Vector2};
use gloo::console::console_dbg;

use futures::{SinkExt, StreamExt};
use gloo::console::log;
use hex::{Axial, HDir};
use mesh::bitfield::BitField;
use mesh::small_mesh::SingleMesh;
use model::matrix::{self, MyMatrix};
use moves::ActualMove;
use serde::{Deserialize, Serialize};
use shader_sys::ShaderSystem;

use shogo::utils;
use wasm_bindgen::prelude::*;

pub mod animation;
pub mod board;
pub mod dom;
pub mod grids;
pub mod mesh;
pub mod model_parse;
pub mod move_build;
pub mod moves;
pub mod projection;
pub mod scroll;
pub mod shader_sys;
pub mod util;
pub mod worker;
use dom::DomToWorker;
use projection::*;
pub mod ace;
pub mod hex;
pub mod unit;

use unit::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum WorkerToDom {
    ShowUndo,
    HideUndo,
    GameFinish {
        replay_string: String,
        result: GameOver,
    },
    CantParseReplay,
    ReplayFinish,
    Ack,
}

#[wasm_bindgen]
pub async fn worker_entry2() {
    let (mut worker, mut response) = worker::Worker::<AiCommand, AiResponse>::new();

    loop {
        console_dbg!("worker:waiting");
        let mut res = response.next().await.unwrap();
        let the_move = ace::ai::iterative_deepening(&mut res.game, &res.world, res.team);
        console_dbg!("worker:processing");

        console_dbg!("worker:finished processing");
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
    wr.post_message(WorkerToDom::Ack);

    console_dbg!("Found game thingy", game_type);

    let mut frame_timer = shogo::FrameTimer::new(60, ss);

    let scroll_manager = scroll::TouchController::new([0., 0.].into());
    use cgmath::SquareMatrix;

    let (mut ai_worker, mut ai_response) =
        worker::WorkerInterface::<AiCommand, AiResponse>::new("./gridlock_worker2.js").await;

    console_dbg!("created ai worker");

    let last_matrix = cgmath::Matrix4::identity();
    let ctx = &utils::get_context_webgl2_offscreen(&wr.canvas());

    let grid_matrix = grids::HexConverter::new();

    let shader = shader_sys::ShaderSystem::new(ctx).unwrap();

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
        let Ok(j) = ace::share::load(&rr) else {
            wr.post_message(WorkerToDom::CantParseReplay);
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

                console_dbg!("render:finished ai");
                ace::Response::AiFinish(k.expect("AI workerresponse error?").the_move)
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
            console_dbg!("send response!");
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

        let mut game = ace::game_init(&world);

        let mut game_history = ace::selection::MoveHistory::new();

        let mut team_gen = ActiveTeam::White.iter();

        //doop.send_command(ActiveTeam::Dogs, &mut game, Command::HideUndo).await;

        //Loop over each team!
        loop {
            let team = team_gen.next().unwrap();

            if let Some(g) = game.game_is_over(&world) {
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
                //ai_worker.post_message(AiCommand{game:game.clone(),world:world.clone(),team});
                console_dbg!("game:Sending ai command");

                let the_move = doop.wait_ai(team, &mut game).await;
                //let the_move = ace::ai::iterative_deepening(&mut game, &world, team);

                //let the_move=

                console_dbg!("game:finished");
                // use futures::FutureExt;
                // let mut jj=game.clone();
                // let k=futures::select!(
                //     _ = doop.do_nothing( team, &mut jj).fuse()=>unreachable!(),
                //     x = ai_response.next() => x
                // );
                // console_dbg!("GOT RESPONSEEEE");

                //let k=ai_response.next().await;
                //let the_move = k.unwrap().the_move;

                //let the_move = ace::ai::iterative_deepening(&mut game.clone(), &world, team);

                let kk = the_move.as_move();

                let effect_m = kk
                    .animate(team, &mut game, &world, &mut doop)
                    .await
                    .apply(team, &mut game, &world);

                let effect_a = kk
                    .into_attack(the_move.attackto)
                    .animate(team, &mut game, &world, &mut doop)
                    .await
                    .apply(team, &mut game, &world, &effect_m);

                game_history.push((the_move, effect_m.combine(effect_a)));

                continue;
            }

            let r = ace::handle_player(&mut game, &world, &mut doop, team, &mut game_history).await;
            game_history.push(r);

            let mut e = ace::ai::Evaluator::default();
            e.absolute_evaluate(&mut game, &world, true);
        }
    };

    let ((result, game), ()) = futures::join!(gameplay_thread, render_thead);

    wr.post_message(WorkerToDom::GameFinish {
        replay_string: ace::share::save(&game.into_just_move(world.seed)),
        result,
    });

    log!("Worker thread closin");
}

pub struct EngineStuff {
    grid_matrix: grids::HexConverter,
    models: Models<Foo<TextureGpu, ModelGpu>>,
    //numm: Numm,
    ctx: WebGl2RenderingContext,
    canvas: OffscreenCanvas,
    scroll_manager: scroll::TouchController,
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
    engine_worker: &mut shogo::EngineWorker<DomToWorker, WorkerToDom>,
) -> ace::Response {
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
    let black_rook = &models.black_rook;
    let white_rook = &models.white_rook;
    let black_knight = &models.black_knight;
    let white_knight = &models.white_knight;
    let black_bishop = &models.black_bishop;
    let white_bishop = &models.white_bishop;

    let black_pawn = &models.black_pawn;
    let white_pawn = &models.white_pawn;

    //let fog_asset = &models.fog;
    let water = &models.water;
    let grass = &models.grass;
    let mountain_asset = &models.mountain;
    let snow = &models.snow;
    let select_model = &models.select_model;
    let attack_model = &models.attack;

    //First lets process the command. Break it down
    //into pieces that this thread understands.
    let mut get_mouse_input = None;
    let mut unit_animation = None;
    let mut terrain_animation = None;
    let mut poking = 0;

    let mut waiting_engine_ack = false;
    let mut rr = 0.0;
    match command {
        ace::Command::HideUndo => {
            engine_worker.post_message(WorkerToDom::HideUndo);
            waiting_engine_ack = true;
            //return ace::Response::Ack;
        }
        ace::Command::ShowUndo => {
            engine_worker.post_message(WorkerToDom::ShowUndo);
            waiting_engine_ack = true;
            //return ace::Response::Ack;
        }
        ace::Command::Animate(ak) => match ak {
            animation::AnimationCommand::Movement {
                unit,
                ttt,
                end,
                path,
                data,
            } => {
                let ff = match data {
                    move_build::PushInfo::PushedLand => {
                        Some(animation::land_delta(unit, end, grid_matrix))
                    }
                    move_build::PushInfo::UpgradedLand => {
                        todo!("BLAP");
                    }
                    move_build::PushInfo::PushedUnit => {
                        todo!("BLAP");
                    }

                    move_build::PushInfo::None => None,
                };

                let it = path.animation_iter(unit, grid_matrix);

                unit_animation = Some((Vector2::new(0.0, 0.0), it, unit, ttt, ff));
            }
            animation::AnimationCommand::Terrain {
                pos,
                terrain_type,
                dir,
            } => {
                let (a, b) = match dir {
                    animation::AnimationDirection::Up => (-5., 0.),
                    animation::AnimationDirection::Down => (0., -6.), //TODO 6 to make sure there is a frame with it gone
                };
                let it = animation::terrain_create(a, b);
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

    //TODO don't calculate 60 times a second?
    let visible_water = {
        // let mut g = game.env.terrain.gen_all_terrain();
        // g.union_with(&game.env.fog);

        // g.toggle_range(..);

        // ace::ai::expand_mesh(&mut g, &mut BitField::new());
        // let k = BitField::from_iter(world.get_game_cells().iter_mesh(Axial::zero()));

        // g.intersect_with(&k);

        // g
        //world.get_game_cells();
        let mut foo = SingleMesh::new();
        for q in 0..8 {
            for r in 0..8 {
                if q % 2 == 0 && r % 2 == 0 || q % 2 == 1 && r % 2 == 1 {
                    foo.add(Axial { q, r })
                }
            }
        }
        foo
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
                    if let scroll::MouseUp::Select = scroll_manager.on_touch_up(touches) {
                        on_select = true;
                    }
                }
                DomToWorker::CanvasMouseLeave => {
                    log!("mouse leaving!");
                    let _ = scroll_manager.on_mouse_up();
                }
                DomToWorker::CanvasMouseUp => {
                    if let scroll::MouseUp::Select = scroll_manager.on_mouse_up() {
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

        let proj = projection::projection(viewport).generate();
        let view_proj = projection::view_matrix(
            scroll_manager.camera(),
            scroll_manager.zoom(),
            scroll_manager.rot(),
        );

        let my_matrix = proj.chain(view_proj).generate();

        *last_matrix = my_matrix;

        let mouse_world =
            scroll::mouse_to_world(scroll_manager.cursor_canvas(), &my_matrix, viewport);

        if get_mouse_input.is_some() {
            if on_undo {
                return if let Some((selection, _grey)) = get_mouse_input.unwrap() {
                    ace::Response::MouseWithSelection(selection, MouseEvent::Undo)
                } else {
                    ace::Response::Mouse(MouseEvent::Undo)
                };
                //return ace::Response::Mouse(MouseEvent::Undo);
            } else if on_select {
                let mouse: Axial = grid_matrix.world_to_hex(mouse_world.into());
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
        if let Some((lpos, a, _, _, _data)) = &mut unit_animation {
            if let Some(pos) = a.next() {
                *lpos = pos;
            } else {
                return ace::Response::AnimationFinish;
            }
        }

        scroll_manager.step();

        draw_sys.draw_clear([0.5, 0.5, 0.5, 0.0]);

        pub const LAND_OFFSET: f32 = -10.0;
        rr += 0.01;

        let grid_snap = |c: Axial, cc| {
            let pos = grid_matrix.hex_axial_to_world(&c);
            let t = matrix::translation(pos.x, pos.y, cc);
            //let r=matrix::z_rotation(1.0);
            my_matrix.chain(t).//.chain(matrix::z_rotation(rr)).
            generate()
        };
        let mut foo = false;
        // draw_sys
        //     .batch(visible_water.iter_mesh().map(|e| grid_snap(e, 0.0)))
        //     .build(water);

        {
            //Draw grass
            let grass1 = game
                .env
                .terrain
                .land
                .iter_mesh()
                .map(|e| grid_snap(e, LAND_OFFSET));

            let ani_grass = if let Some((zpos, _, gpos, k)) = &terrain_animation {
                if let animation::TerrainType::Grass = k {
                    let gpos = *gpos;

                    let pos = grid_matrix.hex_axial_to_world(&gpos);

                    let t = matrix::translation(pos.x, pos.y, LAND_OFFSET + *zpos);
                    let m = my_matrix.chain(t).generate();
                    Some(m)
                } else {
                    None
                }
            } else {
                None
            };

            let push_grass = if let Some((pos, _, _unit, _, data)) = &unit_animation {
                if let Some(f) = data {
                    let kk = pos + f;
                    let m = my_matrix
                        .chain(matrix::translation(kk.x, kk.y, LAND_OFFSET))
                        .chain(matrix::scale(1.0, 1.0, 1.0))
                        .generate();
                    Some(m)
                } else {
                    None
                }
            } else {
                None
            };

            let all_grass = grass1
                .chain(ani_grass.into_iter())
                .chain(push_grass.into_iter());

            draw_sys.batch(all_grass).build(grass);
        }

        {
            //Draw forest
            let grass1 = game
                .env
                .terrain
                .forest
                .iter_mesh()
                .map(|e| grid_snap(e, LAND_OFFSET));

            let all_grass = grass1;

            draw_sys.batch(all_grass).build(mountain_asset);
        }

        {
            //Draw mountain
            let grass1 = game
                .env
                .terrain
                .mountain
                .iter_mesh()
                .map(|e| grid_snap(e, 0.0));

            let all_grass = grass1;

            draw_sys.batch(all_grass).build(mountain_asset);
        }

        {
            //Draw fog
            let fog1 = game.env.fog.iter_mesh().map(|e| grid_snap(e, LAND_OFFSET));

            let ani_fog = if let Some((zpos, _, gpos, k)) = &terrain_animation {
                if let animation::TerrainType::Fog = k {
                    let gpos = *gpos;

                    let pos = grid_matrix.hex_axial_to_world(&gpos);

                    let t = matrix::translation(pos.x, pos.y, LAND_OFFSET + *zpos);
                    let m = my_matrix.chain(t).generate();
                    Some(m)
                } else {
                    None
                }
            } else {
                None
            };

            let all_fog = fog1.chain(ani_fog.into_iter());

            draw_sys.batch(all_fog).build(snow);
        }

        if let Some(a) = &get_mouse_input {
            if let Some((selection, grey)) = a {
                match selection {
                    CellSelection::MoveSelection(point, mesh, hh) => {
                        let cells = mesh.iter_mesh(Axial::zero()).map(|e| grid_snap(e, 2.0));
                        draw_sys
                            .batch(cells)
                            .no_lighting()
                            .grey(*grey)
                            .build(select_model);

                        // {
                        //     let mut mesh=mesh::small_mesh::SmallMesh::new();
                        //     game.attack_mesh_add(&mut mesh, world, point, team,true);
                        //     //console_dbg!(mesh.iter_mesh(*point).count());
                        //     let cells = mesh.iter_mesh(*point).filter(|d|world.get_game_cells().is_set(*d)).map(|e| {

                        //         grid_snap(e, -8.0)
                        //     });
                        //     draw_sys
                        //         .batch(cells)
                        //         .no_lighting()
                        //         .grey(*grey)
                        //         .build(attack_model);
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
                    CellSelection::BuildSelection(_) => {}
                }
            }
        }

        {
            let pos = mouse_world;

            let t = matrix::translation(pos[0], pos[1], 2.0);

            let m = my_matrix.chain(t).//.chain(matrix::z_rotation(std::f32::consts::TAU/6.0)).
                generate();

            draw_sys.batch([m]).no_lighting().build(attack_model);
        }

        // {
        //     let mouse: Axial = grid_matrix.world_to_hex(mouse_world.into());
        //     let m=grid_snap(mouse,-9.0);

        //     draw_sys.batch([m]).no_lighting().build(water);
        // }

        let zzzz = 1.;
        {
            // Draw shadows
            let _d = DepthDisabler::new(ctx);

            let shadows = game
                .factions
                .units
                .all_units()
                .intersect(&game.factions.parity.not())
                .iter_mesh()
                .map(|e| grid_snap(e, zzzz));

            let ani_drop_shadow = unit_animation.as_ref().map(|a| {
                let pos = a.0;
                my_matrix
                    .chain(matrix::translation(pos.x, pos.y, zzzz))
                    .generate()
            });

            let all_shadows = shadows.chain(ani_drop_shadow.into_iter());

            draw_sys.batch(all_shadows).build(drop_shadow);
        }

        // draw_sys
        //     .batch(game.factions.parity.iter_mesh().map(|c| {
        //         let pos = grid_matrix.hex_axial_to_world(&c);
        //         let t = matrix::translation(pos.x, pos.y, 0.0);
        //         //let r=matrix::z_rotation(1.0);
        //         my_matrix.chain(t).chain(matrix::scale(0.5,0.5,0.5)).//.chain(matrix::z_rotation(rr)).
        //         generate()
        //     }))
        //     .build(grass);

        let zzzz = 0.;
        let mut draw_unit_type =
            |mytype: UnitType, my_team: ActiveTeam, model: &Foo<TextureGpu, ModelGpu>| {
                // let i = match mytype {
                //     UnitType::Book(Parity::One) => 2,
                //     UnitType::Book(Parity::Two) => 1,
                //     UnitType::Pawn | UnitType::King => {
                //         if let ActiveTeam::White = my_team {
                //             2
                //         } else {
                //             5
                //         }
                //     }
                //     UnitType::Knight(Parity::One) => 2,
                //     UnitType::Knight(Parity::Two) => 1,
                //     UnitType::Rook => 0,
                //     UnitType::Trook(TrookParity::One) => 3,
                //     UnitType::Trook(TrookParity::Two) => 2,
                //     UnitType::Trook(TrookParity::Three) => 1,
                // };

                let foo = game.factions.specific_unit(mytype, my_team);

                //let rr = (std::f32::consts::TAU / 6.0) * i as f32;

                let color = foo.iter_mesh().map(|e| {
                    // let rr=if e.q>=0{
                    //     0.0
                    // }else{
                    //     std::f32::consts::PI
                    // };
                    let c = e;

                    let (cc, rr) = if game.factions.parity.is_set(e) {
                        (-1.0, std::f32::consts::PI)
                    } else {
                        (1.0, 0.0)
                    };
                    // let c = if e.q>=0{
                    //     e
                    // }else{
                    //     e.add(Axial{q:8,r:0})
                    // };
                    //let cc = zzzz;
                    let pos = grid_matrix.hex_axial_to_world(&c);
                    let t = matrix::translation(pos.x, pos.y, cc);

                    my_matrix.chain(t).chain(matrix::x_rotation(rr)).generate()
                });

                let ani = unit_animation
                    .as_ref()
                    .filter(|(pos, _, unit, ttt, _data)| {
                        *ttt == mytype && team == my_team
                        //foo.is_set(*unit)
                    })
                    .map(|(pos, _, unit, _, _data)| {
                        my_matrix
                            .chain(matrix::translation(pos.x, pos.y, zzzz))
                            .chain(matrix::scale(1.0, 1.0, 1.0))
                            .generate()
                    });

                let k = color.chain(ani.into_iter());

                draw_sys.batch(k).build(model)
            };

        //TODO combine into one draw call

        draw_unit_type(UnitType::Rook, ActiveTeam::White, white_rook);
        draw_unit_type(UnitType::Rook, ActiveTeam::Black, black_rook);

        draw_unit_type(UnitType::Pawn, ActiveTeam::White, white_pawn);
        draw_unit_type(UnitType::Pawn, ActiveTeam::Black, black_pawn);

        draw_unit_type(UnitType::Knight, ActiveTeam::White, white_knight);
        draw_unit_type(UnitType::Knight, ActiveTeam::Black, black_knight);

        draw_unit_type(UnitType::Bishop, ActiveTeam::White, white_bishop);
        draw_unit_type(UnitType::Bishop, ActiveTeam::White, white_bishop);

        draw_unit_type(UnitType::King, ActiveTeam::White, &models.white_king);
        draw_unit_type(UnitType::King, ActiveTeam::Black, &models.black_king);

        draw_unit_type(UnitType::Queen, ActiveTeam::Black, &models.black_trook);
        draw_unit_type(UnitType::Queen, ActiveTeam::White, &models.white_trook);

        draw_sys
            .batch(visible_water.iter_mesh().map(|e| grid_snap(e, 0.0)))
            .build(water);

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

#[must_use]
pub struct BatchBuilder<'a, I> {
    sys: &'a mut ShaderSystem,
    ff: I,
    lighting: bool,
    grey: bool,
}
impl<I: Iterator<Item = K>, K: MyMatrix> BatchBuilder<'_, I> {
    pub fn build(&mut self, texture: &Foo<TextureGpu, ModelGpu>) {
        let mmatrix: Vec<[f32; 16]> = (&mut self.ff)
            .map(|x| {
                let x = x.generate();
                let x: &[f32; 16] = x.as_ref();
                *x
            })
            .collect();

        self.sys.draw(
            &texture.model.res,
            &texture.texture.texture,
            &mmatrix,
            self.grey,
            false,
            self.lighting,
        );
    }
    pub fn grey(&mut self, grey: bool) -> &mut Self {
        self.grey = grey;
        self
    }
    pub fn no_lighting(&mut self) -> &mut Self {
        self.lighting = false;
        self
    }
}

impl Doop for ShaderSystem {
    fn batch<K: MyMatrix, I>(&mut self, ff: I) -> BatchBuilder<'_, I::IntoIter>
    where
        I: IntoIterator<Item = K>,
    {
        BatchBuilder {
            sys: self,
            ff: ff.into_iter(),
            lighting: true,
            grey: false,
        }
    }
}

//TODO get rid of this interface??
pub trait Doop {
    fn batch<K: MyMatrix, I>(&mut self, ff: I) -> BatchBuilder<'_, I::IntoIter>
    where
        I: IntoIterator<Item = K>;
}

// fn draw_health_text(
//     f: impl IntoIterator<Item = (Axial, i8)>,

//     gg: &grids::HexConverter,
//     health_numbers: &NumberTextManager,
//     view_proj: &Matrix4<f32>,
//     proj: &Matrix4<f32>,
//     draw_sys: &mut ShaderSystem,
//     text_texture: &TextureGpu,
// ) {
//     //draw text
//     for (ccat, ii) in f {
//         let pos = gg.hex_axial_to_world(&ccat);

//         let t = matrix::translation(pos.x, pos.y + 20.0, 20.0);

//         let jj = view_proj.chain(t).generate();
//         let jj: &[f32; 16] = jj.as_ref();
//         let tt = matrix::translation(jj[12], jj[13], jj[14]);
//         let new_proj = (*proj).chain(tt);

//         let s = matrix::scale(5.0, 5.0, 5.0);
//         let m = new_proj.chain(s).generate();

//         let nn = health_numbers.get_number(ii, text_texture);
//         draw_sys
//             .view(&m)
//             .draw_a_thing_ext(&nn, false, false, true, false);
//     }
// }

pub struct DepthDisabler<'a> {
    ctx: &'a WebGl2RenderingContext,
}
impl<'a> Drop for DepthDisabler<'a> {
    fn drop(&mut self) {
        self.ctx.enable(WebGl2RenderingContext::DEPTH_TEST);
        self.ctx.enable(WebGl2RenderingContext::CULL_FACE);
    }
}
impl<'a> DepthDisabler<'a> {
    pub fn new(ctx: &'a WebGl2RenderingContext) -> DepthDisabler<'a> {
        ctx.disable(WebGl2RenderingContext::DEPTH_TEST);
        ctx.disable(WebGl2RenderingContext::CULL_FACE);

        DepthDisabler { ctx }
    }
}

use web_sys::{OffscreenCanvas, WebGl2RenderingContext};

use crate::ace::MouseEvent;
use crate::model_parse::{Foo, ModelGpu, TextureGpu};

//purple #6d32e7
//orange #ff8100

//TODO move this to a seperate crate
pub struct Models<T> {
    select_model: T,
    drop_shadow: T,
    fog: T,
    attack: T,
    white_rook: T,
    black_rook: T,
    white_pawn: T,
    black_pawn: T,
    white_knight: T,
    black_knight: T,
    white_bishop: T,
    black_bishop: T,
    white_king: T,
    black_king: T,
    grass: T,
    snow: T,
    water: T,
    direction: T,
    mountain: T,
    black_trook: T,
    white_trook: T,
}

impl Models<Foo<TextureGpu, ModelGpu>> {
    pub fn new(grid_matrix: &grids::HexConverter, shader: &ShaderSystem) -> Self {
        let quick_load = |name, res, alpha| {
            let (data, t) = model::load_glb(name).gen_ext(grid_matrix.spacing(), res, alpha);

            log!(format!("texture:{:?}", (t.width, t.height)));

            model_parse::Foo {
                texture: model_parse::TextureGpu::new(&shader.ctx, &t),
                model: model_parse::ModelGpu::new(shader, &data),
            }
        };

        pub const RESIZE: usize = 10;

        Models {
            select_model: quick_load(include_bytes!("../assets/select_model.glb"), 1, None),
            drop_shadow: quick_load(include_bytes!("../assets/drop_shadow.glb"), 1, Some(0.5)),
            fog: quick_load(include_bytes!("../assets/fog.glb"), RESIZE, None),
            attack: quick_load(include_bytes!("../assets/attack.glb"), 1, None),
            white_rook: quick_load(
                include_bytes!("../assets/white_chess_rook.glb"),
                RESIZE,
                None,
            ),
            black_rook: quick_load(
                include_bytes!("../assets/black_chess_rook.glb"),
                RESIZE,
                None,
            ),
            white_king: quick_load(include_bytes!("../assets/white_king.glb"), RESIZE, None),
            black_king: quick_load(include_bytes!("../assets/black_king.glb"), RESIZE, None),
            white_knight: quick_load(
                include_bytes!("../assets/chess_white_knight.glb"),
                RESIZE,
                None,
            ),
            black_knight: quick_load(
                include_bytes!("../assets/chess_black_knight.glb"),
                RESIZE,
                None,
            ),
            white_bishop: quick_load(include_bytes!("../assets/white_bishop.glb"), RESIZE, None),
            black_bishop: quick_load(include_bytes!("../assets/black_bishop.glb"), RESIZE, None),
            white_pawn: quick_load(include_bytes!("../assets/white_pawn.glb"), RESIZE, None),
            black_pawn: quick_load(include_bytes!("../assets/black_pawn.glb"), RESIZE, None),
            //grass: quick_load(include_bytes!("../assets/square_grass.glb"), RESIZE, None),
            grass: quick_load(include_bytes!("../assets/hex-grass.glb"), RESIZE, None),

            snow: quick_load(include_bytes!("../assets/snow.glb"), RESIZE, None),
            //water: quick_load(include_bytes!("../assets/water.glb"), RESIZE, None),
            water: quick_load(include_bytes!("../assets/chess_cell.glb"), RESIZE, None),

            direction: quick_load(include_bytes!("../assets/direction.glb"), 1, None),
            mountain: quick_load(include_bytes!("../assets/mountain.glb"), 1, None),
            black_trook: quick_load(include_bytes!("../assets/trook_black.glb"), RESIZE, None),
            white_trook: quick_load(include_bytes!("../assets/trook_white.glb"), RESIZE, None),
        }
    }
}

// pub struct Numm {
//     text_texture: TextureGpu,
//     health_numbers: NumberTextManager,
// }
// impl Numm {
//     pub fn new(ctx: &WebGl2RenderingContext) -> Self {
//         let text_texture = {
//             let ascii_tex = model::load_texture_from_data(include_bytes!("../assets/ascii5.png"));

//             model_parse::TextureGpu::new(ctx, &ascii_tex)
//         };

//         let health_numbers = NumberTextManager::new(ctx);

//         Numm {
//             text_texture,
//             health_numbers,
//         }
//     }
// }

// pub struct NumberTextManager {
//     pub numbers: Vec<model_parse::ModelGpu>,
// }
// impl NumberTextManager {
//     fn new(ctx: &WebGl2RenderingContext) -> Self {
//         let range = -10..=10;
//         fn generate_number(number: i8, ctx: &WebGl2RenderingContext) -> model_parse::ModelGpu {
//             let data = string_to_coords(&format!("{}", number));
//             model_parse::ModelGpu::new(ctx, &data)
//         }

//         let numbers = range.into_iter().map(|i| generate_number(i, ctx)).collect();
//         Self { numbers }
//     }

//     pub fn get_number<'b>(
//         &self,
//         num: i8,
//         texture: &'b model_parse::TextureGpu,
//     ) -> model_parse::Foo<&'b model_parse::TextureGpu, &model_parse::ModelGpu> {
//         let gpu = &self.numbers[(num + 10) as usize];

//         model_parse::Foo {
//             texture,
//             model: gpu,
//         }
//     }
// }

fn string_to_coords<'a>(st: &str) -> model::ModelData {
    let num_rows = 16;
    let num_columns = 16;

    let mut tex_coords = vec![];
    let mut counter = 0.0;
    let dd = 20.0;
    let mut positions = vec![];

    let mut inds = vec![];
    for (_, a) in st.chars().enumerate() {
        let ascii = a as u8;
        let index = ascii as u16;

        let x = (index % num_rows) as f32 / num_rows as f32;
        let y = (index / num_rows) as f32 / num_columns as f32;

        let x1 = x;
        let x2 = x1 + 1.0 / num_rows as f32;

        let y1 = y;
        let y2 = y + 1.0 / num_columns as f32;

        let a = [[x1, y1], [x2, y1], [x1, y2], [x2, y2]];

        tex_coords.extend(a);

        let iii = [0u16, 1, 2, 2, 1, 3].map(|a| positions.len() as u16 + a);

        let xx1 = counter;
        let xx2 = counter + dd;
        let yy1 = dd;
        let yy2 = 0.0;

        let zz = 0.0;
        let y = [
            [xx1, yy1, zz],
            [xx2, yy1, zz],
            [xx1, yy2, zz],
            [xx2, yy2, zz],
        ];

        positions.extend(y);

        inds.extend(iii);

        assert!(ascii >= 32);
        counter += dd;
    }

    let normals = positions.iter().map(|_| [0.0, 0.0, 1.0]).collect();

    let cc = 1.0 / dd;
    let mm = matrix::scale(cc, cc, cc).generate();

    let positions = positions
        .into_iter()
        .map(|a| mm.transform_point(a.into()).into())
        .collect();

    model::ModelData {
        positions,
        tex_coords,
        indices: Some(inds),
        normals,
        matrix: mm,
    }
}
