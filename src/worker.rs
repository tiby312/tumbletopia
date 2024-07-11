use gloo::utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen::{JsCast, JsValue};

pub struct WorkerInterface<MW, WM> {
    worker: std::rc::Rc<std::cell::RefCell<web_sys::Worker>>,
    _handle: gloo::events::EventListener,
    _p: PhantomData<(MW, WM)>,
}

impl<MW: 'static + Serialize, WM: for<'a> Deserialize<'a> + 'static> WorkerInterface<MW, WM> {
    ///
    /// Create the engine. Blocks until the worker thread reports that
    /// it is ready to receive the offscreen canvas.
    ///
    pub async fn new(
        web_worker_url: &str,
    ) -> (Self, futures::channel::mpsc::UnboundedReceiver<WM>) {
        let (fs, fr) = futures::channel::oneshot::channel();
        let mut fs = Some(fs);

        let (ks, kr) = futures::channel::mpsc::unbounded();

        let mut options = web_sys::WorkerOptions::new();
        options.type_(web_sys::WorkerType::Module);

        let worker = Rc::new(RefCell::new(
            web_sys::Worker::new_with_options(web_worker_url, &options).unwrap_throw(),
        ));

        let _handle = gloo::events::EventListener::new(&worker.borrow(), "message", move |event| {
            //log!("waaa");
            let event = event.dyn_ref::<web_sys::MessageEvent>().unwrap_throw();
            let data = event.data();

            let data: js_sys::Array = data.dyn_into().unwrap_throw();
            let m = data.get(0);
            let k = data.get(1);

            if !m.is_null() {
                if let Some(s) = m.as_string() {
                    if s == "ready" {
                        if let Some(f) = fs.take() {
                            f.send(()).unwrap_throw();
                        }
                    }
                }
            } else {
                let payload: js_sys::JsString = k.into();
                let payload: String = payload.into();

                let e = match serde_json::from_str(&payload) {
                    Ok(e) => e,
                    Err(f) => {
                        crate::console_dbg!("ERRRR", f);
                        crate::console_dbg!(payload);
                        panic!();
                    }
                };

                ks.unbounded_send(e).unwrap_throw();
            }
        });

        let _ = fr.await.unwrap_throw();

        // let arr = js_sys::Array::new_with_length(1);
        // arr.set(0, canvas.clone().into());

        // let data = js_sys::Array::new();
        // data.set(0, canvas.into());
        // data.set(1, JsValue::null());

        // worker
        //     .borrow()
        //     .post_message_with_transfer(&data, &arr)
        //     .unwrap_throw();

        (
            WorkerInterface {
                worker,
                _handle,
                _p: PhantomData,
            },
            kr,
        )
    }

    pub fn post_message(&mut self, val: MW) {
        let stest = serde_json::to_string(&val).unwrap();
        //crate::console_dbg!("tttest",stest);

        let a: js_sys::JsString = stest.into();

        //let a = JsValue::from_serde(&val).expect("Couldn't put it into json!!!!");

        if a.is_null() {
            crate::console_dbg!("FAILED TO PACK PAYLOAD INTO JSON");
        }

        let data = js_sys::Array::new();
        data.set(0, JsValue::null());
        data.set(1, a.into());

        self.worker
            .borrow()
            .post_message(&data)
            .expect("Could not post message!");
    }

    // ///
    // /// Register a new event that will be packaged and sent to the worker thread.
    // ///
    // pub fn register_event(
    //     &mut self,
    //     elem: &web_sys::EventTarget,
    //     event_type: &'static str,
    //     mut func: impl FnMut(EventData) -> Option<MW> + 'static,
    // ) -> gloo::events::EventListener {
    //     let w = self.worker.clone();

    //     let e = elem.clone();

    //     use gloo::events::EventListenerOptions;
    //     use gloo::events::EventListenerPhase;
    //     let options = EventListenerOptions {
    //         phase: EventListenerPhase::Bubble,
    //         passive: false,
    //     };

    //     gloo::events::EventListener::new_with_options(elem, event_type, options, move |event| {
    //         let e = EventData {
    //             elem: &e,
    //             event,
    //             event_type,
    //         };

    //         if let Some(val) = func(e) {
    //             let a = JsValue::from_serde(&val).unwrap_throw();

    //             let data = js_sys::Array::new();
    //             data.set(0, JsValue::null());
    //             data.set(1, a);

    //             w.borrow().post_message(&data).unwrap_throw();
    //         }
    //     })
    // }
}

pub struct Worker<MW, WM> {
    _handle: gloo::events::EventListener,
    _p: PhantomData<(MW, WM)>,
}

impl<MW: 'static + for<'a> Deserialize<'a>, WM: Serialize> Worker<MW, WM> {
    pub fn new() -> (
        Worker<MW, WM>,
        futures::channel::mpsc::UnboundedReceiver<MW>,
    ) {
        let scope = shogo::utils::get_worker_global_context();

        // let (fs, fr) = futures::channel::oneshot::channel();
        // let mut fs = Some(fs);

        let (bags, bagf) = futures::channel::mpsc::unbounded();

        let _handle = gloo::events::EventListener::new(&scope, "message", move |event| {
            crate::console_dbg!("working got something");
            let event = event.dyn_ref::<web_sys::MessageEvent>().unwrap_throw();
            let data = event.data();

            let data: js_sys::Array = data.dyn_into().unwrap_throw();

            //let offscreen = data.get(0);
            let payload = data.get(1);

            // if !offscreen.is_null() {
            //     let offscreen: web_sys::OffscreenCanvas = offscreen.dyn_into().unwrap_throw();
            //     if let Some(fs) = fs.take() {
            //         fs.send(offscreen).unwrap_throw();
            //     }
            // }

            if !payload.is_null() {
                let payload: js_sys::JsString = payload.into();
                let payload: String = payload.into();

                // crate::console_dbg!("a");
                // //crate::console_dbg!(payload);

                // let k=js_sys::JSON::stringify(&payload)
                // .map(String::from)
                // .unwrap_throw();
                // crate::console_dbg!(k);

                let e = match serde_json::from_str(&payload) {
                    Ok(e) => e,
                    Err(f) => {
                        crate::console_dbg!("ERRRR", f);
                        crate::console_dbg!(payload);
                        panic!();
                    }
                };

                // let e=match payload.into_serde(){
                //     Ok(e) => e,
                //     Err(f) => {
                //         crate::console_dbg!("ERRRR",f);
                //         crate::console_dbg!(payload);
                //         panic!();
                //     },
                // };
                //let e = payload.into_serde().unwrap_throw();
                bags.unbounded_send(e).unwrap_throw();
            } else {
                crate::console_dbg!("THE PAYLOAD IS NULL");
            }
        });

        let data = js_sys::Array::new();
        data.set(0, JsValue::from_str("ready"));
        data.set(1, JsValue::null());

        scope.post_message(&data).unwrap_throw();

        (
            Worker {
                _handle,
                _p: PhantomData,
            },
            bagf,
        )
    }

    pub fn post_message(&mut self, a: WM) {
        let stest = serde_json::to_string(&a).unwrap();
        //crate::console_dbg!("tttest",stest);

        let a: js_sys::JsString = stest.into();

        let scope = shogo::utils::get_worker_global_context();

        let data = js_sys::Array::new();
        data.set(0, JsValue::null());
        data.set(1, a.into());

        scope.post_message(&data).unwrap_throw();
    }
}
