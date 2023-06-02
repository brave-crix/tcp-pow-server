use anyhow::Result;
use std::error::Error;
use std::net::TcpStream;
use colored::Colorize;

use lib::*;
use word_of_wisdom::{handle_server_response, server_addr, setup_env_logger};

fn main() -> Result<(), Box<dyn Error>> {
    setup_env_logger();
    let addr = server_addr();
    let mut stream = TcpStream::connect(addr.as_str())?;
    match handle_server_response(&mut stream) {
        Ok(r) => {
            log::info!("Word of wsdom: {}", r.yellow());
            Ok(())
        }
        Err(e) => {
            log::error!("{}", e);
            Err(e.into())
        }
    }
}
