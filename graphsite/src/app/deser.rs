use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerProps {
    pub id: usize,
    pub c: [u8; 3],
    pub l: [u8; 3],
    pub x: f32,
    pub y: f32,
}
