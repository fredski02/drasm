use std::collections::HashMap;

pub struct RetryTracker {
    attempts: HashMap<String, u32>,
}

impl RetryTracker {
    pub fn new() -> Self {
        Self {
            attempts: HashMap::new(),
        }
    }
    
    pub fn increment(&mut self, job_id: &str) -> u32 {
        let count = self.attempts.entry(job_id.to_string()).or_insert(0);
        *count += 1;
        *count
    }
    
    pub fn remove(&mut self, job_id: &str) {
        self.attempts.remove(job_id);
    }
}