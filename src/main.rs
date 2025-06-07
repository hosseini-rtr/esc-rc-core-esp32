use crate::common_config::{Config, SERVER_URL};
use std::{env, fs, path::Path};

mod common_config;

// TODO: Run this as a server that can be used to send firmware to the ESP32
// TODO: Run Server as a service

use esp_idf_svc::{
    ws::client::{EspWebSocketClient, WebSocketConfig},
};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <firmware_path>", args[0]);
        return;
    }
    let firmware_path = Path::new(&args[1]);
    let firmware = fs::read(firmware_path).expect("Failed to read firmware file");
    let mut client = EspWebSocketClient::new(SERVER_URL).expect("Failed to connect to server");
    client
        .write_all(&firmware)
        .expect("Failed to send firmware");
    println!("Firmware sent successfully");
}
