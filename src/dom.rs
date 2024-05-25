use super::*;

///Common data sent from the main thread to the worker.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DomToWorker {
    CanvasMouseMove {
        x: f32,
        y: f32,
    },
    CanvasMouseDown {
        x: f32,
        y: f32,
    },
    TouchMove {
        touches: scroll::Touches,
    },
    TouchDown {
        touches: scroll::Touches,
    },
    TouchEnd {
        touches: scroll::Touches,
    },
    Start(GameType),
    Undo,
    Ack,
    CanvasMouseUp,
    CanvasMouseLeave,
    ButtonClick,
    ShutdownClick,
    Resize {
        canvasx: u32,
        canvasy: u32,
        x: f32,
        y: f32,
    },
}
impl DomToWorker {
    fn some(self) -> Option<Self> {
        Some(self)
    }
}

//convert DOM coordinate to canvas relative coordinate
fn convert_coord(canvas: &web_sys::EventTarget, event: &web_sys::Event) -> [f32; 2] {
    shader_sys::convert_coord(
        canvas.dyn_ref().unwrap_throw(),
        event.dyn_ref().unwrap_throw(),
    )
}

fn convert_coord_touch(canvas: &web_sys::EventTarget, event: &web_sys::Event) -> scroll::Touches {
    event.prevent_default();
    event.stop_propagation();
    convert_coord_touch_inner(canvas, event.dyn_ref().unwrap_throw())
}

///
/// Convert a mouse event to a coordinate for simple2d.
///
pub fn convert_coord_touch_inner(
    canvas: &web_sys::EventTarget,
    e: &web_sys::TouchEvent,
) -> scroll::Touches {
    let canvas: &web_sys::HtmlElement = canvas.dyn_ref().unwrap_throw();
    let rect = canvas.get_bounding_client_rect();

    let canvas_width: f64 = canvas
        .get_attribute("width")
        .unwrap_throw()
        .parse()
        .unwrap_throw();
    let canvas_height: f64 = canvas
        .get_attribute("height")
        .unwrap_throw()
        .parse()
        .unwrap_throw();

    let scalex = canvas_width / rect.width();
    let scaley = canvas_height / rect.height();

    let touches = e.touches();

    let mut k = scroll::Touches {
        all: [(0, 0.0, 0.0); 4],
        count: 0,
    };

    for a in (0..touches.length()).take(4) {
        let touch = touches.get(a).unwrap();
        let x = touch.client_x() as f64;
        let y = touch.client_y() as f64;
        //let rx = touch.radius_x() as f64;
        //let ry = touch.radius_y() as f64;
        let [x, y] = [
            (x * scalex - rect.left() * scalex),
            (y * scaley - rect.top() * scaley),
        ];

        let id = touch.identifier();
        k.all[k.count] = (id, x as f32, y as f32);
        k.count += 1;
        //ans.push([x as f32, y as f32]);
    }
    k
}

