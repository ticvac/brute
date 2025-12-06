use std::sync::{Arc, Mutex};
use super::friend::Friend;

#[derive(Debug)]
pub enum NodeState {
    Idle,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub address: String,
    pub friends: Arc<Mutex<Vec<Friend>>>,
    pub state: Arc<Mutex<NodeState>>,
    pub communicating: Arc<Mutex<bool>>,
}

impl Node {
    pub fn new(address: String, friends: Vec<Friend>) -> Self {
        Node {
            address,
            friends: Arc::new(Mutex::new(friends)),
            state: Arc::new(Mutex::new(NodeState::Idle)),
            communicating: Arc::new(Mutex::new(true)),
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

}