use crate::utils::node::{LeaderState};
use crate::utils::friend::{Friend};
use crate::utils::node::Node;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderSnapshot {
    pub timestamp: u128,
    // parts are in leader state...
    pub leader_state: LeaderState,
    // every children should be in state child...
    pub children: Vec<Friend>
}

impl LeaderSnapshot {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn deserialize(data: &str) -> Option<LeaderSnapshot> {
        serde_json::from_str(data).ok()
    }
}

pub fn create_leader_snapshot(node: &Node) -> LeaderSnapshot {
    let snapshot = LeaderSnapshot {
        timestamp: get_current_timestamp(),
        leader_state: node.get_leader_state().clone(),
        children: node.get_children_friends().clone(),
    };
    snapshot
}

pub fn get_current_timestamp() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_nanos()
}