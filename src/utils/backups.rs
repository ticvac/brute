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
            eprintln!("! Selecting new backup...");
            node.remove_friend(&backup_address);
            select_new_backup(node);
        } else {
            println!("[Backup] Sent backup data to child {}", backup_message.to);
        }
    }
}

/// Select a new backup node from available children
pub fn select_new_backup(node: &Node) {
    // Get first available child as new backup
    if let Some(new_backup_addr) = node.get_child_addresses().first().cloned() {
        node.set_backup_address(new_backup_addr.clone());
        println!("[Backup] Selected new backup: {}", new_backup_addr);
        send_backup_data(node);
    } else {
        node.set_has_backup(false);
        println!("[Backup] No available children to select as new backup.");
    }
}