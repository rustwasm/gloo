#![cfg(target_arch = "wasm32")]

use gloo_ani_frames::*;
use futures::prelude::*;

use wasm_bindgen_test::*;
use std::future::Future;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
pub fn test() -> impl Future<> {
    let mut is_executed = false;
    let index1 = Animation::request_frame(|| {
        is_executed = true;
    });
    let index2 = Animation::request_frame(|| {
        unreachable!()
    });
    Animation::cancel_frame(index2);

    Animation::request_frame(|| {
        Animation::request_frame(|| {
            assert_eq!(is_executed, true);

            let mut ani = Animation::<()>::new();

            let mut is_executed = false;
            let ix1 = ani.add(|s| {
                unreachable!();
            });
            let ix2 = ani.add(|s| {
                is_executed = true
            });
            ani.remove(ix1);
            ani.add(|s| {
                assert_eq!();
            });
        });
    });
    ;
}