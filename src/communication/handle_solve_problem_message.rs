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
    node.transition_to_child_solving(problem);
    
    // start solving...
    let node_clone = node.clone();
    let stop_flag = std::sync::atomic::AtomicBool::new(false);
    println!("Starting to solve problem...");
    std::thread::spawn(move || {
        match problem_clone.brute_force(&stop_flag) {
            Some(solution) => {
                println!("Problem solved! Solution: {}", solution);
                // send solution back to leader
                let solution_message = Message::new(
                    node_clone.address.clone(),
                    node_clone.get_leader_address(),
                    MessageType::SolutionFound { solution }
                );
                let _ = send_message(&solution_message, &node_clone);
                // TODO handle if cant reach parent
            }
            None => {
                println!("Problem could not be solved in the given range.");
                let message = Message::new(
                    node_clone.address.clone(),
                    node_clone.get_leader_address(),
                    MessageType::SolutionNotFound
                );
                let _ = send_message(&message, &node_clone);
                // TODO handle if cant reach parent
            }
        }
    });

}