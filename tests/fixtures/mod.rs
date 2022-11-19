use once_cell::sync::OnceCell;
use rstest::*;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Mutex};
use std::thread;

/// Path to our binary
fn bin_path() -> PathBuf {
    assert_cmd::cargo::cargo_bin(env!("CARGO_PKG_NAME"))
}

pub struct MyProcess {
    child: Child,
    stdin: Mutex<mpsc::Sender<Vec<u8>>>,
    stdout: Mutex<mpsc::Receiver<String>>,
    stderr: Mutex<mpsc::Receiver<String>>,
}

impl MyProcess {
    pub fn spawn() -> io::Result<Self> {
        let mut child = Command::new(bin_path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = {
            let mut stdin = child.stdin.take().unwrap();
            let (tx, rx) = mpsc::channel::<Vec<u8>>();
            thread::spawn(move || {
                while let Ok(data) = rx.recv() {
                    if let Err(x) = stdin.write_all(&data) {
                        eprintln!("Error: {x}");
                        break;
                    }
                }
            });
            tx
        };

        let stdout = {
            let stdout = BufReader::new(child.stdout.take().unwrap());
            Self::spawn_reader(stdout)
        };

        let stderr = {
            let stderr = BufReader::new(child.stderr.take().unwrap());
            Self::spawn_reader(stderr)
        };

        Ok(Self {
            child,
            stdin: Mutex::new(stdin),
            stdout: Mutex::new(stdout),
            stderr: Mutex::new(stderr),
        })
    }

    pub fn write_stdin(&self, data: impl Into<Vec<u8>>) {
        self.stdin.lock().unwrap().send(data.into()).unwrap()
    }

    pub fn read_stdout_line(&self) -> String {
        self.stdout.lock().unwrap().recv().unwrap()
    }

    pub fn read_stderr_line(&self) -> String {
        self.stderr.lock().unwrap().recv().unwrap()
    }

    fn spawn_reader<T: Read + Send + 'static>(mut reader: BufReader<T>) -> mpsc::Receiver<String> {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line) {
                line.truncate(n);
                if tx.send(line).is_err() {
                    break;
                }
                line = String::new();
            }
        });
        rx
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
