use futures::{SinkExt, StreamExt, channel::mpsc::unbounded};
use gloo::console::console_dbg;
use gui::dom::DomToWorker;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, UnwrapThrowExt};

use futures::channel::mpsc::UnboundedSender;

#[derive(Clone, PartialEq, Eq)]
pub struct ID(usize);

use shogo::main::MainSender;
use std::rc::Rc;
use web_sys::{HtmlCanvasElement, HtmlInputElement};

pub trait MyListen {
    fn new_event(&self, args: Args) -> Vec<gloo::events::EventListener>;
    fn on_destroy(&self) {}
}

pub struct Args {
    id: ID,
    destroy_self: UnboundedSender<ID>,
    worker: Rc<MainSender<DomToWorker>>,
}

#[derive(Clone)]
struct Foo {
    w: Rc<MainSender<DomToWorker>>,
    canvas: HtmlCanvasElement,
}

pub struct MainMenu {}
impl MyListen for MainMenu {
    fn new_event(&self, args: Args) -> Vec<gloo::events::EventListener> {
        let canvas = shogo::utils::get_by_id_canvas("mycanvas");

        let w = args.worker.clone();
        let canvas2 = canvas.clone();
        let foo = Foo { w, canvas: canvas2 };

        let a1 = {
            let foo = foo.clone();
            gloo::events::EventListener::new(
                &shogo::utils::get_by_id_elem("stack_gui_val"),
                "change",
                move |e| {
                    let val: HtmlInputElement = shogo::utils::get_by_id_elem("stack_gui_val")
                        .dyn_into()
                        .unwrap();

                    foo.w.clone().post_message(DomToWorker::Button(format!(
                        "{}{}",
                        "stack",
                        val.value()
                    )));
                },
            )
        };

        let a2 = {
            let foo = foo.clone();
            gloo::events::EventListener::new(
                &shogo::utils::get_by_id_elem("player1"),
                "change",
                move |e| {
                    let val: HtmlInputElement =
                        shogo::utils::get_by_id_elem("player1").dyn_into().unwrap();
                    console_dbg!("player1:", val.checked());
                    foo.w
                        .clone()
                        .post_message(DomToWorker::Button("player1".into()));
                },
            )
        };

        let a3 = {
            let foo = foo.clone();
            gloo::events::EventListener::new(
                &shogo::utils::get_by_id_elem("player2"),
                "change",
                move |e| {
                    let val: HtmlInputElement =
                        shogo::utils::get_by_id_elem("player2").dyn_into().unwrap();
                    console_dbg!("player2:", val.checked());
                    foo.w
                        .clone()
                        .post_message(DomToWorker::Button("player2".into()));
                },
            )
        };

        let a4 = {
            let foo = foo.clone();
            gloo::events::EventListener::new(
                &shogo::utils::get_by_id_elem("player3"),
                "change",
                move |e| {
                    let val: HtmlInputElement =
                        shogo::utils::get_by_id_elem("player3").dyn_into().unwrap();
                    console_dbg!("player3:", val.checked());
                    foo.w
                        .clone()
                        .post_message(DomToWorker::Button("player3".into()));
                },
            )
        };

        let a5 = {
            let foo = foo.clone();
            gloo::events::EventListener::new(
                &shogo::utils::get_by_id_elem("empty"),
                "change",
                move |e| {
                    let val: HtmlInputElement =
                        shogo::utils::get_by_id_elem("player3").dyn_into().unwrap();
                    console_dbg!("empty:", val.checked());
                    foo.w
                        .clone()
                        .post_message(DomToWorker::Button("empty".into()));
                },
            )
        };

        vec![a1, a2, a3, a4, a5]
    }
}

pub struct EngineListeners {}

