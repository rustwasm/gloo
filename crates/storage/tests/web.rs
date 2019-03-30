#![cfg(target_arch = "wasm32")]
#![cfg(feature = "json-storage")]

use gloo_storage::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn local_storage() {
    let mut s = JsonStorage::with_local_storage("some-storage");
    s.add_version(1, |v| {
        v.add_and_update_table("first-table", |t| t.add_row_with_index("id", Index::Unique))
    });
    s.add_version(2, |v| {
        v.add_table("second-table")
            .update_table("first-table", |t| t.remove_old_row("id"))
    });
}
