use crate::utils::leader_snapshot::LeaderSnapshot;
use crate::utils::node::Node;
use crate::messages::{Message, MessageType};

pub fn handle_received_backup_data(node: &Node, message: &Message) {
    if !node.is_child() {
        eprintln!("! Received BackupData but node is not a child. Ignoring.");
        return;
    }
    let data = match message.message_type {
        MessageType::BackupData { ref data } => data.clone(),
        _ => {
            eprintln!("! Invalid message type for BackupData handler.");
            return;
        }
    };
    let snapshot = LeaderSnapshot::deserialize(&data);

    if let Some(leader_snapshot) = snapshot {
        node.update_backup_snapshot(leader_snapshot);
    } else {
        eprintln!("! Failed to deserialize LeaderSnapshot from BackupData.");
    }
}