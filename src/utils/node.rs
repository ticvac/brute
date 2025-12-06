use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::problem::{PartOfAProblem, Problem};

use super::friend::{FriendType, Friend};

#[derive(Debug, Clone)]
pub enum ChildState {
    Connected,
    Solving {
        part: Problem,
    }
}

#[derive(Debug, Clone)]
pub enum LeaderState {
    WaitingForProblem,
    Solving {
        problem: Problem,
        parts: Vec<PartOfAProblem>,
    }
}

#[derive(Debug, Clone)]
pub enum NodeState {
    Idle,
    Child {
        leader_address: String,
        state: ChildState,
    },
    Leader {
        state: LeaderState,
    },
}

#[derive(Debug, Clone)]
pub struct Node {
    pub address: String,
    pub friends: Arc<Mutex<Vec<Friend>>>,
    pub state: Arc<Mutex<NodeState>>,
    pub communicating: Arc<Mutex<bool>>,
    // k hashes per second
    pub power: u32,
    pub stop_flag: Arc<AtomicBool>,
}

impl Node {
    pub fn new(address: String, friends: Vec<Friend>, power: u32) -> Self {
        Node {
            address,
            friends: Arc::new(Mutex::new(friends)),
            state: Arc::new(Mutex::new(NodeState::Idle)),
            communicating: Arc::new(Mutex::new(true)),
            power,
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn print_info(&self) {
        let mut output = String::new();
        output.push_str("=== Node Information ===\n");
        output.push_str(&format!("Address: {}\n", self.address));
        output.push_str(&format!("State: {:?}\n", self.state.lock().unwrap()));
        output.push_str(&format!("Communicating: {:?}\n", self.communicating.lock().unwrap()));
        output.push_str(&format!("Power: {} k hashes/sec\n", self.power));
        output.push_str("Friends:\n");
        let friends = self.friends.lock().unwrap();
        for friend in friends.iter() {
            output.push_str(&format!(" - {:?}\n", friend));
        }
        output.push_str("========================\n");
        println!("{}", output);
    }

    pub fn is_communicating(&self) -> bool {
        *self.communicating.lock().unwrap()
    }

    pub fn remove_friend(&self, address: &str) {
        let mut friends = self.friends.lock().unwrap();
        friends.retain(|f| f.address != address);
    }

    pub fn is_friend(&self, address: &str) -> bool {
        let friends = self.friends.lock().unwrap();
        friends.iter().any(|f| f.address == address)
    }

    pub fn add_friend(&self, friend_address: String) {
        let mut friends = self.friends.lock().unwrap();
        if !friends.iter().any(|f| f.address == friend_address) {
            friends.push(Friend::new(friend_address));
        } else {
            println!("Friend with address {} already exists.", friend_address);
        }
    }

    pub fn is_idle(&self) -> bool {
        matches!(*self.state.lock().unwrap(), NodeState::Idle)
    }

    pub fn is_leader(&self) -> bool {
        matches!(*self.state.lock().unwrap(), NodeState::Leader { .. })
    }

    pub fn is_leader_waiting_for_problem(&self) -> bool {
        match &*self.state.lock().unwrap() {
            NodeState::Leader { state } => matches!(state, LeaderState::WaitingForProblem),
            _ => false,
        }
    }

    pub fn transition_to_leader(&self) {
        let mut state = self.state.lock().unwrap();
        *state = NodeState::Leader {
            state: LeaderState::WaitingForProblem,
        };
    }

    pub fn transition_leader_to_waiting(&self) {
        let mut state = self.state.lock().unwrap();
        if let NodeState::Leader { state: leader_state } = &mut *state {
            *leader_state = LeaderState::WaitingForProblem;
        } else {
            eprintln!("! Cannot transition to leader waiting: Node is not a Leader.");
        }
    }

    pub fn transition_to_child(&self, leader_address: String) {
        let mut state = self.state.lock().unwrap();
        let leader_address_clone = leader_address.clone();
        *state = NodeState::Child {
            leader_address,
            state: ChildState::Connected,
        };
        let mut friends = self.friends.lock().unwrap();
        
        if let Some(friend) = friends.iter_mut().find(|f| f.address == leader_address_clone) {
            friend.set_as_leader();
        }

    }

    pub fn transition_friend_to_child(&self, friend_address: String, power: u32) {
        let mut friends = self.friends.lock().unwrap();
        if let Some(friend) = friends.iter_mut().find(|f| f.address == friend_address) {
            friend.transition_to_child(power);
        } else {
            eprintln!("! Friend with address {} not found for transition to child.", friend_address);
        }
    }

    pub fn transition_to_idle(&self) {
        let mut state = self.state.lock().unwrap();
        *state = NodeState::Idle;
        let mut friends = self.friends.lock().unwrap();
        for friend in friends.iter_mut() {
            friend.transition_to_sibling();
        }
    }

    pub fn set_problem(&self, problem: Problem) {
        let problem_part = PartOfAProblem::new_from_problem(&problem, problem.start.clone(), problem.end.clone());
        let mut state = self.state.lock().unwrap();
        *state = NodeState::Leader {
            state: LeaderState::Solving {
                problem,
                parts: vec![problem_part],
            },
        };
    }

    pub fn set_problem_parts(&self, parts: Vec<PartOfAProblem>) {
        let mut state = self.state.lock().unwrap();
        if let NodeState::Leader { state: leader_state } = &mut *state {
            match leader_state {
                LeaderState::Solving { problem: _, parts: leader_parts } => {
                    *leader_parts = parts;
                },
                _ => {
                    eprintln!("! Cannot set problem parts: Node is not in Solving state.");
                }
            }
        } else {
            eprintln!("! Cannot set problem parts: Node is not a Leader.");
        }
    }

    pub fn get_total_power_of_friends(&self) -> u32 {
        let friends = self.friends.lock().unwrap();
        friends.iter().map(|f| {
            match f.friend_type {
                FriendType::Sibling => 0,
                FriendType::Child { power, ..} => power,
                FriendType::Leader => 0,
            }
        }).sum()
    }

    pub fn get_child_addresses(&self) -> Vec<String> {
        let friends = self.friends.lock().unwrap();
        friends.iter()
            .filter(|f| matches!(f.friend_type, FriendType::Child { .. }))
            .map(|f| f.address.clone())
            .collect()
    }

    pub fn set_all_children_to_waiting(&self) {
        use super::friend::FriendTypeChildState;
        let mut friends = self.friends.lock().unwrap();
        for friend in friends.iter_mut() {
            if let FriendType::Child { state, .. } = &mut friend.friend_type {
                *state = FriendTypeChildState::WaitingForProblemParts;
            }
        }
    }

    pub fn set_friend_child_state_solving(&self, friend_address: &str, part: PartOfAProblem) {
        let mut friends = self.friends.lock().unwrap();
        if let Some(friend) = friends.iter_mut().find(|f| f.address == friend_address) {
            friend.set_child_state_solving(part);
        } else {
            eprintln!("! Friend with address {} not found for setting solving state.", friend_address);
        }
    }

    pub fn is_child_connected(&self) -> bool {
        match &*self.state.lock().unwrap() {
            NodeState::Child { state, .. } => matches!(state, ChildState::Connected),
            _ => false,
        }
    }

    pub fn is_child_solving(&self) -> bool {
        match &*self.state.lock().unwrap() {
            NodeState::Child { state, .. } => matches!(state, ChildState::Solving { .. }),
            _ => false,
        }
    }

    pub fn transition_child_to_connected(&self) {
        let mut state = self.state.lock().unwrap();
        if let NodeState::Child { state: child_state, .. } = &mut *state {
            *child_state = ChildState::Connected;
        } else {
            eprintln!("! Cannot transition to child connected: Node is not a Child.");
        }
    }

    pub fn set_stop_flag(&self, value: bool) {
        self.stop_flag.store(value, Ordering::SeqCst);
    }

    pub fn get_stop_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stop_flag)
    }

