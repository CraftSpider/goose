
use std::fs;
use assert_cmd::Command;

#[test]
fn build_examples() {
    for item in fs::read_dir("examples/").unwrap() {
        let entry = item.unwrap();
        let path = entry.path();

        Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .unwrap()
            .arg(path)
            .unwrap();
    }
}
