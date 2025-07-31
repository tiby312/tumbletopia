use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;

use super::*;

use crate as gui;

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
    //GameChange(String),
    TouchMove {
        touches: gui::scroll::Touches,
    },
    TouchDown {
        touches: gui::scroll::Touches,
    },
    TouchEnd {
        touches: gui::scroll::Touches,
    },
    Start(GameType),
    Button(String), //TODO use array backed
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

//convert DOM coordinate to canvas relative coordinate
fn convert_coord(canvas: &web_sys::EventTarget, event: &web_sys::Event) -> [f32; 2] {
    gui::shader_sys::convert_coord(
        canvas.dyn_ref().unwrap_throw(),
        event.dyn_ref().unwrap_throw(),
    )
}

fn convert_coord_touch(
    canvas: &web_sys::EventTarget,
    event: &web_sys::Event,
) -> gui::scroll::Touches {
    convert_coord_touch_inner(canvas, event.dyn_ref().unwrap_throw())
}

///
/// Convert a mouse event to a coordinate for simple2d.
///
pub fn convert_coord_touch_inner(
    canvas: &web_sys::EventTarget,
    e: &web_sys::TouchEvent,
) -> gui::scroll::Touches {
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

    let mut k = gui::scroll::Touches {
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
    ctx.set_fill_style_str("black");
    ctx.clear_rect(0., 0., width as f64, height as f64);
    ctx.fill_text(text, width as f64 / 2., height as f64 / 2.)
        .unwrap();
    canvas
}

#[must_use]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameOverGui {
    WhiteWon,
    BlackWon,
    Tie,
}

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct Text {
    pub text: String,
    pub pos: [f32; 2],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WorkerToDom {
    ShowUndo,
    HideUndo,
    TextUpdate(Vec<Text>, ScoreData, String),
    GameFinish {
        replay_string: String,
        result: GameOverGui,
    },
    ExportMap(String),
    CantParseReplay,
    ReplayFinish,
    ShowPopup(String),
    HidePopup(String),
    Ack,
}

trait Any {}
impl<T: ?Sized> Any for T {}

fn engine_handlers<'a>(
    worker: &'a shogo::main::MainSender<DomToWorker>,
    canvas: &'a web_sys::HtmlCanvasElement,
) -> impl Any + 'a {
    use gloop::EventListen;

    let reg_button = |worker: &'a shogo::main::MainSender<DomToWorker>, s: &'static str| {
        let undo = shogo::utils::get_by_id_elem(s);
        gloop::EventListen::from_closure(&undo, "click", move |_| {
            worker.post_message(DomToWorker::Button(s.to_string()));
        })
    };

    let option = gloo::events::EventListenerOptions::enable_prevent_default();
    (
        EventListen::from_closure(
            &shogo::utils::get_by_id_elem("stack_gui_val"),
            "change",
            |_| {
                let val: HtmlInputElement = shogo::utils::get_by_id_elem("stack_gui_val")
                    .dyn_into()
                    .unwrap();
                console_dbg!("stack val is now:", val.value());
                worker.post_message(DomToWorker::Button(format!("{}{}", "stack", val.value())));
            },
        ),
        EventListen::from_closure(&shogo::utils::get_by_id_elem("player1"), "change", |_| {
            let val: HtmlInputElement = shogo::utils::get_by_id_elem("player1").dyn_into().unwrap();
            console_dbg!("player1:", val.checked());
            worker.post_message(DomToWorker::Button("player1".into()));
        }),
        EventListen::from_closure(&shogo::utils::get_by_id_elem("player2"), "change", |_| {
            let val: HtmlInputElement = shogo::utils::get_by_id_elem("player2").dyn_into().unwrap();
            console_dbg!("player2:", val.checked());
            worker.post_message(DomToWorker::Button("player2".into()));
        }),
        EventListen::from_closure(&shogo::utils::get_by_id_elem("player3"), "change", |_| {
            let val: HtmlInputElement = shogo::utils::get_by_id_elem("player3").dyn_into().unwrap();
            console_dbg!("player3:", val.checked());
            worker.post_message(DomToWorker::Button("player3".into()));
        }),
        EventListen::from_closure(&shogo::utils::get_by_id_elem("empty"), "change", |_| {
            let val: HtmlInputElement = shogo::utils::get_by_id_elem("empty").dyn_into().unwrap();
            console_dbg!("empty:", val.checked());
            worker.post_message(DomToWorker::Button("empty".into()));
        }),
        EventListen::from_closure(canvas, "mousemove", |e| {
            let [x, y] = convert_coord(canvas, e);
            worker.post_message(DomToWorker::CanvasMouseMove { x, y });
        }),
        EventListen::from_closure(canvas, "mousedown", |e| {
            let [x, y] = convert_coord(canvas, e);
            worker.post_message(DomToWorker::CanvasMouseDown { x, y });
        }),
        EventListen::from_closure_with_options(canvas, "wheel", option, |e| {
            e.prevent_default();
            e.stop_propagation();
        }),
        EventListen::from_closure(canvas, "mouseup", |_| {
            worker.post_message(DomToWorker::CanvasMouseUp);
        }),
        EventListen::from_closure(canvas, "mouseleave", |_| {
            worker.post_message(DomToWorker::CanvasMouseLeave);
        }),
        EventListen::from_closure_with_options(canvas, "touchstart", option, |e| {
            e.prevent_default();
            e.stop_propagation();

            let touches = convert_coord_touch(canvas, e);
            worker.post_message(DomToWorker::TouchDown { touches })
        }),
        EventListen::from_closure_with_options(canvas, "touchmove", option, |e| {
            e.prevent_default();
            e.stop_propagation();

            let touches = convert_coord_touch(canvas, e);
            worker.post_message(DomToWorker::TouchMove { touches })
        }),
        EventListen::from_closure_with_options(canvas, "touchend", option, |e| {
            e.prevent_default();
            e.stop_propagation();

            let touches = convert_coord_touch(canvas, e);
            worker.post_message(DomToWorker::TouchEnd { touches })
        }),
        //TODO register buttons based on game type
        reg_button(worker, "undo"),
        reg_button(worker, "pass"),
        reg_button(worker, "lighthouse"),
        reg_button(worker, "b_next"),
        reg_button(worker, "b_prev"),
        reg_button(worker, "popup_ack"),
        // reg_button(worker, "b_ice"),
        // reg_button(worker, "b_land"),
        // reg_button(worker, "b_water"),
        // reg_button(worker, "b_forest"),
        // reg_button(worker, "b_start1"),
        // reg_button(worker, "b_start2"),
        // reg_button(worker, "b_export"),
    )
}

