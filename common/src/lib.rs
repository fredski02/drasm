use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Job {
    pub job_id: String,
    pub wat: String,
    pub input: i32,
}
