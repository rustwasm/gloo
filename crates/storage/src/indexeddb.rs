use wasm_bindgen::prelude::*;

pub async fn open(
    name: &str,
    version: u32,
    upgrade_fn: impl FnOnce() + 'static,
) -> Result<Db, JsValue> {
    Ok(Db)
}

pub struct Db;
