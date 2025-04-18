#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .default_bin_name("protomd")
        .case("tests/cmd/*.toml");
}
