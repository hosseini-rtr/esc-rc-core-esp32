use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Direction {
    Forward,
    Backward,
    Left,
    Right,
    Stop,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MotorCommand {
    pub direction: Direction,
    pub speed: u8, // 0-100 percentage
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RcCommand {
    Move(MotorCommand),
    Stop,
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RcResponse {
    Status {
        battery_level: u8,
        connected: bool,
        current_speed: u8,
        current_direction: Direction,
    },
    Pong,
    Error(String),
}
