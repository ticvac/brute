pub mod send_message;


#[derive(Debug, Clone)]
pub enum MessageType {
    Ping,
    Ack,
    CalculatePower,
    CalculatePowerResult { power: u32 },
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
            MessageType::CalculatePower => format!("CALCULATE_POWER|{}|{}", self.from, self.to),
            MessageType::CalculatePowerResult { power } => {
                format!("CALCULATE_POWER_RESULT|{}|{}|{}", self.from, self.to, power)
            }
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
        "CALCULATE_POWER" => MessageType::CalculatePower,
        "CALCULATE_POWER_RESULT" => {
            if parts.len() != 4 {
                return None;
            }
            let power = parts[3].parse::<u32>().ok()?;
            MessageType::CalculatePowerResult { power }
        }
        _ => return None,
    };

    Some(Message::new(from, to, message_type))
}