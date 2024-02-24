use tokio_stream::StreamExt;

use tinkerforge_async::error::TinkerforgeError;
use tinkerforge_async::ip_connection::async_io::AsyncIpConnection;
use tinkerforge_async::ip_connection::EnumerationType;
use tinkerforge_async::lcd_128_x_64::{Lcd128X64Bricklet, SetTouchPositionCallbackConfigurationRequest};
use tinkerforge_async::master::MasterBrick;
use tinkerforge_async::DeviceIdentifier;

#[tokio::main]
async fn main() -> Result<(), TinkerforgeError> {
    let mut connection = AsyncIpConnection::new(("localhost", 4223)).await?;
    let mut stream = connection.enumerate().await?;
    while let Some(event) = stream.next().await {
        println!("Enumeration: {event:?}");
        match event.enumeration_type {
            EnumerationType::Available | EnumerationType::Connected => {
                if let Some(device_type) = event.device_identifier.parsed() {
                    match device_type {
                        DeviceIdentifier::MasterBrick => {
                            let uid = event.uid;
                            let connection = connection.clone();
                            tokio::spawn(async move {
                                let mut master = MasterBrick::new(uid, connection);
                                if let Ok(status) = master.get_ethernet_status().await {
                                    println!("Status: {status:?}");
                                }
                                println!("Done");
                            });
                        }
                        DeviceIdentifier::Lcd128X64Bricklet => {
                            let uid = event.uid;
                            let connection = connection.clone();
                            tokio::spawn(async move {
                                let mut bricklet = Lcd128X64Bricklet::new(uid, connection);
                                let config_result = bricklet
                                    .set_touch_position_callback_configuration(SetTouchPositionCallbackConfigurationRequest {
                                        period: 50,
                                        value_has_to_change: true,
                                    })
                                    .await;
                                println!("Initialized: {config_result:?}");
                                let mut stream = bricklet.touch_position_stream().await;
                                while let Some(event) = stream.next().await {
                                    println!("Event: {event:?}");
                                }
                            });
                        }
                        _ => {}
                    }
                }
            }
            EnumerationType::Disconnected => {}
            EnumerationType::Unknown => {}
        }
    }
    Ok(())
}