pub async fn start_game(game_type: GameType, host: &str) {
    let canvas = shogo::utils::get_by_id_canvas("mycanvas");

    let offscreen = canvas.transfer_control_to_offscreen().unwrap_throw();

    //let (worker, mut response) = shogo::EngineMain::new("./gridlock_worker.js", offscreen).await;

    let (sender, mut recv) = shogo::main::create_main("./gridlock_worker.js", offscreen).await;

    let (mut repaint_text_send, mut repaint_text_recv) = futures::channel::mpsc::channel(20);
    let mut repaint_text_send2 = repaint_text_send.clone();

    use futures::SinkExt;
    let _h = {
        let w = gloo::utils::window();
        gloop::EventListen::from_closure(&w, "resize", |_| {
            sender.post_message(resize2());
            repaint_text_send2.send(()).now_or_never().unwrap().unwrap()
        })
    };

    let _handlers = engine_handlers(&sender, &canvas);

    sender.post_message(DomToWorker::Start(game_type));
    let hay: WorkerToDom = recv.recv().next().await.unwrap_throw();
    matches!(hay, WorkerToDom::Ack);

    log!("dom:worker received the game");

    sender.post_message(resize2());

    //repaint_text_send.send(()).await.unwrap();

    let mut text = vec![];
    // text.push(Text {
    //     text: "Hello".to_string(),
    //     pos: [40.0, 40.0],
    // });

    let mut score_data = None;

    loop {
        futures::select! {
            _ = repaint_text_recv.next() =>{

                redraw_text(&text,score_data.as_ref().unwrap());
            },
            hay = recv.recv().next() => {
                let hay = hay.unwrap_throw();

                match hay {
                    WorkerToDom::ShowPopup(s)=>{
                        let foo = shogo::utils::get_by_id_elem("nextplayer_popup");
                        foo.set_attribute("style", "display:grid").unwrap_throw();
                    }
                    WorkerToDom::HidePopup(s)=>{
                        let foo = shogo::utils::get_by_id_elem("nextplayer_popup");
                        foo.set_attribute("style", "display:none;").unwrap_throw();
                    }
                    WorkerToDom::TextUpdate(t,p,console_entry)=>{
                        text=t;
                        score_data=Some(p);

                        //let foo:HtmlInputElement = shogo::utils::get_by_id_elem("fen").dyn_into().unwrap_throw();
                        //let game=format!("[{}]",game);
                        //foo.set_value(&game);

                        if !console_entry.is_empty(){
                            let foo:web_sys::HtmlElement = shogo::utils::get_by_id_elem("history");
                            //let game=format!("[{}]",game);
                            foo.set_inner_html(&format!("{}<br>{}",foo.inner_html(),console_entry));

                            foo.set_scroll_top(foo.scroll_height());
                        }
                        //objDiv.scrollTop = objDiv.scrollHeight;

                        //foo.set_value(&history);
                        //foo.set_scroll_left(foo.scroll_width());

                        //ta.scrollLeft = ta.scrollWidth;



                        repaint_text_send.send(()).await.unwrap();
                    }
                    WorkerToDom::Ack => {
                        unreachable!();
                    }
                    WorkerToDom::ExportMap(map) => {
                        let foo = shogo::utils::get_by_id_elem("popup");

                        foo.set_attribute("style", "display:grid").unwrap_throw();

                        let foo = shogo::utils::get_by_id_elem("textarea");
                        foo.set_text_content(Some(&map));
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
                        let team_str = match result {
                            GameOverGui::WhiteWon => "White Won!",
                            GameOverGui::BlackWon => "Black Won!",
                            GameOverGui::Tie => "Its a tie!",
                        };

                        let foo = shogo::utils::get_by_id_elem("gameover_popup");
                        foo.set_attribute("style", "display:grid").unwrap_throw();
                        let foo = shogo::utils::get_by_id_elem("gameover_title");
                        foo.set_text_content(Some(&format!("Game over: {}", team_str)));
                        let foo = shogo::utils::get_by_id_elem("gameover_code");
                        foo.set_text_content(Some(&replay_string));
                    }
                    WorkerToDom::ShowUndo => {
                        let undo = shogo::utils::get_by_id_elem("undo");

                        undo.set_hidden(false);

                        //let k = r###"<button id="undo" class="foo">Undo</button>"###;
                        //let body = gloo::utils::document().body().expect("get body fail");

                        //body.insert_adjacent_html("beforeend",&k).expect("inserting undo fail");
                        //undo.set_attribute("hidden","false").unwrap();

                        sender.post_message(DomToWorker::Ack);
                        //popup.set_text_content(Some(&text));
                    }
                    WorkerToDom::HideUndo => {
                        let undo = shogo::utils::get_by_id_elem("undo");
                        //let body = gloo::utils::document().body().expect("get body fail");
                        //body.remove_child(&undo).expect("Couldnt remove undo");
                        //popup.set_text_content(Some(""));
                        undo.set_hidden(true);
                        //undo.set_attribute("hidden","true").unwrap();

                        sender.post_message(DomToWorker::Ack);
                    }
                }

            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameType {
    SinglePlayer(String),
    PassPlay(String),
    AIBattle(String),
    MapEditor(String),
    Replay(String),
    Game(Slot, Slot, Team, String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]

pub enum Team {
    White,
    Black,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Slot {
    Player,
    Ai,
}

#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct ScoreData {
    pub white: usize,
    pub black: usize,
    pub neutral: usize,
}

fn redraw_text(text: &Vec<Text>, data: &ScoreData) {
    console_dbg!("Redrawing text");
    let canvas = shogo::utils::get_by_id_canvas("mycanvas2");
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    context.clear_rect(0., 0., canvas.width() as f64, canvas.height() as f64);

    context.set_text_align("center");
    context.set_font("30px roboto");
    context.set_fill_style_str("grey");
    context.set_text_baseline("middle");

    for a in text {
        context
            .fill_text(&a.text, a.pos[0] as f64, a.pos[1] as f64)
            .unwrap();
    }

    let total = data.white + data.black + data.neutral;
    let white_percentage = data.white as f64 / total as f64;
    let black_percentage = data.black as f64 / total as f64;
    let neutral_percentage = 1.0 - (white_percentage + black_percentage);

    console_dbg!(white_percentage, black_percentage);
    // let radius = 80.0;
    // let x = 505.0;
    // let y = 100.0;

    // let r1 = std::f64::consts::TAU * white_percentage;
    // let r2 = std::f64::consts::TAU * (white_percentage + black_percentage);

    let w = canvas.width() as f64;

    let h1 = canvas.height() as f64 - 40.0;
    let h2 = canvas.height() as f64;

    context.set_fill_style_str("white");
    context.begin_path();
    context.rect(0.0, h1, white_percentage * w, h2);
    context.fill();
    context.set_fill_style_str("#383838");
    context.begin_path();
    context.rect(
        (white_percentage + neutral_percentage) * w,
        h1,
        black_percentage * w,
        h2,
    );
    context.fill();

    context.set_stroke_style_str("gray");
    context.set_line_width(10.);
    context.begin_path();
    context.move_to(0.5 * w, h1);
    context.line_to(0.5 * w, h2);
    context.stroke();

    // context.begin_path();
    // context
    //     .arc(x, y, radius, 0., r1)
    //     .unwrap();
    // context.line_to(x,y);
    // context.fill();

    // context.set_fill_style_str("blue");
    // context.begin_path();
    // context
    //     .arc(x, y, radius, r1, r2)
    //     .unwrap();

    //     context.line_to(x,y);
    // context.fill();
}

fn resize2() -> DomToWorker {
    let canvas = shogo::utils::get_by_id_canvas("mycanvas");
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

    let canvasx = gloo::utils::body().client_width() as u32;
    let canvasy = gloo::utils::body().client_height() as u32;
    {
        let canvas = shogo::utils::get_by_id_canvas("mycanvas2");
        canvas.set_width((canvasx as f64 * realpixels) as u32);
        canvas.set_height((canvasy as f64 * realpixels) as u32);
    }

    DomToWorker::Resize {
        canvasx,
        canvasy,
        x: gl_width as f32,
        y: gl_height as f32,
    }
}
