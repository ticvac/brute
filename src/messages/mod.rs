pub mod send_message;


#[derive(Debug, Clone)]
pub enum MessageType {
    Ping,
    Ack,
    CalculatePower { leader_address: String },
    CalculatePowerResult { power: u32 },
    SolveProblem {
        start: String,
        end: String,
        alphabet: String,
        hash: String,
    },
    SolutionFound {
        solution: String,
    },
    SolutionNotFound,
    StopSolving,
    BackupData {
        data: String,
    }
}


#[derive(Debug, Clone)]
pub struct Message {
    // adresses
    pub from: String,
    pub to: String,
    pub message_type: MessageType,
}

impl Message {
    pub fn new(from: String, to: String, message_type: MessageType) -> Self {
        Message {
            from,
            to,
            message_type,
        }
    }

    pub fn serialize(&self) -> String {
        match &self.message_type {
            MessageType::Ping => format!("PING|{}|{}", self.from, self.to),
            MessageType::Ack => format!("ACK|{}|{}", self.from, self.to),
            MessageType::CalculatePower { leader_address } => format!("CALCULATE_POWER|{}|{}|{}", self.from, self.to, leader_address),
            MessageType::CalculatePowerResult { power } => {
                format!("CALCULATE_POWER_RESULT|{}|{}|{}", self.from, self.to, power)
            },
            MessageType::SolveProblem { start, end, alphabet, hash } => {
                format!("SOLVE_PROBLEM|{}|{}|{}|{}|{}|{}", self.from, self.to, start, end, alphabet, hash)
            },
            MessageType::SolutionFound { solution } => {
                format!("SOLUTION_FOUND|{}|{}|{}", self.from, self.to, solution)
            },
            MessageType::SolutionNotFound => {
                format!("SOLUTION_NOT_FOUND|{}|{}", self.from, self.to)
            },
            MessageType::StopSolving => {
                format!("STOP_SOLVING|{}|{}", self.from, self.to)
            },
            MessageType::BackupData { data } => {
                format!("BACKUP_DATA|{}|{}|{}", self.from, self.to, data)
            },
        }
    }
}

pub fn deserialize(input: &str) -> Option<Message> {
    let parts: Vec<&str> = input.split('|').collect();
    if parts.len() < 3 {
        return None;
    }

    let message_type_str = parts[0];
    let from = parts[1].to_string();
    let to = parts[2].to_string();

    let message_type = match message_type_str {
        "PING" => MessageType::Ping,
        "ACK" => MessageType::Ack,
        "CALCULATE_POWER" => {
            if parts.len() != 4 {
                return None;
            }
            let leader_address = parts[3].to_string();
            MessageType::CalculatePower { leader_address }
        },
        "CALCULATE_POWER_RESULT" => {
            if parts.len() != 4 {
                return None;
            }
            let power = parts[3].parse::<u32>().ok()?;
            MessageType::CalculatePowerResult { power }
        }
        "SOLVE_PROBLEM" => {
            if parts.len() != 7 {
                return None;
            }
            let start = parts[3].to_string();
            let end = parts[4].to_string();
            let alphabet = parts[5].to_string();
            let hash = parts[6].to_string();
            MessageType::SolveProblem { start, end, alphabet, hash }
        },
        "SOLUTION_FOUND" => {
            if parts.len() != 4 {
                return None;
            }
            let solution = parts[3].to_string();
            MessageType::SolutionFound { solution }
        },
        "SOLUTION_NOT_FOUND" => MessageType::SolutionNotFound,
        "STOP_SOLVING" => MessageType::StopSolving,
        "BACKUP_DATA" => {
            if parts.len() != 4 {
                return None;
            }
            let data = parts[3].to_string();
            MessageType::BackupData { data }
        },
        _ => return None,
    };

    Some(Message::new(from, to, message_type))
}