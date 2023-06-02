use std::{net::{TcpStream, Shutdown}, mem::size_of, error::Error};
use anyhow::Result;
use chrono::Utc;
use colored::Colorize;
use env_logger::Env;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use bincode::{serialize};
use std::env;
use std::io::{Read, Write};

use crate::{constants::{ZERO_COUNT, MAX_DURATION, MAX_TRIES}, WISDOMS, HOST, PORT};

use super::hashcash::Hashcash;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Command {
    RequestService,
    Challenge,
    Response,
    GrantService,
    InvalidHashcash,
}



#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Message {
    pub cmd: Command,
    pub data: String,
}

pub fn send_data<V>(mut stream: TcpStream, value: &V) -> Result<()> 
where V: Serialize
{
    let data_content = serialize(value)?;
    let data_size = serialize(&data_content.len())?;

    stream.write_all(&data_size)?;
    stream.write_all(&data_content)?;

    Ok(())
}

pub fn receive_data<R>(mut stream: TcpStream) -> Result<R> 
where R: DeserializeOwned,
{
    let mut size_buffer = [0u8; size_of::<usize>()];
    stream.read(&mut size_buffer)?;
    let data_size = usize::from_le_bytes(size_buffer);
    
    let mut data_buffer = vec![0u8; data_size];
    stream.read(&mut data_buffer)?;
    
    let data = bincode::deserialize::<R>(&data_buffer)?;
    Ok(data)
}

pub fn server_addr() -> String {
    let host = env::var("HOST").unwrap_or_else(|_| HOST.to_string());
    let port = env::var("PORT").unwrap_or_else(|_| PORT.to_string());
    format!("{}:{}", host, port)
}

pub fn setup_env_logger() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
}

pub fn handle_client_request(stream: &mut TcpStream) -> Result<(), anyhow::Error> {

    let zero_count = env::var("ZERO_COUNT").unwrap_or_else(|_| format!("{}", ZERO_COUNT)).parse().unwrap();
    let duration = env::var("MAX_DURATION").unwrap_or_else(|_| format!("{}", MAX_DURATION)).parse().unwrap();

    let client_addr = format!("{}", stream.peer_addr()?);
    log::info!("New TCP connection: {}", client_addr);

    let hashcash = Hashcash::new(zero_count, client_addr.clone());
    let rand_value = hashcash.rand;
    loop {
        let msg: Message =  receive_data(stream.try_clone()?)?;
        match msg.cmd {
            Command::RequestService => {
                log::info!("Received Request Challenge");
                send_data(stream.try_clone()?, &Message{
                    cmd: Command::Response,
                    data: serde_json::to_string(&hashcash)?,
                })?;
                log::info!("Sending Hashcash");
            }
            Command::Challenge => {
                log::info!("Received Request Service");
                let received_hashcash:Hashcash = serde_json::from_str(&msg.data)?;

                if client_addr.clone() != received_hashcash.resource {
                    log::error!("Invalid hashcash resource")
                }

                if Utc::now().timestamp() - received_hashcash.date > duration {
                    log::error!("Challenge expired or not sent");
                }

                if received_hashcash.rand != rand_value {
                    log::error!("Challenge expired");
                }

                if received_hashcash.is_valid() {
                    log::info!("Received Hashcash is valid");

                    send_data(stream.try_clone()?, &Message{
                        cmd: Command::GrantService,
                        data: WISDOMS.choose(& mut rand::thread_rng()).unwrap().to_string()
                    })?;
                } else {
                    log::error!("Received Hashcash is invalid");
                    send_data(stream.try_clone()?, &Message{
                        cmd: Command::InvalidHashcash,
                        data: "".to_string()
                    })?;
                }
                stream.shutdown(Shutdown::Both)?;
                log::info!("Connection closed");
                break;
            }
            _ => {
                log::error!("Mismatching command type");
            }
        }
    }
    Ok(())
}

pub fn handle_server_response(stream: &mut TcpStream) -> Result<String, Box<dyn Error>> {

    let max_iter:i32 = env::var("MAX_TRIES").unwrap_or_else(|_| format!("{}", MAX_TRIES)).parse().unwrap();

    log::info!("Requesting challenge");
    send_data(stream.try_clone()?, &Message{
        cmd: Command::RequestService,
        data: "".to_string()
    })?;

    let msg:Message = receive_data(stream.try_clone()?)?;
    if msg.cmd != Command::Response {
        return Err("Read message from stream ".into());
    }
    log::info!("Received Hashcash to solve");
    let mut hashcash:Hashcash = serde_json::from_str(&msg.data)?;

    log::info!("Brute Forcing....");
    hashcash.try_work(max_iter)?;

    log::info!("Computed Hashcash {}", hashcash.to_string().red());
    send_data(stream.try_clone()?, &Message{
        cmd: Command::Challenge,
        data: serde_json::to_string(&hashcash)?
    })?;

    let msg:Message = receive_data(stream.try_clone()?)?;
    if msg.cmd == Command::InvalidHashcash {
        return Err("The hashcash is invalid".into());
    } else if msg.cmd != Command::GrantService {
        return Err("Read message from stream".into());
    }
    
    // log::info!("Received word of wisdom: {}", msg.data);
    stream.shutdown(Shutdown::Both)?;
    Ok(msg.data)
}