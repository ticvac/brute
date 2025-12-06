use crate::utils::node::Node;
use crate::messages::{Message, MessageType};
use crate::messages::send_message::send_message;
use std::thread;

pub fn handle_solution_found_message(node: &Node, message: &Message) {
    println!("Received SolutionFound message. {:?}", message);
    if !node.is_leader() {
        eprintln!("Node is not a leader. Ignoring SolutionFound message.");
        return;
    }
    println!("Solution found by a child node: {:?}", message);
    println!("----- SOLUTION -----");
    if let MessageType::SolutionFound { solution } = &message.message_type {
        println!("Solution: {}", solution);
    }
    println!("--------------------");

    // transition back to leader waiting state
    node.transition_leader_to_waiting();

    // set all children to waiting state before sending stop messages
    node.set_all_children_to_waiting();

    // stop all child nodes
    let child_addresses = node.get_child_addresses();
    for child_address in child_addresses {
        let node_clone = node.clone();
        let node_address = node.address.clone();
        thread::spawn(move || {
            let stop_message = Message::new(
                node_address,
                child_address.clone(),
                MessageType::StopSolving,
            );
            println!("Sending StopSolving message to child: {}", child_address);
            let _ = send_message(&stop_message, &node_clone);
        });
    }
    
}