use crate::messages::send_message::send_message;
use crate::utils::node::Node;
use crate::messages::{Message, MessageType};
use crate::problem::{Problem};


pub fn handle_solve_problem_message(node: &Node, message: &Message) {
    println!("Received SolveProblem message. {:?}", message);
    if !node.is_child_connected() {
        eprintln!("Node is not in a state to solve a problem. Ignoring SolveProblem message.");
        return;
    }
    let problem_data = match &message.message_type {
        MessageType::SolveProblem { start, end, alphabet, hash } => {
            (start.clone(), end.clone(), alphabet.clone(), hash.clone())
        }
        _ => {
            eprintln!("Invalid message type for SolveProblem handler.");
            return;
        }
    };
    let problem = Problem::new(
        problem_data.2,  // alphabet
        problem_data.0,  // start
        problem_data.1,  // end
        problem_data.3,  // hash
    );
    let mut problem_clone = problem.clone();
    
    // Reset the stop flag before starting
    node.set_stop_flag(false);
    node.transition_to_child_solving(problem_clone.as_part_of_a_problem());
    
    // start solving...
    let node_clone = node.clone();
    let stop_flag = node.get_stop_flag();
    println!("Starting to solve problem...");
    std::thread::spawn(move || {
        match problem_clone.brute_force(&stop_flag) {
            Some(solution) => {
                println!("Problem solved! Solution: {}", solution);
                // send solution back to leader
                loop {
                    let solution_message = Message::new(
                        node_clone.address.clone(),
                        node_clone.get_leader_address(),
                        MessageType::SolutionFound { solution: solution.clone() }
                    );
                
                    if send_message(&solution_message, &node_clone).is_some() {
                        break;
                    }
                    println!("Could not reach leader, retrying to send solution...");
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }
            None => {
                if node_clone.get_stop_flag().load(std::sync::atomic::Ordering::Relaxed) {
                    println!("Problem solving was stopped before completion.");
                    return;
                }
                loop {
                    println!("Could not find a solution in the given range, notifying leader...");
                    let solution_message = Message::new(
                        node_clone.address.clone(),
                        node_clone.get_leader_address(),
                        MessageType::SolutionNotFound
                    );
                    if send_message(&solution_message, &node_clone).is_some() {
                        break;
                    }
                    println!("Could not reach leader, retrying to send no-solution message...");
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }
        }
        // transition back to connected (waiting) state
        node_clone.transition_child_to_connected();
    });

}