mod private;
mod public;
mod queue;

use crate::messages::*;
use crate::worker_ext::{worker_self, WorkerExt};
pub use private::{Private, PrivateWorker};
pub use public::{Public, PublicWorker};

use crate::Worker;
use js_sys::{Array, Reflect};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use web_sys::{Blob, BlobPropertyBag, Url, WorkerOptions};

fn send_to_remote<W>(worker: &web_sys::Worker, msg: ToWorker<W>)
where
    W: Worker,
{
    let msg = msg.pack();
    worker.post_message_vec(msg);
}

fn worker_new(
    name_of_resource: &str,
    resource_is_relative: bool,
    is_module: bool,
) -> web_sys::Worker {
    let origin = gloo_utils::document()
        .location()
        .unwrap_throw()
        .origin()
        .unwrap_throw();
    let pathname = gloo_utils::window().location().pathname().unwrap_throw();

    let prefix = if resource_is_relative {
        pathname
            .rfind(|c| c == '/')
            .map(|i| &pathname[..i])
            .unwrap_or_default()
    } else {
        ""
    };
    let script_url = format!("{}{}/{}", origin, prefix, name_of_resource);
    let wasm_url = format!(
        "{}{}/{}",
        origin,
        prefix,
        name_of_resource.replace(".js", "_bg.wasm")
    );
    let array = Array::new();
    array.push(
        &format!(
            r#"importScripts("{}");wasm_bindgen("{}");"#,
            script_url, wasm_url
        )
        .into(),
    );
    let blob = Blob::new_with_str_sequence_and_options(
        &array,
        BlobPropertyBag::new().type_("application/javascript"),
    )
    .unwrap();
    let url = Url::create_object_url_with_blob(&blob).unwrap();

    if is_module {
        let options = WorkerOptions::new();
        Reflect::set(
            options.as_ref(),
            &JsValue::from_str("type"),
            &JsValue::from_str("module"),
        )
        .unwrap();
        web_sys::Worker::new_with_options(&url, &options).expect("failed to spawn worker")
    } else {
        web_sys::Worker::new(&url).expect("failed to spawn worker")
    }
}
