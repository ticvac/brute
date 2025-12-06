
use crate::utils::{node::Node, parse_address};
use crate::messages::{MessageType, Message};
use crate::messages::send_message::send_message;

// removes friend if no response or unexpected response
pub fn handle_ping_command(node: &Node, parts: Vec<&str>) {
    if parts.len() < 2 {
        println!("Usage: ping <address>");
        return;
    }
    let to_address = parse_address(parts[1]);

    let message = Message::new(
        node.address.clone(),
        to_address.clone(),
        MessageType::Ping,
    );

    // if not in friends, add it first
    if !node.is_friend(&to_address) {
        node.add_friend(to_address.clone());
    }

    println!("Sending PING to {}", to_address);
    let result = send_message(&message, node);

    match result {
        Some(response) => {
            match response.message_type {
                MessageType::Ack => {
                    println!("Received ACK from {}", response.from);
                }
                _ => {
                    println!("Unexpected response type from {}: {:?}", response.from, response.message_type);
                    node.remove_friend(&to_address);
                }
            }
        }
        None => {
            println!("Unable to send to {}", to_address);
            node.remove_friend(&to_address);
        }
    }
}