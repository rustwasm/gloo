#[test]
fn macro_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/oneshot/*-fail.rs");
    t.pass("tests/oneshot/*-pass.rs");
}
