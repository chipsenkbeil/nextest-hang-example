mod fixtures;
use fixtures::*;
use rstest::*;

#[rstest]
#[test_log::test]
fn should_run_successfully(proc: &'static MyProcess) {
    assert_eq!(proc.send_msg("hello").unwrap(), "Response = hello\n");
}
