mod fixtures;
use fixtures::*;
use rstest::*;

#[rstest]
#[test_log::test]
fn should_run_successfully(proc: &'static MyProcess) {
    assert_eq!(proc.read_stdout_line(), "iter 0\n");
    assert_eq!(proc.read_stdout_line(), "iter 1\n");
    assert_eq!(proc.read_stdout_line(), "iter 2\n");
}
