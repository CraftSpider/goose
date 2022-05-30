use assert_cmd::Command;
use std::fs;
use std::time::Duration;

#[test]
fn build_examples() {
    for item in fs::read_dir("examples/").unwrap() {
        let entry = item.unwrap();
        let path = entry.path();

        Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .unwrap()
            .timeout(Duration::from_secs(5))
            .arg(path)
            .unwrap();
    }
}
