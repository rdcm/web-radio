use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioChunk {
    pub freq: u64,
    pub pcm: Vec<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSubscribe {
    pub freq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectrumChunk {
    pub center_freq: u64,
    pub sample_rate: u32,
    pub bins: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectrumSubscribe {}
