use crate::communication::handle_calculate_power_message::send_calculate_power_messages;
use crate::utils::node::Node;

pub fn handle_cal_command(node: &Node) {
    if !node.is_idle() {
        println!("Node is not idle. Cannot process 'cal' command.");
        return;
    }
    // set to leader
    node.transition_to_leader();
    
    // Now handle the fake received message as if it was received from the network
    send_calculate_power_messages(node, &node.address.clone());
}