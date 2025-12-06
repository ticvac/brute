use crate::messages::Message;
use crate::utils::node::Node;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use crate::messages::deserialize;
use crate::messages::MessageType;

pub mod handle_calculate_power_message;
pub mod handle_calculate_power_result;

use handle_calculate_power_message::handle_calculate_power_message;
use handle_calculate_power_result::handle_calculate_power_result;

pub fn listen(node: Node) {
    let listener = TcpListener::bind(&node.address).expect("Failed to bind to port");
    // listen loop
    for stream in listener.incoming() {
        // communication
        if !node.is_communicating() {
            println!("Incoming connection rejected because node is not communicating.");
            continue; 
        }
        match stream {
            Ok(mut stream) => {
                if let Some(message) = read_message(&mut stream) {
                    // received message
                    println!("Received message: {}", message);
                    let node_clone = node.clone();
                    let mut stream_clone = stream.try_clone().unwrap();
                    let message_clone = message.clone();
                    // spawn thread to handle message
                    std::thread::spawn(move || {
                        // parsing message
                        let message = deserialize(&message_clone);
                        // handling new connection
                        if let Some(msg) = message {
                            handle_new_connection(&node_clone, &mut stream_clone, msg);
                        } else {
                            eprintln!("Failed to deserialize message: {}", message_clone);
                        }
                    });
                } else {
                    eprintln!("Failed to read message from incoming connection.");
                }
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

fn read_message(_stream: &mut TcpStream) -> Option<String> {
    let mut buffer = [0; 1024];
    match _stream.read(&mut buffer) {
        Ok(size) if size > 0 => {
            let message = String::from_utf8_lossy(&buffer[..size]).to_string();
            Some(message)
        }
        _ => None,
    }
}

fn handle_new_connection(node: &Node, stream: &mut TcpStream, message: Message) {
    // adding friend if not exists
    if !node.is_friend(&message.from) {
        println!("Received message from non-friend: {}", message.from);
        node.add_friend(message.from.clone());
    }
    match message.message_type {
        MessageType::Ping => {
            // nothing, just respond with ACK
        }
        MessageType::CalculatePower { leader_address: _ } => {
            handle_calculate_power_message(node, &message);
        }
        MessageType::CalculatePowerResult { power: _ } => {
            handle_calculate_power_result(node, &message);
        }
        MessageType::Ack => {
            eprintln!("! Received ACK as new connection, ignoring. {:?}", message);
        }
    }
    send_ack_back(node, &message, stream);
}

fn send_ack_back(node: &Node, responding_to_message: &Message, stream: &mut TcpStream) {
    let ack_message = Message::new(
        node.address.clone(),
        responding_to_message.from.clone(),
        MessageType::Ack,
    );
    let serialized_ack = ack_message.serialize();
    println!("Sending ACK back to message {:?}", responding_to_message);
    let _  = stream.write_all(serialized_ack.as_bytes());
}