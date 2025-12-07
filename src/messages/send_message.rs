use crate::messages::{Message, deserialize};
use crate::utils::node::Node;
use std::net::TcpStream;
use std::time::Duration;
use std::io::{Write, Read};

// if returned None, message sending failed
pub fn send_message(message: &Message, node: &Node) -> Option<Message> {
    // prevent sending if not communicating
    if !node.is_communicating() {
        println!("Node is not communicating. Cannot send message.");
        return None;
    }
    // prevent sending to self
    if message.to == node.address {
        println!("Cannot send message to self.");
        return None;
    }
    // prevent sending to non-friend
    if !node.is_friend(&message.to) {
        println!("Cannot send message to non-friend: {}", message.to);
        return None;
    }
    // sending message
    match TcpStream::connect_timeout(
        &message.to.parse().unwrap(),
        Duration::from_secs(3)
    ) {
        Ok(mut stream) => {
            // Set timeouts
            let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));
            let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));

            let serialized_message = message.serialize();
            // sending the message
            if let Err(e) = stream.write_all(serialized_message.as_bytes()) {
                eprintln!("Failed to write to {}: {}", message.to, e);
                return None;
            }

            // waiting for response
            let mut buffer = [0u8; 2048];
            match stream.read(&mut buffer) {
                Ok(size) if size > 0 => {
                    let response_str = String::from_utf8_lossy(&buffer[..size]).to_string();
                    if let Some(response_message) = deserialize(&response_str) {
                        return Some(response_message);
                    } else {
                        eprintln!("Failed to deserialize response from {}", message.to);
                        return None;
                    }
                }
                Ok(_) | Err(_) => {
                    eprintln!("No response received from {}", message.to);
                    return None;
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to {}: {}", message.to, e);
            return None;
        }
    }
}