mod common;
use common::{TestCase, start_integration_test_postgres};
use locksmith::QueryOracle;
use std::str::FromStr;
use tracing_test::traced_test;

macro_rules! test_suite {
    ($($name:ident=$file:literal;)*) => {
        $(test_suite_test!($name=$file);)*
    };
}
macro_rules! test_suite_test {
    ($name:ident=$file:literal) => {
        #[traced_test]
        #[tokio::test]
        async fn $name() {
            let test_case =
                TestCase::from_str(include_str!($file)).expect("Failed to parse test case");
            let (_container, dsn) = start_integration_test_postgres().await;
            let mut oracle = QueryOracle::new(&dsn);
            let result = oracle
                .inspect_statement(&test_case.statement)
                .await
                .unwrap();
            test_case.check_result(result);
        }
    };
}

test_suite! {
    set_not_null="queries/set_not_null.sql";
    alter_column_type="queries/alter_type.sql";
    drop_column="queries/drop_column.sql";
    drop_index="queries/drop_index.sql";
    drop_table="queries/drop_table.sql";
}
