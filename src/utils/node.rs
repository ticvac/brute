use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::problem::{PartOfAProblem, Problem};
use crate::utils::backup_watcher::start_backup_watcher;
use crate::utils::leader_snapshot::LeaderSnapshot;
use serde::{Serialize, Deserialize};
use crate::utils::backups::send_backup_data;
use crate::utils::watcher::start_watcher;
use crate::messages::{Message, MessageType};
use crate::messages::send_message::send_message;
use crate::problem::{PartOfAProblemState, update_state_of_parts};

use super::friend::{FriendType, Friend, FriendTypeChildState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChildState {
    Connected,
    Solving {
        part: PartOfAProblem,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeaderState {
    WaitingForProblem,
    Solving {
        parts: Vec<PartOfAProblem>,
    }
}

#[derive(Debug, Clone)]
pub enum NodeState {
    Idle,
    Child {
        leader_address: String,
        state: ChildState,
        backup_snapshot: Arc<Mutex<Option<LeaderSnapshot>>>,
    },
    Leader {
        state: LeaderState,
        has_backup: Arc<Mutex<AtomicBool>>,
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

    pub fn is_child(&self) -> bool {
        matches!(*self.state.lock().unwrap(), NodeState::Child { .. })
    }

    pub fn is_leader_waiting_for_problem(&self) -> bool {
        match &*self.state.lock().unwrap() {
            NodeState::Leader { state, .. } => matches!(state, LeaderState::WaitingForProblem),
            _ => false,
        }
    }

    pub fn is_leader_solving(&self) -> bool {
        match &*self.state.lock().unwrap() {
            NodeState::Leader { state, .. } => matches!(state, LeaderState::Solving { .. }),
            _ => false,
        }
    }

    pub fn transition_to_leader(&self) {
        let mut state = self.state.lock().unwrap();
        *state = NodeState::Leader {
            state: LeaderState::WaitingForProblem,
            has_backup: Arc::new(Mutex::new(AtomicBool::new(false))),
        };
    }

    pub fn transition_leader_to_waiting(&self) {
        let mut state = self.state.lock().unwrap();
        if let NodeState::Leader { state: leader_state, .. } = &mut *state {
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
            backup_snapshot: Arc::new(Mutex::new(None)),
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
                parts: vec![problem_part],
            },
            has_backup: Arc::new(Mutex::new(AtomicBool::new(false))),
        };
    }

    pub fn set_problem_parts(&self, parts: Vec<PartOfAProblem>) {
        let mut state = self.state.lock().unwrap();
        if let NodeState::Leader { state: leader_state, .. } = &mut *state {
            match leader_state {
                LeaderState::Solving { parts: leader_parts } => {
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

    pub fn transition_to_child_solving(&self, part: PartOfAProblem) {
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
                if let NodeState::Leader { state: LeaderState::Solving { parts, .. }, .. } = &mut *state {
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

    pub fn get_leader_state(&self) -> LeaderState {
        let state = self.state.lock().unwrap();
        if let NodeState::Leader { state: leader_state, .. } = &*state {
            leader_state.clone()
        } else {
            eprintln!("! Cannot get leader state: Node is not a Leader.");
            LeaderState::WaitingForProblem
        }
    }

    pub fn get_children_friends(&self) -> Vec<Friend> {
        let friends = self.friends.lock().unwrap();
        friends.iter()
            .filter(|f| matches!(f.friend_type, FriendType::Child { .. }))
            .cloned()
            .collect()
    }

    pub fn has_backup(&self) -> bool {
        let state = self.state.lock().unwrap();
        if let NodeState::Leader { has_backup, .. } = &*state {
            has_backup.lock().unwrap().load(Ordering::SeqCst)
        } else {
            false
        }
    }

    pub fn set_has_backup(&self, value: bool) {
        let state = self.state.lock().unwrap();
        if let NodeState::Leader { has_backup, .. } = &*state {
            has_backup.lock().unwrap().store(value, Ordering::SeqCst);
        }
    }

    pub fn set_backup_address(&self, address: String) {
        let mut friends = self.friends.lock().unwrap();
        for friend in friends.iter_mut() {
            if friend.address == address {
                friend.is_backup = true;
            } else {
                friend.is_backup = false;
            }
        }
    }

    pub fn get_backup_address(&self) -> Option<String> {
        let friends = self.friends.lock().unwrap();
        for friend in friends.iter() {
            if friend.is_backup {
                return Some(friend.address.clone());
            }
        }
        None
    }

    pub fn update_backup_snapshot(&self, snapshot: LeaderSnapshot) {
        let state = self.state.lock().unwrap();
        if let NodeState::Child { backup_snapshot, .. } = &*state {
            let mut backup = backup_snapshot.lock().unwrap();
            // Only update if new snapshot timestamp is greater than the existing one
            let should_update = match &*backup {
                Some(existing) => snapshot.timestamp > existing.timestamp,
                None => {
                    // setting new backup snapshot
                    println!("[Node] Setting new backup snapshot.");
                    start_backup_watcher(self.clone().into());
                    true
                },
            };
            if should_update {
                *backup = Some(snapshot);
            }
        } else {
            eprintln!("! Cannot update backup snapshot: Node is not a Child.");
        }
    }

    pub fn has_backup_snapshot(&self) -> bool {
        let state = self.state.lock().unwrap();
        if let NodeState::Child { backup_snapshot, .. } = &*state {
            backup_snapshot.lock().unwrap().is_some()
        } else {
            false
        }
    }

    pub fn get_backup_snapshot(&self) -> Option<LeaderSnapshot> {
        let state = self.state.lock().unwrap();
        if let NodeState::Child { backup_snapshot, .. } = &*state {
            backup_snapshot.lock().unwrap().clone()
        } else {
            None
        }
    }

    pub fn promote_to_leader_from_backup(&self) {
        let snapshot = match self.get_backup_snapshot() {
            Some(s) => s,
            None => {
                eprintln!("! Cannot promote to leader: No backup snapshot available.");
                return;
            }
        };

        println!("[Node] Promoting to leader from backup snapshot...");

        // Stop any solving work we were doing as a child
        self.set_stop_flag(true);

        // Find our own part from snapshot (the part we were solving as backup) and mark it as NotDistributed
        let my_part_to_reset: Option<PartOfAProblem> = snapshot.children.iter()
            .find(|c| c.address == self.address)
            .and_then(|my_entry| {
                if let FriendType::Child { state: FriendTypeChildState::Solving { part }, .. } = &my_entry.friend_type {
                    let mut p = part.clone();
                    p.state = PartOfAProblemState::NotDistributed;
                    Some(p)
                } else {
                    None
                }
            });

        // Transition to leader with the snapshot's leader state
        {
            let mut state = self.state.lock().unwrap();
            *state = NodeState::Leader {
                state: snapshot.leader_state.clone(),
                has_backup: Arc::new(Mutex::new(AtomicBool::new(false))),
            };

            // Mark our own part as NotDistributed in leader state
            if let Some(ref part) = my_part_to_reset {
                if let NodeState::Leader { state: LeaderState::Solving { parts, .. }, .. } = &mut *state {
                    update_state_of_parts(parts, part);
                    println!("[Node] Marked my own part [{} - {}] as NotDistributed", part.start, part.end);
                }
            }
        }

        // Update friends list from snapshot (exclude ourselves, clear backup flags)
        {
            let mut friends = self.friends.lock().unwrap();
            for snapshot_child in &snapshot.children {
                // Skip ourselves - we can't be our own friend
                if snapshot_child.address == self.address {
                    continue;
                }
                if let Some(friend) = friends.iter_mut().find(|f| f.address == snapshot_child.address) {
                    friend.friend_type = snapshot_child.friend_type.clone();
                    friend.is_backup = false;
                } else {
                    let mut new_friend = snapshot_child.clone();
                    new_friend.is_backup = false;
                    friends.push(new_friend);
                }
            }
        }

        // Notify all children about new leader in parallel
        let child_addresses = self.get_child_addresses();
        println!("[Node] Notifying {} children about new leader...", child_addresses.len());

        let handles: Vec<_> = child_addresses.iter().map(|child_addr| {
            let node_address = self.address.clone();
            let node_clone = self.clone();
            let child_addr = child_addr.clone();

            std::thread::spawn(move || {
                let message = Message::new(node_address, child_addr.clone(), MessageType::IAmANewLeader);
                match send_message(&message, &node_clone) {
                    Some(response) if matches!(response.message_type, MessageType::Ack) => {
                        println!("[Node] Child {} acknowledged new leader.", child_addr);
                        Some(child_addr)
                    }
                    Some(response) => {
                        println!("[Node] Child {} sent unexpected response: {:?}", child_addr, response.message_type);
                        None
                    }
                    None => {
                        println!("[Node] Child {} did not respond.", child_addr);
                        None
                    }
                }
            })
        }).collect();

        // Collect responding children
        let responding_children: Vec<String> = handles.into_iter()
            .filter_map(|h| h.join().ok().flatten())
            .collect();

        // Remove non-responding children
        for child_addr in &child_addresses {
            if !responding_children.contains(child_addr) {
                println!("[Node] Removing non-responding child: {}", child_addr);
                self.remove_friend(child_addr);
            }
        }

        // Select first responding child as backup
        if let Some(backup_addr) = responding_children.first() {
            self.set_has_backup(true);
            self.set_backup_address(backup_addr.clone());
            println!("[Node] Selected {} as new backup.", backup_addr);
            send_backup_data(self);
        } else {
            println!("[Node] No children responded to select as backup.");
        }

        // Start watcher to monitor children
        if self.is_leader_solving() {
            println!("[Node] Starting watcher to monitor children...");
            start_watcher(self.clone());
        }

        println!("[Node] Successfully promoted to leader from backup.");
    }

    pub fn set_leader_address(&self, leader_address: String) {
        let mut state = self.state.lock().unwrap();
        if let NodeState::Child { leader_address: addr, .. } = &mut *state {
            *addr = leader_address;
        } else {
            eprintln!("! Cannot set leader address: Node is not a Child.");
        }
    }
}