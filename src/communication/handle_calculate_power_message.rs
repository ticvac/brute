use crate::messages::{Message, MessageType};
use crate::utils::node::Node;
use crate::messages::send_message::send_message;


pub fn handle_calculate_power_message(node: &Node, message: &Message) {
    if !node.is_idle() {
        println!("Node is not idle. Cannot process CalculatePower message.");
        return;
    }
    let leader_address = match message.clone().message_type {
        MessageType::CalculatePower { leader_address } => leader_address,
        _ => {
            eprintln!("Invalid message type for handle_calculate_power_message.");
            return;
        }
    };
    
    // transition to child
    node.transition_to_child(leader_address.clone());

    // send CalculatePower messages to friends
    send_calculate_power_messages(node, &leader_address);

    // add leader as friend
    if !node.is_friend(&leader_address) {
        node.add_friend(leader_address.clone());
    }
    // send message back to leader with power
    let node_clone = node.clone();
    std::thread::spawn(move || {
        let my_power = node_clone.power.clone();
        let message = Message::new(
            node_clone.address.clone(),
            leader_address,
            MessageType::CalculatePowerResult { power: my_power },
        );
        send_message(&message, &node_clone);
    });
    
}

pub fn send_calculate_power_messages(node: &Node, leader_address: &str) {
    let friends = node.friends.lock().unwrap();
    let friends_addresses: Vec<String> = {
        friends.iter().map(|f| f.address.clone()).collect()
    };
    drop(friends);

    for friend_address in friends_addresses {
        let node_clone = node.clone();
        let message = Message::new(
            node.address.clone(),
            friend_address.clone(),
            MessageType::CalculatePower { leader_address: leader_address.to_string() },
        );
        std::thread::spawn(move || {
            send_message(&message, &node_clone);
        });
    }
}