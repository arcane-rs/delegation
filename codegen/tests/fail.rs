#[rustversion::stable(1.84)]
#[test]
fn fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/*/*.rs");
}
