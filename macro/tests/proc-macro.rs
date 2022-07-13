#[test]
fn proc_macro() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/proc-macro-tests/*.rs");
}

fn main() {}
