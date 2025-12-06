
#[derive(Debug, Clone)]
pub enum FriendType {
    Sibling,
    Child {
        power: u32,
    }
}

#[derive(Debug, Clone)]
pub struct Friend {
    pub address: String,
    pub friend_type: FriendType,
}

impl Friend {
    pub fn new(address: String) -> Self {
        Friend {
            address,
            friend_type: FriendType::Sibling,
        }
    }
}