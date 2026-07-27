use serde::{Deserialize, Serialize};

pub trait ToMsg{
    fn to_msg<T: Serialize>(&self, msg: &T) -> anyhow::Result<Vec<u8>>;
}

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
    TouchEvent(TouchEvent),
    Heartbeat,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TouchEvent {
    Move { x: f32, y: f32 },
    Down { x: f32, y: f32 },
    Up { x: f32, y: f32 },
    Scroll { delta_x: f32, delta_y: f32 },
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

impl ToMsg for ServerMessage {
    fn to_msg<T: Serialize>(&self,msg: &T)-> anyhow::Result<Vec<u8>> {
        serialize_message(msg)
    }
}

// 序列化辅助函数
pub fn serialize_message<T: Serialize>(msg: &T) -> anyhow::Result<Vec<u8>> {
    Ok(serde_json::to_vec(msg)?)
}

pub fn deserialize_message<T: for<'a> Deserialize<'a>>(data: &[u8]) -> anyhow::Result<T> {
    Ok(serde_json::from_slice(data)?)
}