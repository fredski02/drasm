use serde::{Deserialize, Serialize};

// Kafka broker and topic constants
pub const MAX_RETRIES: u32 = 3;
pub const BROKERS: &str = "localhost:9094";
pub const JOBS_TOPIC: &str = "wasm_jobs";
pub const RESULTS_TOPIC: &str = "wasm_results";
pub const DLQ_TOPIC: &str = "wasm_jobs_dlq";

// Redis connection
pub const REDIS_URL: &str = "redis://localhost:6379";
pub const REDIS_TTL_SECONDS: u64 = 86400; // 24 hours

// Job definition
#[derive(Serialize, Deserialize, Clone)]
pub struct Job {
    pub job_id: String,
    pub wat: String,
    pub input: i32,
}

// Result message with worker identification
#[derive(Serialize, Deserialize)]
pub struct ResultMsg {
    pub job_id: String,
    pub worker_id: String,
    pub ok: bool,
    pub output: Option<i32>,
    pub error: Option<String>,
}

// DLQ message (failed job with metadata)
#[derive(Serialize, Deserialize)]
pub struct DLQMessage {
    pub job_id: String,
    pub job: Job,
    pub attempts: u32,
    pub last_error: String,
    pub timestamp: u64,
}