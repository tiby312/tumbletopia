use std::{cell::RefCell, marker::PhantomData, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use serde::{Deserialize, Serialize};
use wasm_bindgen::UnwrapThrowExt;
use gloo::utils::format::JsValueSerdeExt;


pub struct EngineMain<MW, WM> {
    worker: std::rc::Rc<std::cell::RefCell<web_sys::Worker>>,
    _handle: gloo::events::EventListener,
    _p: PhantomData<(MW, WM)>,
}

impl<MW: 'static + Serialize, WM: for<'a> Deserialize<'a> + 'static> EngineMain<MW, WM> {
    ///
    /// Create the engine. Blocks until the worker thread reports that
    /// it is ready to receive the offscreen canvas.
    ///
    pub async fn new(
        web_worker_url: &str,
    ) -> (Self, futures::channel::mpsc::UnboundedReceiver<WM>) {
        let mut options = web_sys::WorkerOptions::new();
        options.type_(web_sys::WorkerType::Module);
        let worker = Rc::new(RefCell::new(
            web_sys::Worker::new_with_options(web_worker_url, &options).unwrap_throw(),
        ));

        // let (fs, fr) = futures::channel::oneshot::channel();
        // let mut fs = Some(fs);

        let (ks, kr) = futures::channel::mpsc::unbounded();
        let _handle =
            gloo::events::EventListener::new(&worker.borrow(), "message", move |event| {
                //log!("waaa");
                let event = event.dyn_ref::<web_sys::MessageEvent>().unwrap_throw();
                let data = event.data();

                let data: js_sys::Array = data.dyn_into().unwrap_throw();
                let m = data.get(0);
                let k = data.get(1);

                if !m.is_null() {
                    // if let Some(s) = m.as_string() {
                    //     if s == "ready" {
                    //         if let Some(f) = fs.take() {
                    //             f.send(()).unwrap_throw();
                    //         }
                    //     }
                    // }
                } else {
                    let a = k.into_serde().unwrap_throw();
                    ks.unbounded_send(a).unwrap_throw();
                }
            });

        //let _ = fr.await.unwrap_throw();

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
            EngineMain {
                worker,
                _handle,
                _p: PhantomData,
            },
            kr,
        )
    }


    pub fn post_message(&mut self, val: MW) {
        let a = JsValue::from_serde(&val).unwrap_throw();

        let data = js_sys::Array::new();
        data.set(0, JsValue::null());
        data.set(1, a);

        self.worker.borrow().post_message(&data).unwrap_throw();
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


pub struct EngineWorker<MW, WM> {
    _handle: gloo::events::EventListener,
    _p: PhantomData<(MW, WM)>,
}

impl<MW: 'static + for<'a> Deserialize<'a>, WM: Serialize> EngineWorker<MW, WM> {
    
    pub fn new() -> (
        EngineWorker<MW, WM>,
        futures::channel::mpsc::UnboundedReceiver<MW>,
    ) {
        let scope = shogo::utils::get_worker_global_context();

        // let (fs, fr) = futures::channel::oneshot::channel();
        // let mut fs = Some(fs);

        let (bags, bagf) = futures::channel::mpsc::unbounded();

        let _handle = gloo::events::EventListener::new(&scope, "message", move |event| {
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
                let e = payload.into_serde().unwrap_throw();
                bags.unbounded_send(e).unwrap_throw();
            }
        });

        let data = js_sys::Array::new();
        data.set(0, JsValue::from_str("ready"));
        data.set(1, JsValue::null());

        scope.post_message(&data).unwrap_throw();

        (
            EngineWorker {
                _handle,
                _p: PhantomData,
            },
            bagf,
        )
    }

    pub fn post_message(&mut self, a: WM) {
        let scope = shogo::utils::get_worker_global_context();

        let data = js_sys::Array::new();
        data.set(0, JsValue::null());
        data.set(1, JsValue::from_serde(&a).unwrap_throw());

        scope.post_message(&data).unwrap_throw();
    }
}
