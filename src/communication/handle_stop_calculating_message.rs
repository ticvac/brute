use crate::utils::node::Node;

pub fn handle_stop_calculating_message(node: &Node) {
    println!("Received StopCalculating message.");
    
    // If child and connected (waiting), ignore
    if node.is_child_connected() {
        println!("Node is child and waiting, ignoring StopCalculating message.");
        return;
    }
    
    // If child and solving, stop the brute force and transition to connected (waiting)
    if node.is_child_solving() {
        println!("Node is child and solving, stopping calculation and transitioning to waiting.");
        node.set_stop_flag(true);
        node.transition_child_to_connected();
        return;
    }
    
    println!("Node is not a child, ignoring StopCalculating message.");
}