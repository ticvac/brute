pub mod friend;
pub mod node;
pub mod watcher;
pub mod leader_snapshot;
pub mod backups;
pub mod backup_watcher;


pub fn parse_address(input: &str) -> String {
    if input.contains(':') {
        input.to_string()
    } else {
        format!("127.0.0.1:{}", input)
    }
}