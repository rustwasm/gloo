use crate::app::App;
use discard::Discard;
use dominator::DomHandle;
use futures_signals::signal::SignalExt;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

#[macro_use]
mod macros;
mod app;
mod todo;
mod util;

#[wasm_bindgen(start)]
pub fn main_js() {
    console_error_panic_hook::set_once();

    // Request our data be persisted
    wasm_bindgen_futures::spawn_local(async move {
        gloo::storage::persist().await;
    });

    let app = App::new();
    App::init(&app);
    let dom: Arc<Mutex<Option<DomHandle>>> = Arc::new(Mutex::new(None));
    wasm_bindgen_futures::spawn_local(app.signal_cloned().for_each(move |state| {
        //gloo::console::console_dbg!(state);
        let mut dom = dom.lock().unwrap();
        if let Some(dom) = dom.take() {
            dom.discard();
        }
        *dom = Some(dominator::append_dom(
            &dominator::get_id("todoapp"),
            state.render(),
        ));
        async move {}
    }));
}