impl MyListen for EngineListeners {
    fn new_event(&self, args: Args) -> Vec<gloo::events::EventListener> {
        let canvas = shogo::utils::get_by_id_canvas("mycanvas");

        let w = args.worker.clone();
        let canvas2 = canvas.clone();

        let foo = Foo { w, canvas: canvas2 };

        let a1 = {
            let foo = foo.clone();
            gloo::events::EventListener::new(&canvas, "mousedown", move |e| {
                let [x, y] = gui::dom::convert_coord(&foo.canvas, e);
                foo.w
                    .clone()
                    .post_message(DomToWorker::CanvasMouseDown { x, y });
            })
        };

        let a2 = {
            let foo = foo.clone();
            gloo::events::EventListener::new(&canvas, "mousemove", move |e| {
                let [x, y] = gui::dom::convert_coord(&foo.canvas.clone(), e);
                foo.w
                    .clone()
                    .post_message(DomToWorker::CanvasMouseMove { x, y });
            })
        };

        let a3 = {
            let option = gloo::events::EventListenerOptions::enable_prevent_default();
            gloo::events::EventListener::new_with_options(&canvas, "wheel", option, move |e| {
                e.prevent_default();
                e.stop_propagation();
            })
        };

        let a4 = {
            let foo = foo.clone();
            gloo::events::EventListener::new(&canvas, "mouseup", move |e| {
                foo.w.clone().post_message(DomToWorker::CanvasMouseUp);
            })
        };

        let a5 = {
            let foo = foo.clone();
            gloo::events::EventListener::new(&canvas, "mouseleave", move |e| {
                foo.w.clone().post_message(DomToWorker::CanvasMouseLeave);
            })
        };

        let a6 = {
            let foo = foo.clone();
            let option = gloo::events::EventListenerOptions::enable_prevent_default();

            gloo::events::EventListener::new_with_options(&canvas, "touchstart", option, move |e| {
                e.prevent_default();
                e.stop_propagation();

                let touches = gui::dom::convert_coord_touch(&foo.canvas, e);

                foo.w
                    .clone()
                    .post_message(DomToWorker::TouchDown { touches });
            })
        };

        let a7 = {
            let foo = foo.clone();
            let option = gloo::events::EventListenerOptions::enable_prevent_default();

            gloo::events::EventListener::new_with_options(&canvas, "touchmove", option, move |e| {
                e.prevent_default();
                e.stop_propagation();

                let touches = gui::dom::convert_coord_touch(&foo.canvas, e);

                foo.w
                    .clone()
                    .post_message(DomToWorker::TouchMove { touches });
            })
        };

        let a8 = {
            let foo = foo.clone();
            let option = gloo::events::EventListenerOptions::enable_prevent_default();

            gloo::events::EventListener::new_with_options(&canvas, "touchend", option, move |e| {
                e.prevent_default();
                e.stop_propagation();

                let touches = gui::dom::convert_coord_touch(&foo.canvas, e);

                foo.w
                    .clone()
                    .post_message(DomToWorker::TouchEnd { touches });
            })
        };

        vec![a1, a2, a3, a4, a5, a6, a7, a8]
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ShowPopup {
    name: String,
    typ: String,
}
impl MyListen for ShowPopup {
    fn new_event(&self, args: Args) -> Vec<gloo::events::EventListener> {
        //TODO make html show up here.

        let foo = shogo::utils::get_by_id_elem(&self.name);
        vec![gloo::events::EventListener::new(
            &foo,
            self.typ.clone(),
            move |_| {
                args.worker.post_message(DomToWorker::Ack);
                args.destroy_self.unbounded_send(args.id.clone()).unwrap();
            },
        )]
    }

    fn on_destroy(&self) {
        //TODO make html destroy itself here
        todo!()
    }
}

struct Counter(usize);

#[derive(Serialize, Deserialize)]
pub enum WorkerToDom {
    ShowPopup(ShowPopup),
}
impl MyListen for WorkerToDom {
    fn new_event(&self, args: Args) -> Vec<gloo::events::EventListener> {
        match self {
            WorkerToDom::ShowPopup(o) => o.new_event(args),
        }
    }

    fn on_destroy(&self) {
        match self {
            WorkerToDom::ShowPopup(o) => o.on_destroy(),
        }
    }
}

pub async fn lap() {
    struct Foo {
        id: ID,
        _event: Vec<gloo::events::EventListener>,
        hay: WorkerToDom,
    }

    let canvas = shogo::utils::get_by_id_canvas("mycanvas");

    let offscreen = canvas.transfer_control_to_offscreen().unwrap_throw();

    let mut my_events = vec![];
    let mut id_counter = Counter(0);

    let (destroy_listener_queue_send, mut destroy_listener_queue_recv) =
        futures::channel::mpsc::unbounded();

    let (sender, mut recv) =
        shogo::main::create_main::<DomToWorker, WorkerToDom, _>("./gridlock_worker.js", offscreen)
            .await;

    let dom_to_worker = Rc::new(sender);

    futures::select! {
            a = destroy_listener_queue_recv.next()=>{
                let a:ID=a.unwrap_throw();

                let mut it=my_events.extract_if(..,|x:&mut Foo|{
                    x.id==a
                });

                let Foo{_event,hay,..} = it.next().unwrap();
                assert!(it.next().is_none());

                drop(_event);

                hay.on_destroy();
            }
            hay = recv.recv().next() => {
                let hay:WorkerToDom=hay.unwrap_throw();

                let id=ID(id_counter.0);
                id_counter.0+=1;
                let _event=hay.new_event(Args{id:id.clone(),destroy_self:destroy_listener_queue_send,worker:dom_to_worker});


                my_events.push(Foo{
                    hay,
                    _event,
                    id
                })
            }
    }
}
