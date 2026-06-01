#[derive(Debug, Clone)]
pub struct MinecraftServerTarget {
    pub address: String,
    pub port: u16,
}

impl MinecraftServerTarget {
    pub fn parse(full_address: &str) -> Self {
        let parts: Vec<&str> = full_address.split(':').collect();
        let address = parts[0].to_string();
        let port = parts
            .get(1)
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(25565);
        MinecraftServerTarget { address, port }
    }

    pub fn new(address: &str, port: u16) -> Self {
        MinecraftServerTarget {
            address: address.to_string(),
            port,
        }
    }
}
