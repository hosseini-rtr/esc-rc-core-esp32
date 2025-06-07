pub const SERVER_URL: &str = "ws://localhost:8080";

pub struct Config {
    pub firmware_path: String,
    pub server_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            firmware_path: String::new(),
            server_url: SERVER_URL.to_string(),
        }
    }
}
