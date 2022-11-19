use once_cell::sync::OnceCell;
use rstest::*;
use std::io;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

/// Path to our binary
fn bin_path() -> PathBuf {
    assert_cmd::cargo::cargo_bin(env!("CARGO_PKG_NAME"))
}

pub struct MyProcess {
    child: Child,
    pub name: String,
}

impl MyProcess {
    pub fn spawn() -> io::Result<Self> {
        let name = format!("test-server-{}", rand::random::<u16>());
        let mut child = Command::new(bin_path())
            .arg("server")
            .arg(&name)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Self { child, name })
    }

    /// Sends a message to the server by invoking `{bin} client {name} {msg}`,
    /// returning the response.
    pub fn send_msg(&self, msg: &str) -> io::Result<String> {
        let output = Command::new(bin_path())
            .arg("client")
            .arg(&self.name)
            .arg(msg)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Example of a process being started that cannot be killed during nextest
/// since its lifetime is the entire duration of testing
#[fixture]
pub fn proc() -> &'static MyProcess {
    static INSTANCE: OnceCell<MyProcess> = OnceCell::new();

    INSTANCE
        .get_or_try_init(MyProcess::spawn)
        .expect("Failed to spawn process")
}
