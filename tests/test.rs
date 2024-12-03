use trybuild::TestCases;

#[test]
fn builds() {
    let t = TestCases::new();
    t.pass("tests/macros/simple.rs");
}
