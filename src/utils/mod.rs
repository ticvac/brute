pub mod friend;
pub mod node;
pub mod watcher;


pub fn parse_address(input: &str) -> String {
    if input.contains(':') {
        input.to_string()
    } else {
        format!("127.0.0.1:{}", input)
    }
}