pub mod handle_ping_command;
mod handle_cal_command;

use crate::utils::node::Node;
use std::io::{self, BufRead};
use handle_ping_command::handle_ping_command;
use handle_cal_command::handle_cal_command;

pub fn proccess_commands(node: &Node) {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l.trim().to_string(),
            Err(_) => continue,
        };
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[0] {
            "die" => {
                println!("Node is shutting down.");
                std::process::exit(0);
            }
            "info" => {
                node.print_info();
            }
            "ping" => {
                handle_ping_command(node, parts);
            }
            "cal" => {
                handle_cal_command(node);
            }
            _ => {
                println!("Unknown command: {}", parts[0]);
            }
        }
    }
}