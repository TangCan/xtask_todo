//! Real-binary trycmd snapshots for list filters and sorting.

#[test]
fn list_filters_and_sort_snapshots() {
    trycmd::TestCases::new()
        .register_bin("todo", std::path::PathBuf::from(env!("CARGO_BIN_EXE_todo")))
        .case("tests/trycmd/list_filters.trycmd");
}
