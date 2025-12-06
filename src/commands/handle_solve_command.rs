use crate::utils::backups::send_backup_data;
use crate::utils::node::Node;
use crate::utils::friend::{FriendType, FriendTypeChildState};
use crate::utils::watcher::start_watcher;
use crate::problem::{merge_parts, PartOfAProblem, Problem, PartOfAProblemState};
use crate::messages::send_message::send_message;
use crate::messages::{Message, MessageType};

pub fn handle_solve_command(node: &Node, parts: Vec<&str>) {
    if !node.is_leader_waiting_for_problem() {
        eprintln!("Node is not in a state to start solving a problem. run 'cal' command first.");
        return;
    }
    if parts.len() < 4 {
        println!("Usage: solve <alphabet> <min_len> <max_len> <target_hash>");
        println!("Example: solve abc 2 3 ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb");
        return;
    }
    // parsing input
    let alphabet = parts[1].to_string();
    let min_length = match parts[2].parse::<usize>() {
        Ok(n) => n,
        Err(_) => {
            println!("Invalid min_length: {}", parts[2]);
            return;
        }
    };
    let max_length = match parts[3].parse::<usize>() {
        Ok(n) => n,
        Err(_) => {
            println!("Invalid max_length: {}", parts[3]);
            return;
        }
    };
    let hash = parts[4].to_string();
    let start = alphabet.chars().next().unwrap().to_string().repeat(min_length);
    let end = alphabet.chars().last().unwrap().to_string().repeat(max_length);
    
    // creating problem
    let problem = Problem::new(alphabet, start, end, hash);
    // setting problem to node
    node.set_problem(problem.clone());

    // distributing problem parts to friends
    let total_power = node.get_total_power_of_friends();

    println!("Distributing problem among friends with total power: {}", total_power);
    let keep_percentage = 25.0;
    let parts = problem.divide_into_n_and_keep_percentage(total_power as usize, keep_percentage);
    let mut parts = merge_parts_by_child_strength(node, parts);

    // send parts to children
    send_problem_parts_to_children(node, &mut parts);
    // save parts to leader
    node.set_problem_parts(parts);
    
    // start watcher to monitor children
    start_watcher(node.clone());

    send_backup_data(node);
}


/// Merges parts based on each child's power.
/// For each child, takes `power` number of parts and merges them into one.
fn merge_parts_by_child_strength(node: &Node, parts: Vec<PartOfAProblem>) -> Vec<PartOfAProblem> {
    let friends = node.friends.lock().unwrap();
    
    // Collect children with their power (only those waiting for problem parts)
    let children: Vec<u32> = friends
        .iter()
        .filter_map(|f| match &f.friend_type {
            FriendType::Child { power, state: FriendTypeChildState::WaitingForProblemParts } => Some(*power),
            _ => None,
        })
        .collect();
    
    drop(friends); // Release the lock
    
    if children.is_empty() || parts.is_empty() {
        return parts;
    }
    
    let mut result = Vec::new();
    let mut part_index = 0;
    
    for child_power in children {
        let power = child_power as usize;
        if power == 0 {
            continue;
        }
        
        // Take `power` parts for this child
        let end_index = (part_index + power).min(parts.len());
        if part_index >= parts.len() {
            break;
        }
        
        let parts_for_child: Vec<PartOfAProblem> = parts[part_index..end_index].to_vec();
        
        if parts_for_child.len() == 1 {
            result.push(parts_for_child[0].clone());
        } else if !parts_for_child.is_empty() {
            // Merge the parts into one
            let merged = merge_parts(&parts_for_child);
            result.push(merged);
        }
        
        part_index = end_index;
    }
    
    // The remaining part should be exactly one (for the leader to keep)
    if part_index < parts.len() {
        assert_eq!(parts.len() - part_index, 1, "Expected exactly one remaining part for the leader");
        result.push(parts[part_index].clone());
    }
    
    result
}

fn send_problem_parts_to_children(node: &Node, parts: &mut Vec<PartOfAProblem>) {
    let friends = node.friends.lock().unwrap();
    
    // Collect children addresses (only those waiting for problem parts)
    let children_addresses: Vec<String> = friends
        .iter()
        .filter_map(|f| match &f.friend_type {
            FriendType::Child { state: FriendTypeChildState::WaitingForProblemParts, .. } => Some(f.address.clone()),
            _ => None,
        })
        .collect();
    
    drop(friends); // Release the lock
    
    if children_addresses.is_empty() || parts.is_empty() {
        return;
    }
    
    let mut handles = Vec::new();
    
    for (i, child_address) in children_addresses.iter().enumerate() {
        if i >= parts.len() {
            break;
        }
        
        let part = parts[i].clone();
        let node_clone = node.clone();
        let child_address_clone = child_address.clone();
        
        let handle = std::thread::spawn(move || {
            let message = Message::new(
                node_clone.address.clone(),
                child_address_clone.clone(),
                MessageType::SolveProblem {
                    start: part.start.clone(),
                    end: part.end.clone(),
                    alphabet: part.alphabet.clone(),
                    hash: part.hash.clone(),
                },
            );
            let res = send_message(&message, &node_clone);
            if res.is_none() {
                eprintln!("Failed to send ProblemPart message to child: {}", child_address_clone);
                node_clone.remove_friend(&child_address_clone);
                return false;
            }
            true
        });
        handles.push((i, child_address.clone(), handle));
    }
    
    // Wait for all threads to complete and update part states
    for (i, child_address, handle) in handles {
        if let Ok(success) = handle.join() {
            if success {
                parts[i].state = PartOfAProblemState::Distributed;
                // Update friend state to Solving with the assigned part
                node.set_friend_child_state_solving(&child_address, parts[i].clone());
            }
        }
    }
}