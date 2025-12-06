mod args;
mod utils;
mod communication;
mod commands;
mod messages;
mod problem;

use args::Args;
use utils::parse_address;
use clap::Parser;
use utils::friend::Friend;
use utils::node::Node;
use communication::listen;
use problem::solve_for_one_sec;

use crate::commands::proccess_commands;


fn main() {
    let args = Args::parse();
    let my_address = parse_address(&format!("{}", args.port));

    // loading friends
    let friends: Vec<Friend> = args.friends
        .into_iter()
        .map(|f| {
            let address = parse_address(&f);
            Friend::new(address)
        })
        .collect();

    // calculate node power
    let power = solve_for_one_sec();
    let k_power = power / 1000;
    

    // create node
    let node = Node::new(my_address, friends, k_power as u32);

    // printing node info
    node.print_info();

    // listening for commands
    let node_clone = node.clone();
    std::thread::spawn(move || {
        proccess_commands(&node_clone);
    });

    // upcoming connections
    listen(node);
}
