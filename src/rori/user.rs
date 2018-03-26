#[derive(Debug, Clone)]
pub struct Device {
    pub name: String,
    pub ring_id: String
}

impl Device {
    pub fn new(ring_id: &String) -> Device {
        Device {
            name: String::new(),
            ring_id: ring_id.clone()
        }
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub devices: Vec<Device>
}

impl User {
    pub fn new() -> User {
        User {
            name: String::new(),
            devices: Vec::new()
        }
    }
}
