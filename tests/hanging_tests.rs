mod fixtures;
use fixtures::*;

use rstest::*;
use std::process::Child;

#[rstest]
#[test_log::test]
fn should_run_successfully(_child: &'static Child) {
    assert_eq!(1, 1);
}
