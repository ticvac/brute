use crate::problem::PartOfAProblem;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FriendTypeChildState {
    WaitingForProblemParts,
    Solving {
        part: PartOfAProblem,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FriendType {
    Sibling,
    Child {
        power: u32,
        state: FriendTypeChildState,
    },
    Leader
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friend {
    pub address: String,
    pub friend_type: FriendType,
    pub is_backup: bool,
}

impl Friend {
    pub fn new(address: String) -> Self {
        Friend {
            address,
            friend_type: FriendType::Sibling,
            is_backup: false,
        }
    }

    pub fn transition_to_child(&mut self, power: u32) {
        self.friend_type = FriendType::Child { power, state: FriendTypeChildState::WaitingForProblemParts };
    }

    pub fn transition_to_sibling(&mut self) {
        self.friend_type = FriendType::Sibling;
    }

    pub fn set_as_leader(&mut self) {
        self.friend_type = FriendType::Leader;
    }

    pub fn set_child_state_solving(&mut self, part: PartOfAProblem) {
        if let FriendType::Child { power: _, state } = &mut self.friend_type {
            *state = FriendTypeChildState::Solving { part };
        } else {
            eprintln!("! Cannot set solving state: Friend is not a Child.");
        }
    }

    pub fn get_solving_part_and_transition_to_waiting(&mut self) -> Option<PartOfAProblem> {
        if let FriendType::Child { power: _, state } = &mut self.friend_type {
            if let FriendTypeChildState::Solving { part } = state {
                let result = part.clone();
                *state = FriendTypeChildState::WaitingForProblemParts;
                return Some(result);
            }
        }
        None
    }
}