use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerNode {
    pub id: u32,
    pub name: String,
    pub size: f32,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerEdge {
    pub source: u32,
    pub target: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerGraph {
    pub nodes: Vec<SerNode>,
    pub edges: Vec<SerEdge>,
}
