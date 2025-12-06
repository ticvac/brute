use std::sync::{Arc, Mutex};
use super::friend::Friend;

#[derive(Debug, Clone)]
pub enum ChildState {
    Connected,
}

#[derive(Debug, Clone)]
pub enum LeaderState {
    WaitingForProblem
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
}

impl Node {
    pub fn new(address: String, friends: Vec<Friend>, power: u32) -> Self {
        Node {
            address,
            friends: Arc::new(Mutex::new(friends)),
            state: Arc::new(Mutex::new(NodeState::Idle)),
            communicating: Arc::new(Mutex::new(true)),
            power,
        }
    }

    pub fn print_info(&self) {
        let mut output = String::new();
        output.push_str("=== Node Information ===\n");
        output.push_str(&format!("Address: {}\n", self.address));
        output.push_str(&format!("State: {:?}\n", self.state.lock().unwrap()));
        output.push_str(&format!("Communicating: {:?}\n", self.communicating.lock().unwrap()));
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

    pub fn transition_to_leader(&self) {
        let mut state = self.state.lock().unwrap();
        *state = NodeState::Leader {
            state: LeaderState::WaitingForProblem,
        };
    }

    pub fn transition_to_child(&self, leader_address: String) {
        let mut state = self.state.lock().unwrap();
        *state = NodeState::Child {
            leader_address,
            state: ChildState::Connected,
        };
    }

}