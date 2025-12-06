use crate::utils::node::Node;
use crate::messages::{Message, MessageType};

pub fn handle_calculate_power_result(node: &Node, message: &Message) {
    if !node.is_leader() {
        eprintln!("! Received CalculatePowerResult but node is not a leader. Ignoring.");
        return;
    }
    let power = match message.message_type {
        MessageType::CalculatePowerResult { power } => power,
        _ => {
            eprintln!("! Invalid message type for CalculatePowerResult handler.");
            return;
        }
    };
    let from_address = message.from.clone();
    // update node to child with received power
    node.transition_friend_to_child(from_address, power);
}