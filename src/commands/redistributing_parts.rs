use crate::utils::node::{Node, NodeState, LeaderState};
use crate::utils::friend::{FriendType, FriendTypeChildState};
use crate::problem::{PartOfAProblem, PartOfAProblemState, Combinable};
use crate::messages::send_message::send_message;
use crate::messages::{Message, MessageType};
use crate::communication::handle_solution_found_message::stop_and_send_stop_messages;
use std::thread;
use std::time::Duration;

/// Starts a background thread that monitors for waiting nodes and redistributes parts to them.
/// Every 5 seconds, it checks for children waiting for problem parts and assigns them work.
pub fn start_redistributing_parts(node: &Node) {
    let node_clone = node.clone();
    thread::spawn(move || {
        println!("[WaitingNodesChecker] Started monitoring for waiting nodes.");
        
        loop {
            thread::sleep(Duration::from_secs(5));

            // Check if leader is still solving, if not, stop the checker
            if !node_clone.is_leader_solving() {
                println!("[WaitingNodesChecker] Leader is no longer solving. Stopping checker.");
                break;
            }

            // Check if entire space has been searched
            if is_entire_space_searched(&node_clone) {
                println!("[WaitingNodesChecker] Entire search space has been searched.");
                print_problem_parts(&node_clone);
                println!("----- NO SOLUTION FOUND -----");
                stop_and_send_stop_messages(&node_clone);
                break;
            }
            
            // Get waiting children with their power
            let waiting_children = get_waiting_children(&node_clone);
            
            if waiting_children.is_empty() {
                continue;
            }
            
            println!("[WaitingNodesChecker] Found {} waiting children", waiting_children.len());
            
            for (child_address, child_power) in waiting_children {
                // Calculate how many combinations this child can handle in 5 seconds
                // power is in k hashes/sec, so 5 seconds = 5 * power * 1000 combinations
                let max_combinations_for_5sec = (5 * child_power as usize * 1000).max(1);
                
                // Get the largest not distributed part and potentially split it
                let part_to_send = get_part_for_child(&node_clone, max_combinations_for_5sec);
                
                if let Some(part) = part_to_send {
                    println!(
                        "[WaitingNodesChecker] Sending part [{} - {}] ({} combinations) to child {} (can handle {} in 5s)",
                        part.start, part.end, part.total_combinations(), child_address, max_combinations_for_5sec
                    );
                    
                    // Send the part to the child
                    send_part_to_child(&node_clone, &child_address, part);
                } else {
                    println!("[WaitingNodesChecker] No parts available for child {}", child_address);
                }
            }
        }
        
        println!("[WaitingNodesChecker] Stopped.");
    });
}

/// Returns a list of children waiting for problem parts with their power.
fn get_waiting_children(node: &Node) -> Vec<(String, u32)> {
    let friends = node.friends.lock().unwrap();
    friends
        .iter()
        .filter_map(|f| {
            if let FriendType::Child { power, state: FriendTypeChildState::WaitingForProblemParts } = &f.friend_type {
                Some((f.address.clone(), *power))
            } else {
                None
            }
        })
        .collect()
}

/// Gets a part for a child, potentially splitting a larger part if needed.
/// Returns None if no not-distributed parts are available.
fn get_part_for_child(node: &Node, max_combinations: usize) -> Option<PartOfAProblem> {
    let mut state = node.state.lock().unwrap();
    
    if let NodeState::Leader { state: LeaderState::Solving { parts }, .. } = &mut *state {
        // Find the largest not distributed part
        let largest_idx = parts
            .iter()
            .enumerate()
            .filter(|(_, p)| p.state == PartOfAProblemState::NotDistributed)
            .max_by_key(|(_, p)| p.total_combinations())
            .map(|(i, _)| i);
        
        if let Some(idx) = largest_idx {
            let part = parts.remove(idx);
            let (mut part_to_send, remaining) = part.split_at_combinations(max_combinations);
            
            // Mark the part we're sending as distributed
            part_to_send.state = PartOfAProblemState::Distributed;
            parts.push(part_to_send.clone());
            
            // Add the remaining part back if it exists
            if let Some(remaining_part) = remaining {
                parts.push(remaining_part);
            }
            
            return Some(part_to_send);
        }
    }
    
    None
}

/// Sends a part to a child and updates the friend state
fn send_part_to_child(node: &Node, child_address: &str, part: PartOfAProblem) {
    let message = Message::new(
        node.address.clone(),
        child_address.to_string(),
        MessageType::SolveProblem {
            start: part.start.clone(),
            end: part.end.clone(),
            alphabet: part.alphabet.clone(),
            hash: part.hash.clone(),
        },
    );
    
    let res = send_message(&message, node);
    if res.is_none() {
        eprintln!("[WaitingNodesChecker] Failed to send part to child: {}", child_address);
        node.remove_friend(child_address);
        
        // Mark the part back as NotDistributed since we couldn't send it
        let mut state = node.state.lock().unwrap();
        if let NodeState::Leader { state: LeaderState::Solving { parts }, .. } = &mut *state {
            // Find the part we just tried to send and mark it as not distributed
            for p in parts.iter_mut() {
                if p.start == part.start && p.end == part.end {
                    p.state = PartOfAProblemState::NotDistributed;
                    break;
                }
            }
        }
    } else {
        // Update friend state to Solving with the assigned part
        node.set_friend_child_state_solving(child_address, part);
    }
}

/// Checks if the entire search space has been searched (all parts are SearchedAndNotFound).
fn is_entire_space_searched(node: &Node) -> bool {
    let state = node.state.lock().unwrap();
    if let NodeState::Leader { state: LeaderState::Solving { parts }, .. } = &*state {
        !parts.is_empty() && parts.iter().all(|p| p.state == PartOfAProblemState::SearchedAndNotFound)
    } else {
        false
    }
}

/// Prints all problem parts with their states.
fn print_problem_parts(node: &Node) {
    let state = node.state.lock().unwrap();
    if let NodeState::Leader { state: LeaderState::Solving { parts }, .. } = &*state {
        println!("----- PROBLEM PARTS -----");
        for part in parts {
            println!("  [{} - {}] state: {:?}, combinations: {}", 
                part.start, part.end, part.state, part.total_combinations());
        }
        println!("-------------------------");
    }
}
