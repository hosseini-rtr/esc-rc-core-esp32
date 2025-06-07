use anyhow::Result;
use esp_idf_hal::gpio::*;
use esp_idf_hal::prelude::*;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
    ws::client::{EspWebSocketClient, WebSocketConfig},
};
use esp_idf_sys::EspError;
use log::{error, info};
use std::time::Duration;
use url::Url;

mod motor;
mod protocol;
use motor::MotorController;
use protocol::{Direction, MotorCommand, RcCommand, RcResponse};

const SSID: &str = "YourWiFiSSID";
const PASSWORD: &str = "YourWiFiPassword";
pub const SERVER_URL: &str = "ws://192.168.1.100:8080";

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("RC Car Firmware Starting...");

    let sysloop = EspSystemEventLoop::take()?;

    let peripherals = Peripherals::take().unwrap();
    let mut wifi = setup_wifi(peripherals.modem, sysloop)?;

    let mut motor_controller = MotorController::new(
        peripherals.pins.gpio32,
        peripherals.pins.gpio33,
        peripherals.pins.gpio25,
        peripherals.pins.gpio26,
    )?;

    connect_wifi(&mut wifi)?;

    loop {
        match connect_and_handle_websocket(&mut motor_controller) {
            Ok(_) => info!("WebSocket connection closed, reconnecting..."),
            Err(e) => error!("WebSocket error: {}, reconnecting...", e),
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn setup_wifi(
    modem: impl Peripheral<P = impl esp_idf_hal::modem::Modem>,
    sysloop: EspSystemEventLoop,
) -> Result<BlockingWifi<EspWifi<'static>>> {
    let wifi = BlockingWifi::wrap(EspWifi::new(modem, sysloop.clone(), None)?, sysloop)?;

    Ok(wifi)
}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> Result<()> {
    let wifi_configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.into(),
        password: PASSWORD.into(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;
    wifi.start()?;
    info!("WiFi started");

    wifi.connect()?;
    info!("WiFi connected");

    wifi.wait_netif_up()?;
    info!("WiFi ready");

    Ok(())
}

fn connect_and_handle_websocket(motor_controller: &mut MotorController) -> Result<()> {
    let url = Url::parse(SERVER_URL)?;

    let config = WebSocketConfig::default();
    let mut client = EspWebSocketClient::new(&url, &config)?;

    info!("WebSocket connected to {}", SERVER_URL);

    let mut buffer = [0u8; 1024];
    loop {
        match client.read(&mut buffer) {
            Ok(len) if len > 0 => {
                let message = std::str::from_utf8(&buffer[..len])?;
                match serde_json::from_str::<RcCommand>(message) {
                    Ok(command) => match command {
                        RcCommand::Move(motor_cmd) => {
                            motor_controller.set_direction(motor_cmd.direction, motor_cmd.speed);
                            send_status(&mut client, motor_controller)?;
                        }
                        RcCommand::Stop => {
                            motor_controller.set_direction(Direction::Stop, 0);
                            send_status(&mut client, motor_controller)?;
                        }
                        RcCommand::Ping => {
                            let response = RcResponse::Pong;
                            send_response(&mut client, &response)?;
                        }
                    },
                    Err(e) => {
                        error!("Failed to parse command: {}", e);
                        let response = RcResponse::Error(format!("Invalid command: {}", e));
                        send_response(&mut client, &response)?;
                    }
                }
            }
            Ok(_) => continue,
            Err(e) => {
                error!("WebSocket read error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn send_status(client: &mut EspWebSocketClient, motor_controller: &MotorController) -> Result<()> {
    let (direction, speed) = motor_controller.get_status();
    let status = RcResponse::Status {
        battery_level: 100, // TODO: Implement actual battery monitoring
        connected: true,
        current_speed: speed,
        current_direction: direction,
    };
    send_response(client, &status)
}

fn send_response(client: &mut EspWebSocketClient, response: &RcResponse) -> Result<()> {
    let message = serde_json::to_string(response)?;
    client.write_all(message.as_bytes())?;
    Ok(())
}

// TODO: Add panic handler for debugging
#[cfg(not(feature = "qemu"))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("Firmware panic: {}", info);
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
