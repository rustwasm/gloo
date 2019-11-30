use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo::timers::future::IntervalStream;
use futures_util::{stream::StreamExt, future::ready};
use chrono::Timelike;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    let document = web_sys::window().unwrap_throw().document().unwrap_throw();
    let el = document.get_element_by_id("clock").unwrap_throw();
    render_date(&el);
    spawn_local(async move {
        IntervalStream::new(1_000).for_each(|_| {
            render_date(&el);
            ready(())
        }).await;
    });
}

fn render_date(el: &web_sys::Element) {
    let date = chrono::Local::now();
    let format_str = if date.second() % 2 == 0 {
        "%Y-%m-%d %H %M"
    } else {
        "%Y-%m-%d %H:%M"
    };
    let date_str = date.format(format_str).to_string();
    el.set_text_content(Some(&date_str));
}
