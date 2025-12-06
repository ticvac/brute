use crate::{messages::{Message, send_message::send_message}, utils::node::Node};
use std::sync::Arc;
use crate::messages::MessageType;

pub fn start_backup_watcher(node: Arc<Node>) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(5));
            // check if I am a child with backup responsibility
            if node.is_child() && node.has_backup_snapshot() {
                println!("[BackupWatcher] I am a backup node, monitoring leader...");
                let message = Message::new(
                    node.address.clone(),
                    node.get_leader_address(),
                    MessageType::Ping,
                );
                let res = send_message(&message, node.clone().as_ref());
                if res.is_none() {
                    // promote self to leader
                    println!("[BackupWatcher] Leader is unresponsive! Promoting self to leader.");
                    node.remove_friend(&node.get_leader_address());
                    node.promote_to_leader_from_backup();
                    break;
                }
            }
        }
    });
}