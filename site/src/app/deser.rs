use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerNode {
    pub id: u32,
    pub uid: String,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerGraph {
    pub nodes: Vec<SerNode>,
    pub edges: Vec<(u32, u32)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerCol {
    pub id: u32,
    pub c: [u8; 3],
}
