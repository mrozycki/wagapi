use std::time::Duration;

use async_trait::async_trait;
use bluez_async::{
    BluetoothEvent, BluetoothSession, CharacteristicEvent, DeviceId, WriteOptions, WriteType,
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use futures::StreamExt;
use tokio::time::timeout;
use uuid::Uuid;

use crate::store::measurement::{Record, Value};

use super::device::Device;

const WEIGHT_CUSTOM_CHARACTERISTIC: Uuid = Uuid::from_u128(0x352e3001_28e9_40b8_a361_6db4cca4147c);
const CMD_CHARACTERISTIC: Uuid = Uuid::from_u128(0x352e3002_28e9_40b8_a361_6db4cca4147c);
const CUSTOM_SERVICE_UUID: Uuid = Uuid::from_u128(0x352e3000_28e9_40b8_a361_6db4cca4147c);
const USER_DATA_SERVICE_UUID: Uuid = Uuid::from_u128(0x0000181C_0000_1000_8000_00805F9B34FB);
const HEIGHT_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x00002A8E_0000_1000_8000_00805F9B34FB);
const AGE_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x00002A80_0000_1000_8000_00805F9B34FB);
const SEX_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x00002A8E_0000_1000_8000_00805F9B34FB);

pub struct Shape200 {
    device_id: DeviceId,
}

impl Shape200 {
    pub fn new(device_id: DeviceId) -> Self {
        Self { device_id }
    }
}

#[async_trait]
impl Device for Shape200 {
    async fn connect(&self, session: &BluetoothSession) -> Result<(), Box<dyn std::error::Error>> {
        session.connect(&self.device_id).await?;
        Ok(())
    }

    async fn disconnect(&self, session: &BluetoothSession) -> Result<(), Box<dyn std::error::Error>> {
        session.disconnect(&self.device_id).await?;
        Ok(())
    }

    async fn get_data(
        &self,
        session: &BluetoothSession,
    ) -> Result<Vec<Record>, Box<dyn std::error::Error>> {
        let weight_service = session
            .get_service_by_uuid(&self.device_id, CUSTOM_SERVICE_UUID)
            .await?;
        let user_data_service = session
            .get_service_by_uuid(&self.device_id, USER_DATA_SERVICE_UUID)
            .await?;
        let weight_characteristic = session
            .get_characteristic_by_uuid(&weight_service.id, WEIGHT_CUSTOM_CHARACTERISTIC)
            .await?;
        let cmd_characteristic = session
            .get_characteristic_by_uuid(&weight_service.id, CMD_CHARACTERISTIC)
            .await?;
        session
            .write_characteristic_value_with_options(
                &cmd_characteristic.id,
                vec![0x09, 1],
                WriteOptions {
                    offset: 0,
                    write_type: Some(WriteType::WithResponse),
                },
            )
            .await?;
            session
            .write_characteristic_value_with_options(
                &cmd_characteristic.id,
                vec![0x0c, 1],
                WriteOptions {
                    offset: 0,
                    write_type: Some(WriteType::WithResponse),
                },
            )
            .await?;

        let mut events = session
            .characteristic_event_stream(&weight_characteristic.id)
            .await?;
        session.start_notify(&weight_characteristic.id).await?;

        let mut records = Vec::new();
        while let Ok(Some(bt_event)) = timeout(Duration::from_millis(1000), events.next()).await {
            if let BluetoothEvent::Characteristic {
                event: CharacteristicEvent::Value { value },
                ..
            } = bt_event
            {
                if value[0] == 9 {
                    let year = (value[2] as i32) * 256 + (value[3] as i32);
                    let date = NaiveDate::from_ymd(year, value[4] as u32, value[5] as u32);
                    let time =
                        NaiveTime::from_hms(value[6] as u32, value[7] as u32, value[8] as u32);

                    let weight = ((value[9] as f64) * 256.0 + (value[10] as f64)) / 10.0;
                    let imp5 = (value[11] as i32) * 256 + (value[12] as i32);
                    let imp50 = (value[13] as i32) * 256 + (value[14] as i32);

                    let age = session.read_characteristic_value(&session.get_characteristic_by_uuid(&user_data_service.id, AGE_CHARACTERISTIC_UUID).await?.id).await?[0];
                    let sex = session.read_characteristic_value(&session.get_characteristic_by_uuid(&user_data_service.id, SEX_CHARACTERISTIC_UUID).await?.id).await?[0];
                    let height = session.read_characteristic_value(&session.get_characteristic_by_uuid(&user_data_service.id, HEIGHT_CHARACTERISTIC_UUID).await?.id).await?[0];
                    // TODO: get_fat, get_water, get_muscle
                    records.push(Record::with_values(
                        NaiveDateTime::new(date, time),
                        vec![Value::Weight(weight)],
                    ))

                } else if value[0] == 12 && value[1] == 1 {
                    // TODO get the value[9] byte and use as activity level
                    // see https://github.com/oliexdev/openScale/blob/master/android_app/app/src/main/java/com/health/openscale/core/bluetooth/lib/SoehnleLib.java
                    println!("{:?}", value);
                }
            }
        }

        Ok(records)
    }
}
