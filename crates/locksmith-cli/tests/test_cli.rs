use assert_cmd::cargo;
use locksmith::{InspectedStatement, Lock, TableLock};
use std::collections::HashSet;

const TEST_SCHEMA_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../locksmith/tests/test_schema.sql"
);

#[test]
fn test_basic_cli() {
    let mut cmd = cargo::cargo_bin_cmd!("locksmith-cli");
    let assert = cmd
        .arg(TEST_SCHEMA_PATH)
        .arg("select * from customers")
        .assert()
        .success();
    let output = assert.get_output();
    let expected = InspectedStatement {
        locks: HashSet::from([TableLock {
            table: "customers".into(),
            lock: Lock::AccessShareLock,
        }]),
        ..Default::default()
    };
    let output: InspectedStatement = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output, expected)
}
