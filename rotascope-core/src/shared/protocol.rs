use serde::{Deserialize, Serialize};
use crate::Result;

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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
        total_displays: usize,
        current_display: u8,
        resolutions: Vec<(u32, u32)>,
    },
    Heartbeat,
    Error {
        message: String,
    },
}

// 序列化辅助函数
pub fn serialize_message<T: Serialize>(msg: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(msg).map_err(|e|e.to_string())
}

pub fn deserialize_message<T: for<'a> Deserialize<'a>>(data: &[u8]) -> Result<T> {
   serde_json::from_slice(data).map_err(|e|e.to_string())
}