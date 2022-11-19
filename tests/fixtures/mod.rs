use once_cell::sync::OnceCell;
use rstest::*;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

/// Path to our binary
fn bin_path() -> PathBuf {
    assert_cmd::cargo::cargo_bin(env!("CARGO_PKG_NAME"))
}

/// Example of a process being started that cannot be killed during nextest
/// since its lifetime is the entire duration of testing
#[fixture]
pub fn child() -> &'static Child {
    static INSTANCE: OnceCell<Child> = OnceCell::new();

    INSTANCE
        .get_or_try_init(|| {
            Command::new(bin_path())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        })
        .expect("Failed to spawn process")
}
