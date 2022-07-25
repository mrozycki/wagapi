mod devices;
mod store;

use bluez_async::BluetoothSession;
use std::error::Error;
use std::time::Duration;
use tokio::time;

use crate::devices::device::Device;
use crate::devices::soehnle::Shape200;

const PERIPHERAL_NAME_MATCH_FILTER: &str = "Shape200";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (_, session) = BluetoothSession::new().await?;

    println!("Listening...");
    loop {
        session.start_discovery().await?;
        time::sleep(Duration::from_millis(1000)).await;
        session.stop_discovery().await?;

        let devices = session.get_devices().await?;

        if let Some(scale) = devices
            .into_iter()
            .find(|device| device.name.as_deref() == Some(PERIPHERAL_NAME_MATCH_FILTER))
            .map(|info| Shape200::new(info.id))
        {
            println!("Connecting to peripheral");
            scale.connect(&session).await?;
            println!("Connected");

            let data = scale.get_data(&session).await?;
            println!("Received data: {:?}", data);
            scale.disconnect(&session).await?;
            return Ok(());
        }
    }
}
