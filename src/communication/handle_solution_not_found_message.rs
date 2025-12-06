use crate::messages::Message;
use crate::utils::node::Node;
use crate::utils::backups::send_backup_data;

pub fn handle_solution_not_found_message(node: &Node, message: &Message) {
    if !node.is_leader() {
        eprintln!("! Received SolutionNotFound but node is not a leader. Ignoring.");
        return;
    }
    
    let friend_address = &message.from;
    node.handle_solution_not_found_from_friend(friend_address);
    // notify backup
    send_backup_data(node);

    // TODO check if solved entire space
}