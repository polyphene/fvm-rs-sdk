#[test]
fn proc_macro() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/proc-macro-tests/*_fail.rs");
    t.pass("tests/proc-macro-tests/*_pass.rs")
}

fn main() {}
