use std::ops::Deref;

use example_file_hash::codec::TransferrableCodec;
use example_file_hash::{HashInput, HashWorker};
use gloo_worker::Spawnable;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component]
fn App() -> Html {
    let result = use_state(|| None);
    let calculating = use_state(|| false);

    let worker = {
        let calculating = calculating.clone();
        let result = result.clone();

        use_memo(
            move |_| {
                HashWorker::spawner()
                    .callback(move |o| {
                        calculating.set(false);
                        result.set(Some(o.hash));
                    })
                    .encoding::<TransferrableCodec>()
                    .spawn("/example_file_hash_worker.js")
            },
            (),
        )
    };

    let on_choose_file = {
        let calculating = calculating.clone();
        let result = result.clone();
        use_callback(
            move |e: Event, _i| {
                let el: HtmlInputElement = e.target_unchecked_into();
                if let Some(f) = el.files().and_then(|m| m.item(0)) {
                    calculating.set(true);
                    result.set(None);
                    let input = HashInput { file: Some(f) };
                    TransferrableCodec::pre_encode_input(&input);
                    worker.send(input);
                }
            },
            (),
        )
    };

    html! {
        <div>
            <h1>{"To calculate file hash, please select a file below:"}</h1>
            <p><input type="file" disabled={*calculating} onchange={on_choose_file} /></p>
            if let Some(m) = result.deref().clone() {
                <p><h4>{"SHA256: "}{&m}</h4></p>
            }
            if *calculating {
                <p><h4>{"Calculating..."}</h4></p>
            }
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
