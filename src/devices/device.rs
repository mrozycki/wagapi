use std::error::Error;

use async_trait::async_trait;
use bluez_async::BluetoothSession;

use crate::store::measurement::Record;

#[async_trait]
pub trait Device {
    async fn connect(&self, session: &BluetoothSession) -> Result<(), Box<dyn Error>>;
    async fn disconnect(&self, session: &BluetoothSession) -> Result<(), Box<dyn Error>>;
    async fn get_data(&self, session: &BluetoothSession) -> Result<Vec<Record>, Box<dyn Error>>;
}
