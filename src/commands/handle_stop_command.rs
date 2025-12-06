use crate::utils::node::Node;
use crate::messages::send_message::send_message;

pub fn handle_stop_command(node: &Node) {
    if !node.is_leader() {
        eprintln!("Node is not a leader. Ignoring stop command.");
        return;
    }
    node.transition_leader_to_waiting();
    node.set_all_children_to_waiting();
    let child_addresses = node.get_child_addresses();
    for child_address in child_addresses {
        let node_clone = node.clone();
        let node_address = node.address.clone();
        std::thread::spawn(move || {
            let stop_message = crate::messages::Message::new(
                node_address,
                child_address.clone(),
                crate::messages::MessageType::StopSolving,
            );
            println!("Sending StopSolving message to child: {}", child_address);
            let _ = send_message(&stop_message, &node_clone);
        });
    }
}