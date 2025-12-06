use crate::messages::Message;
use crate::utils::node::Node;

pub fn handle_solution_not_found_message(node: &Node, message: &Message) {
    if !node.is_leader() {
        eprintln!("! Received SolutionNotFound but node is not a leader. Ignoring.");
        return;
    }
    
    let friend_address = &message.from;
    node.handle_solution_not_found_from_friend(friend_address);
}