use locksmith::{DBObject, InspectedStatement, TableLock};
use std::collections::HashSet;
use std::str::FromStr;

/// A test case for the `QueryOracle` integration tests.
/// These are a combination of a SQL statement and the expected results of the
/// `QueryOracle` inspection. Example:
///
/// ```json
/// -- lock:    {"table": {"name": "orders"}, "lock": "AccessExclusiveLock"}
/// -- lock:    {"table": {"name": "customers"}, "lock": "AccessExclusiveLock"}
/// -- removed: {"Column": {"table": {"name": "customers"}, "name": "id", "data_type": "integer"}}
/// -- added:   {"Column": {"table": {"name": "customers"}, "name": "id", "data_type": "bigint"}}
/// -- rewrite: {"Table": {"name": "customers"}}
/// alter table customers alter column id type bigint;
/// ```
///
/// See the `queries` directory for more examples.
#[derive(Debug, Default)]
pub struct TestCase {
    pub statement: String,
    pub expected_locks: HashSet<TableLock>,
    pub expected_removals: HashSet<DBObject>,
    pub expected_additions: HashSet<DBObject>,
    pub expected_rewrites: HashSet<DBObject>,
}

impl TestCase {
    pub fn check_result(self, result: InspectedStatement) {
        let expected = InspectedStatement {
            added_objects: self.expected_additions,
            removed_objects: self.expected_removals,
            locks: self.expected_locks,
            rewrites: self.expected_rewrites,
        };
        assert_eq!(
            expected, result,
            "Result mismatch:\n{expected:#?}\n\n{result:#?}"
        );
    }
}

impl FromStr for TestCase {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut test_case = TestCase {
            statement: s.to_string(),
            ..Default::default()
        };
        for line in s.lines() {
            // All expectations come in the form of comments, with the syntax
            // -- [expectation]: [json serialized object]
            if !line.starts_with("--") {
                continue;
            }
            let (expectation, json) = line.split_once(": ").expect("Invalid line format");
            let expectation = expectation.trim_start_matches("-- ");
            match expectation {
                "lock" => {
                    let obj: TableLock = serde_json::from_str(json)
                        .unwrap_or_else(|_| panic!("Invalid lock line: {}", line));
                    test_case.expected_locks.insert(obj);
                }
                "removed" => {
                    let obj: DBObject = serde_json::from_str(json)
                        .unwrap_or_else(|_| panic!("Invalid removed line: {}", line));
                    test_case.expected_removals.insert(obj);
                }
                "added" => {
                    let obj: DBObject = serde_json::from_str(json)
                        .unwrap_or_else(|_| panic!("Invalid addition line: {}", line));
                    test_case.expected_additions.insert(obj);
                }
                "rewrite" => {
                    let obj: DBObject = serde_json::from_str(json)
                        .unwrap_or_else(|_| panic!("Invalid rewrite line: {}", line));
                    test_case.expected_rewrites.insert(obj);
                }
                _ => continue,
            }
        }

        assert!(
            !test_case.expected_locks.is_empty(),
            "No lock expectations found in test case"
        );
        Ok(test_case)
    }
}
