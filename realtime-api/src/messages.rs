use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FmChunk {
    pub freq: u64,
    pub pcm: Vec<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FmSubscribe {
    pub freq: u64,
}