    pub fn transition_to_child_solving(&self, part: Problem) {
        let mut state = self.state.lock().unwrap();
        if let NodeState::Child { state: child_state, .. } = &mut *state {
            *child_state = ChildState::Solving { part };
        } else {
            eprintln!("! Cannot transition to child solving: Node is not a Child.");
        }
    }

    pub fn get_leader_address(&self) -> String {
        match &*self.state.lock().unwrap() {
            NodeState::Child { leader_address, .. } => leader_address.clone(),
            _ => {
                // should not really happen...
                eprintln!("! Cannot get leader address: Node is not a Child.");
                String::new()
            }
        }
    }

    pub fn handle_solution_not_found_from_friend(&self, friend_address: &str) {
        use crate::problem::{PartOfAProblemState, update_state_of_parts};
        
        let mut friends = self.friends.lock().unwrap();
        let friend = friends.iter_mut().find(|f| f.address == friend_address);
        
        if let Some(friend) = friend {
            if let Some(mut part) = friend.get_solving_part_and_transition_to_waiting() {
                // Mark the part as searched and not found
                part.state = PartOfAProblemState::SearchedAndNotFound;
                
                // Update the problem parts in the leader state
                drop(friends); // Release the lock before acquiring state lock
                let mut state = self.state.lock().unwrap();
                if let NodeState::Leader { state: LeaderState::Solving { parts, .. } } = &mut *state {
                    update_state_of_parts(parts, &part);
                    println!("Marked part [{} - {}] as searched and not found from friend {}", part.start, part.end, friend_address);
                } else {
                    eprintln!("! Cannot update parts: Node is not a Leader in Solving state.");
                }
            } else {
                eprintln!("! Friend {} was not in Solving state.", friend_address);
            }
        } else {
            eprintln!("! Friend {} not found.", friend_address);
        }
    }
}