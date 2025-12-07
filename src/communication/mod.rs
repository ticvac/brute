use crate::messages::Message;
use crate::utils::node::Node;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use crate::messages::deserialize;
use crate::messages::MessageType;

pub mod handle_calculate_power_message;
pub mod handle_calculate_power_result;
pub mod handle_solve_problem_message;
pub mod handle_solution_not_found_message;
pub mod handle_solution_found_message;
pub mod handle_stop_calculating_message;
pub mod handle_received_backup_data;

use handle_calculate_power_message::handle_calculate_power_message;
use handle_calculate_power_result::handle_calculate_power_result;
use handle_solve_problem_message::handle_solve_problem_message;
use handle_solution_not_found_message::handle_solution_not_found_message;
use handle_solution_found_message::handle_solution_found_message;
use handle_stop_calculating_message::handle_stop_calculating_message;
use handle_received_backup_data::handle_received_backup_data;

use handle_calculate_power_message::send_calculate_power_messages;

pub fn listen(node: Node) {
    // Extract port from node address and bind to 0.0.0.0 to listen on all interfaces
    let port = node.address.split(':').last().expect("Invalid address format");
    let bind_address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&bind_address).expect("Failed to bind to port");
    println!("Listening on {} (node address: {})", bind_address, node.address);
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
    let mut buffer = [0; 65536];  // Increased from 1024 to 64KB
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
        
        // If we are a leader, send CalculatePower message to recruit this new friend
        if node.is_leader() {
            let node_clone = node.clone();
            let new_friend_address = message.from.clone();
            std::thread::spawn(move || {
                send_calculate_power_messages(&node_clone, &node_clone.address.clone());
            });
            println!("Sent CalculatePower message to new friend: {}", new_friend_address);
        }
    }
    match message.message_type {
        MessageType::Ping => {
            // nothing, just respond with ACK
        }
        MessageType::CalculatePower { leader_address: _ } => {
            handle_calculate_power_message(node, &message.clone());
        }
        MessageType::CalculatePowerResult { power: _ } => {
            handle_calculate_power_result(node, &message.clone());
        }
        MessageType::SolveProblem { .. } => {
            handle_solve_problem_message(node, &message.clone());
        }
        MessageType::Ack => {
            eprintln!("! Received ACK as new connection, ignoring. {:?}", message);
        }
        MessageType::SolutionFound { .. } => {
            handle_solution_found_message(node, &message.clone());
        }
        MessageType::SolutionNotFound => {
            handle_solution_not_found_message(node, &message.clone());
        }
        MessageType::StopSolving => {
            handle_stop_calculating_message(node);
        }
        MessageType::BackupData { .. } => {
            handle_received_backup_data(node, &message.clone());
        }
        MessageType::IAmANewLeader => {
            node.set_leader_address(message.from.clone());
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