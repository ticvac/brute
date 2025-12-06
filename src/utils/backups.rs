use crate::utils::node::Node;
use crate::utils::leader_snapshot::create_leader_snapshot;
use crate::messages::{Message, MessageType};
use crate::messages::send_message::send_message;

pub fn send_backup_data(node: &Node) {
    // send backup to that child
    if let Some(backup_address) = node.get_backup_address() {
        let snapshot = create_leader_snapshot(node);
        let serialized_snapshot = snapshot.serialize();
        let backup_message = Message::new(
            node.address.clone(),
            backup_address.clone(),
            MessageType::BackupData { data: serialized_snapshot },
        );
        let res = send_message(&backup_message, node);
        if res.is_none() {
            eprintln!("! Failed to send backup data to child {}", backup_message.to);
            eprintln!("! needs new backup...")
        } else {
            println!("[Backup] Sent backup data to child {}", backup_message.to);
        }
    }
}