pub fn text_texture(text: &str, width: usize, height: usize) -> web_sys::HtmlCanvasElement {
    let canvas = gloo::utils::document().create_element("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let ctx = canvas.get_context("2d").unwrap().unwrap();
    let ctx: web_sys::CanvasRenderingContext2d = ctx.dyn_into().unwrap();

    canvas.set_width(width as u32);
    canvas.set_height(height as u32);
    ctx.set_font(text);
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    ctx.set_fill_style(&JsValue::from_str("black"));
    ctx.clear_rect(0., 0., width as f64, height as f64);
    ctx.fill_text(text, width as f64 / 2., height as f64 / 2.)
        .unwrap();
    canvas
}

fn engine_handlers(
    worker: &mut shogo::EngineMain<DomToWorker, WorkerToDom>,
    canvas: &web_sys::HtmlCanvasElement,
) -> [gloo::events::EventListener; 10] {
    [
        worker.register_event(canvas, "mousemove", |e| {
            let [x, y] = convert_coord(e.elem, e.event);
            DomToWorker::CanvasMouseMove { x, y }.some()
        }),
        worker.register_event(canvas, "mousedown", |e| {
            let [x, y] = convert_coord(e.elem, e.event);
            DomToWorker::CanvasMouseDown { x, y }.some()
        }),
        worker.register_event(canvas, "wheel", |e| {
            e.event.prevent_default();
            e.event.stop_propagation();
            None
        }),
        worker.register_event(canvas, "mouseup", |_| DomToWorker::CanvasMouseUp.some()),
        worker.register_event(canvas, "mouseleave", |_| {
            DomToWorker::CanvasMouseLeave.some()
        }),
        worker.register_event(canvas, "touchstart", |e| {
            let touches = convert_coord_touch(e.elem, e.event);
            DomToWorker::TouchDown { touches }.some()
        }),
        worker.register_event(canvas, "touchmove", |e| {
            let touches = convert_coord_touch(e.elem, e.event);
            DomToWorker::TouchMove { touches }.some()
        }),
        worker.register_event(canvas, "touchend", |e| {
            let touches = convert_coord_touch(e.elem, e.event);
            DomToWorker::TouchEnd { touches }.some()
        }),
        {
            let undo = utils::get_by_id_elem("undo");
            worker.register_event(&undo, "click", move |_| {
                log!("clicked the button!!!!!");
                DomToWorker::Undo.some()
            })
        },
        {
            let w = gloo::utils::window();

            worker.register_event(&w, "resize", |_| resize().some())
        },
    ]
}

pub async fn start_game(game_type: GameType, host: &str) {
    let canvas = utils::get_by_id_canvas("mycanvas");

    canvas.set_width(gloo::utils::body().client_width() as u32);
    canvas.set_height(gloo::utils::body().client_height() as u32);

    let offscreen = canvas.transfer_control_to_offscreen().unwrap_throw();

    let (mut worker, mut response) =
        shogo::EngineMain::new("./gridlock_worker.js", offscreen).await;

    let _handlers = engine_handlers(&mut worker, &canvas);

    worker.post_message(DomToWorker::Start(game_type));
    let hay: WorkerToDom = response.next().await.unwrap_throw();
    matches!(hay, WorkerToDom::Ack);

    log!("dom:worker received the game");

    //TODO make this happen on start??
    worker.post_message(resize());

    //TODO put somewhere else
    //let host = "http://localhost:8000";

    loop {
        let hay: WorkerToDom = response.next().await.unwrap_throw();

        match hay {
            WorkerToDom::Ack => {
                unreachable!();
            }
            WorkerToDom::ReplayFinish => {
                let body = gloo::utils::document().body().unwrap();

                use std::fmt::Write;

                let mut k = String::new();
                write!(
                    &mut k,
                    r###"
                <div id="gameover_popup" hidden="true">
                
                <text class="foo">replay finished!</text>
                <a class="foo" href="{host}/index.html ">main menu</a>
    
              </div>
              "###
                )
                .unwrap();

                body.insert_adjacent_html("beforeend", &k).unwrap();
            }
            WorkerToDom::CantParseReplay => {
                let body = gloo::utils::document().body().unwrap();

                use std::fmt::Write;

                let mut k = String::new();
                write!(
                    &mut k,
                    r###"
                <div id="gameover_popup" hidden="true">
                
                <text class="foo">Failed to parse replay code</text>
                <a class="foo" href="{host}/index.html ">main menu</a>
    
              </div>
              "###
                )
                .unwrap();

                body.insert_adjacent_html("beforeend", &k).unwrap();
            }
            WorkerToDom::GameFinish {
                replay_string,
                result,
            } => {
                let body = gloo::utils::document().body().unwrap();

                let team_str = match result {
                    GameOver::WhiteWon => "White Won!",
                    GameOver::BlackWon => "Black Won!",
                    GameOver::Tie => "Its a tie!",
                };
                use std::fmt::Write;

                let mut k = String::new();
                write!(&mut k,r###"
                <div id="gameover_popup" hidden="true">
                
                <text class="foo">GAME OVER:{team_str}</text>
                <text class="foo">Game Replay Code</text>
                <textarea class="foo" id="w3review" readonly="true" rows="6" cols="50">{replay_string}</textarea>
                <a class="foo" href="{host}/game1.html?v=replay&data={replay_string}">replay</a>
                <a class="foo" href="{host}/index.html ">main menu</a>
                
              </div>
              "###).unwrap();

                body.insert_adjacent_html("beforeend", &k).unwrap();

                //gameover.set_hidden(false);
                log!("dom:Game finished");
            }
            WorkerToDom::ShowUndo => {
                let undo = utils::get_by_id_elem("undo");

                undo.set_hidden(false);

                let k = r###"<button id="undo" class="foo">Undo</button>"###;
                //let body = gloo::utils::document().body().expect("get body fail");

                //body.insert_adjacent_html("beforeend",&k).expect("inserting undo fail");
                //undo.set_attribute("hidden","false").unwrap();

                worker.post_message(DomToWorker::Ack);
                //popup.set_text_content(Some(&text));
            }
            WorkerToDom::HideUndo => {
                let undo = utils::get_by_id_elem("undo");
                //let body = gloo::utils::document().body().expect("get body fail");
                //body.remove_child(&undo).expect("Couldnt remove undo");
                //popup.set_text_content(Some(""));
                undo.set_hidden(true);
                //undo.set_attribute("hidden","true").unwrap();

                worker.post_message(DomToWorker::Ack);
            }
        }
        //log!(format!("main thread received={:?}", hay));
    }
    //log!("main thread is closing");
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameType {
    SinglePlayer,
    PassPlay,
    AIBattle,
    Replay(String),
}

#[wasm_bindgen]
pub async fn main_entry() {
    let search = gloo::utils::window().location().search().unwrap();

    let prot = gloo::utils::window().location().protocol().unwrap();
    let host = gloo::utils::window().location().host().unwrap();

    let host = format!("{}//{}", prot, host);

    console_dbg!("host", host);

    let k = search.as_str();

    let (a, k) = k.split_at(1);
    console_dbg!(a, k);
    assert_eq!(a, "?");

    let res = querystring::querify(k);
    console_dbg!("querystring:", res);
    // let mut k = k.chars();

    // assert_eq!(k.next().unwrap(), '?');
    // assert_eq!(k.next().unwrap(), 'v');
    // assert_eq!(k.next().unwrap(), '=');
    // let command = k.as_str();
    // console_dbg!(command);

    //search.sp
    //TODO check if its PLAY AI VS LOCAL PLAY
    console_dbg!(search);

    let command = match res[0] {
        ("v", "singleplayer") => {
            log!("singleplayer!!!");
            GameType::SinglePlayer
        }
        ("v", "passplay") => {
            log!("passplay!!!");
            GameType::PassPlay
        }
        ("v", "aibattle") => {
            log!("aibattle!!!");
            GameType::AIBattle
        }
        ("v", "replay") => {
            assert_eq!(res[1].0, "data");
            GameType::Replay(res[1].1.into())
        }
        _ => {
            unreachable!("unrecognized command");
        }
    };

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

    start_game(command, &host).await;
}
fn resize() -> DomToWorker {
    let canvas = utils::get_by_id_canvas("mycanvas");
    //canvas.set_width(gloo::utils::body().client_width() as u32);
    //canvas.set_height(gloo::utils::body().client_height() as u32);

    // let width = gloo::utils::document().body().unwrap_throw().client_width();
    // let height = gloo::utils::document()
    //     .body()
    //     .unwrap_throw()
    //     .client_height();

    let width = canvas.client_width();
    let height = canvas.client_height();

    let realpixels = gloo::utils::window().device_pixel_ratio();
    log!(format!("pixel ratio:{:?}", realpixels));
    // .body.clientWidth;
    // var height=document.body.clientHeight;

    // var realToCSSPixels = window.devicePixelRatio;
    // var gl_width  = Math.floor(width  * realToCSSPixels);
    // var gl_height = Math.floor(height * realToCSSPixels);

    let gl_width = (width as f64 * realpixels).floor();
    let gl_height = (height as f64 * realpixels).floor();

    DomToWorker::Resize {
        canvasx: gloo::utils::body().client_width() as u32,
        canvasy: gloo::utils::body().client_height() as u32,
        x: gl_width as f32,
        y: gl_height as f32,
    }
}
