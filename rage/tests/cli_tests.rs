#[test]
fn cli_tests() {
    let tests = trycmd::TestCases::new();
    tests.insert_var("[SHA]", env!("RAGE_GIT_SHA")).unwrap();

    tests.case("tests/cmd/rage/*.toml");
    tests.case("tests/cmd/rage-keygen/*.toml");

    tests.case("tests/unix/*/*.toml");
}
