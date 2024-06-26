use std::error::Error;
use std::time::Duration;

use tokio::pin;
use tokio_stream::StreamExt;

use tinkerforge_async::ip_connection::{async_io::AsyncIpConnection, EnumerateResponse, EnumerationType};

const HOST: &str = "localhost";
const PORT: u16 = 4223;

fn print_enumerate_response(response: &EnumerateResponse) {
    println!("UID:               {}", response.uid);
    println!("Enumeration Type:  {:?}", response.enumeration_type);

    if response.enumeration_type == EnumerationType::Disconnected {
        println!("");
        return;
    }

    println!("Connected UID:     {}", response.connected_uid);
    println!("Position:          {}", response.position);
    println!("Hardware Version:  {}", response.hardware_version);
    println!("Firmware Version:  {}", response.firmware_version);
    println!("Device Identifier: {:?}", response.device_identifier);
    println!("");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut ipcon = AsyncIpConnection::new((HOST, PORT)).await?; // Create IP connection and connect to brickd

    // Enumerate
    let stream = ipcon.enumerate().await?.timeout(Duration::from_secs(2));

    pin!(stream);
    while let Some(Ok(paket)) = stream.next().await {
        print_enumerate_response(&paket);
    }
    Ok(())
}
