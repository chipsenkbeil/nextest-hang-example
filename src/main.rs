use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{ClientOptions, ServerOptions};

// Equivalent to winapi::shared::winerror::ERROR_PIPE_BUSY
// DWORD -> c_uLong -> u32
const ERROR_PIPE_BUSY: u32 = 231;

#[tokio::main]
async fn main() {
    match std::env::args().nth(1).expect("no command given").as_str() {
        "client" => {
            let name = std::env::args().nth(2).expect("no name given");
            let msg = std::env::args().nth(3).expect("no msg given");
            eprintln!("Spawning client to connect to {name} and send {msg}");
            let response = spawn_client(name, msg).await.expect("Client failed");
            println!("Response = {response}");
        }
        "server" => {
            let name = std::env::args().nth(2).expect("no name given");
            eprintln!("Spawning server with name: {name}");
            spawn_server(name).await.expect("Server failed");
        }
    }
}

// Create a pipe to connect, print a message, and receive the response
fn spawn_client(name: String, mut msg: String) -> tokio::task::JoinHandle<String> {
    tokio::spawn(async move {
        let addr = format!("\\\\.\\pipe\\{name}");

        // Continue to try to connect to the server while it is busy, failing on any other error
        let mut pipe = loop {
            match ClientOptions::new().open(&addr) {
                Ok(client) => break client,
                Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY as i32) => (),
                Err(e) => panic!("{e}"),
            }

            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        };

        // Send a message to the server with a termination at the end
        msg.push_str("</TERM>");
        pipe.write_all(msg.as_bytes()).await.unwrap();

        // Wait for a response from the server
        let mut response = String::new();
        let mut buf = [0u8; 1024];
        loop {
            let n = pipe.read(&mut buf).await.unwrap();
            if n == 0 {
                panic!("Server connection ended early");
            }
            response.push_str(&String::from_utf8_lossy(&buf[..n]));
            if let Some(n) = response.find("</TERM>") {
                break response[..n].to_string();
            }
        }
    })
}

// Create a pipe to listen to on Windows
fn spawn_server(name: String) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let addr = format!("\\\\.\\pipe\\{name}");
        let mut pipe = ServerOptions::new()
            .first_pipe_instance(true)
            .create(&addr)
            .expect("Failed to create windows pipe");

        loop {
            // Wait for a new connection on the current server pipe
            pipe.connect()
                .await
                .expect("Failed to receive new connection");

            // Create a new server pipe to use for the next connection
            // as the current pipe is now taken with our existing connection
            let mut pipe = std::mem::replace(
                &mut pipe,
                ServerOptions::new()
                    .create(&addr)
                    .expect("Failed to configure for a new connection"),
            );

            // Spawn a new task to handle the pipe connected to our client that
            // merely echoes the data it receives back out to the client
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                loop {
                    let n = pipe.read(&mut buf).await.unwrap();
                    if n == 0 {
                        break;
                    }

                    pipe.write_all(&buf[..n]).await.unwrap();
                }
            });
        }
    })
}
