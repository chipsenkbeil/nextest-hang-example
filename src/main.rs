use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};

#[tokio::main]
async fn main() {
    // Create a pipe to listen to on Windows
    let _pipe = ServerOptions::new()
        .first_pipe_instance(true)
        .create(format!("test-pipe-{}", rand::random::<u16>()))?;

    let mut i = 0;
    loop {
        println!("iter {i}");
        i += 1;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
