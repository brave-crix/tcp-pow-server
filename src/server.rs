use anyhow::Result;
use std::error::Error;
use std::thread;
use std::net::TcpListener;
use lib::*;
use word_of_wisdom::{handle_client_request, server_addr, setup_env_logger};

fn main() -> Result<(), Box<dyn Error>> {
    setup_env_logger();

    let addr = server_addr();
    let listener = TcpListener::bind(addr.as_str())?;
    log::info!("Listening on {}", addr);
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                thread::spawn(move || {
                    if let Err(e) = handle_client_request(&mut stream) {
                        eprintln!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                log::error!("Establishing TCP connection is failed: {}", e);
            }
        }
    }
    Ok(())
}
