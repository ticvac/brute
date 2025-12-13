# Brute

Brute is a simple distributed algorithm written in rust, designed to bruteforce SHA-1. Although this implementation focuses purely on SHA-1, module "problem" can be replaced with any problem with ordered elements.
Algorithm implementation is not in any means optimal, this work serves as showcase for how distributed problems can be done in rust.

# How to run



## Requirements
  
You need to have rust installed, commands are there - [rust-lang.org](https://rust-lang.org/tools/install/)

## Run single node

To run program as single node on port 2000

````
cargo run -- -p 2000
````

To run program on port 2000 and make it register friends, one on localhost:2001 and second on address 10.10.0.98:2021... friends are divided by ","

````
cargo run -- -p 2000 -f 10.10.0.98:2021,2002
````

## Run demo

Demo with 6 nodes can be run using bash script "./run_nodes.sh".
In the demo, commands are same as for a single node, with small changes - adding port number before it, to signal which node should do that particular command...
ex. "2000 die"

## Node commands

```` bash
info # prints informations about node

ping 10.10.2.33:2000 # pings other node

cal # node becomes a leader, other nodes connects to him

solve ABCDEFGH 8 8 1c69f56d1e9fb6ad2dba26bdc1610d9e98b7bee2086c330d2a8ae12ddefe2ac6
# only for leader - start solving - alphabet - min number of chars - max - target hash

stop # only for leader - stop current solve (just throws everything)

comm # toggles node communication (like disconnect from network)

die # node kills itself
````

# Node structure

Key component of this algorithm is struct Node in module utils/node.rs.
Its state can be probably best explained by looking at code that takes advantage of rust's powerful enums, to create something like optional data structure.

```` rust
// simplified to display only parts relevant for node state

pub enum LeaderState {
    WaitingForProblem,
    Solving {
        parts: Vec<PartOfAProblem>,
    }
}

pub enum ChildState {
    Connected,
    Solving {
        part: PartOfAProblem,
    }
}

pub enum NodeState {
    Idle,
    Child {
        leader_address: String,
        state: ChildState,
    },
    Leader {
        state: LeaderState,
    },
}

pub struct Node {
    pub state: Arc<Mutex<NodeState>>,
}
````

# Modules description

commands -> everything about handling commands from user

communication -> receiving and responding to other nodes

messages -> structs for messaging and functions to send them

problem -> definition of problem that algorithm solves

utils -> struct Node, Friend and things related to backups (if child / leader dies)

Outside of these modules there are two files - main.rs and args.rs - defining application entry point and arguments it takes.











