use crate::utils::node::Node;
use crate::communication::handle_solution_found_message::stop_and_send_stop_messages;

pub fn handle_stop_command(node: &Node) {
    if !node.is_leader() {
        eprintln!("Node is not a leader. Ignoring stop command.");
        return;
    }
    stop_and_send_stop_messages(node);
}