use serde::{Deserialize, Serialize};
use bincode;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    SensorData {
        rotation_x: f32,
        rotation_y: f32,
        rotation_z: f32,
    },
    SwitchDisplay {
        direction: SwitchDirection,
    },
    Heartbeat,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SwitchDirection {
    Next,
    Previous,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    VideoFrame {
        display_index: u8,
        width: u32,
        height: u32,
        data: Vec<u8>, // JPEG encoded
        timestamp: u64,
    },
    DisplayConfig {
        total_displays: u8,
        current_display: u8,
        resolutions: Vec<(u32, u32)>,
    },
    Error {
        message: String,
    },
}

pub fn serialize_message<T: Serialize>(msg: &T) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(msg)
}

pub fn deserialize_message<T: for<'a> Deserialize<'a>>(data: &[u8]) -> Result<T, bincode::Error> {
    bincode::deserialize(data)
}