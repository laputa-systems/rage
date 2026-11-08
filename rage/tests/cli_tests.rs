#[test]
fn cli_tests() {
    let tests = trycmd::TestCases::new();

    tests.case("tests/cmd/rage/*.toml");
    tests.case("tests/cmd/rage-keygen/*.toml");
    #[cfg(feature = "mount")]
    tests.case("tests/cmd/rage-mount/*.toml");

    #[cfg(unix)]
    tests.case("tests/unix/*/*.toml");
}
