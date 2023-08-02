#![cfg(not(target_arch = "wasm32"))]

#[test]
fn macro_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/reactor/*-fail.rs");
    t.pass("tests/reactor/*-pass.rs");
}
