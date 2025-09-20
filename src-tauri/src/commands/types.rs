use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanResult {
    pub file_count: usize,
    pub total_size: u64,
    pub files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColumnMapping {
    pub original: String,
    pub mapped: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConvertProgress {
    pub current_file: String,
    pub progress: f32,
    pub total_files: usize,
    pub completed_files: usize,
}