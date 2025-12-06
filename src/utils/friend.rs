#[derive(Debug, Clone)]
pub struct Friend {
    pub address: String,
}

impl Friend {
    pub fn new(address: String) -> Self {
        Friend {
            address,
        }
    }
}