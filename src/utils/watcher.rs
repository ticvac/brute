use std::thread;
use std::time::Duration;

use crate::messages::{Message, MessageType};
use crate::messages::send_message::send_message;
use crate::problem::{update_state_of_parts, PartOfAProblemState};
use crate::utils::friend::{FriendType, FriendTypeChildState};
use crate::utils::node::{LeaderState, Node, NodeState};

/// Starts a watcher thread that monitors children in Solving state.
/// Pings them every 5 seconds. If a child doesn't respond, removes it
/// and marks its part as NotDistributed.
/// The watcher stops when the leader exits the Solving state.
pub fn start_watcher(node: Node) {
    thread::spawn(move || {
        println!("[Watcher] Started monitoring children.");
        
        loop {
            thread::sleep(Duration::from_secs(5));
            
            // Check if leader is still solving, if not, stop the watcher
            if !node.is_leader_solving() {
                println!("[Watcher] Leader is no longer solving. Stopping watcher.");
                break;
            }
            
            // Get children in Solving state
            let solving_children = get_solving_children(&node);
            
            if solving_children.is_empty() {
                continue;
            }
            
            println!("[Watcher] Checking {} solving children...", solving_children.len());
            
            for child_address in solving_children {
                let ping_message = Message::new(
                    node.address.clone(),
                    child_address.clone(),
                    MessageType::Ping,
                );
                
                let response = send_message(&ping_message, &node);
                
                if response.is_none() {
                    println!("[Watcher] Child {} is unresponsive. Removing and reclaiming part.", child_address);
                    mark_child_part_as_not_distributed(&node, &child_address);
                    node.remove_friend(&child_address);
                }
            }
        }
        
        println!("[Watcher] Stopped.");
    });
}

/// Returns addresses of all children currently in Solving state.
fn get_solving_children(node: &Node) -> Vec<String> {
    let friends = node.friends.lock().unwrap();
    friends
        .iter()
        .filter_map(|f| {
            if let FriendType::Child { state: FriendTypeChildState::Solving { .. }, .. } = &f.friend_type {
                Some(f.address.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Finds the part assigned to a child and marks it as NotDistributed.
fn mark_child_part_as_not_distributed(node: &Node, friend_address: &str) {
    // First get the part from the friend
    let mut friends = node.friends.lock().unwrap();
    let friend = friends.iter_mut().find(|f| f.address == friend_address);
    
    let part = if let Some(friend) = friend {
        friend.get_solving_part_and_transition_to_waiting()
    } else {
        None
    };
    
    drop(friends); // Release friends lock before acquiring state lock
    
    if let Some(mut part) = part {
        // Mark the part as NotDistributed in the leader's parts
        part.state = PartOfAProblemState::NotDistributed;
        
        let mut state = node.state.lock().unwrap();
        if let NodeState::Leader { state: LeaderState::Solving { parts, .. } } = &mut *state {
            update_state_of_parts(parts, &part);
            println!("[Watcher] Marked part [{} - {}] as NotDistributed", part.start, part.end);
        }
    }
}
