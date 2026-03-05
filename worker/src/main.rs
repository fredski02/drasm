mod retry;

use common::*;
use rdkafka::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use redis::AsyncCommands;
use retry::RetryTracker;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use wasmtime::{Engine, Instance, Module, Store};

const GROUP_ID: &str = "wasm-workers";

// Get worker ID (hostname or UUID)
fn get_worker_id() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

fn run_wat(wat_src: &str, input: i32) -> anyhow::Result<i32> {
    let wasm_bytes = wat::parse_str(wat_src)?;
    let engine = Engine::default();
    let module = Module::new(&engine, wasm_bytes)?;
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[])?;

    let run = instance.get_typed_func::<i32, i32>(&mut store, "run")?;

    let out = run.call(&mut store, input)?;
    Ok(out)
}

#[tokio::main]
async fn main() {
    let worker_id = get_worker_id();
    println!("Worker starting with ID: {}", worker_id);

    // Connect to Redis for idempotency tracking
    println!("Connecting to Redis at {}...", REDIS_URL);
    let redis_client = redis::Client::open(REDIS_URL).expect("Failed to create Redis client");
    let mut redis_conn = redis_client
        .get_multiplexed_tokio_connection()
        .await
        .expect("Failed to connect to Redis");

    println!("Connected to Redis successfully");

    // Retry tracker
    let mut retry_tracker = RetryTracker::new();

    // Consumer for jobs
    let jobs_consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", BROKERS)
        .set("group.id", GROUP_ID)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Failed to create jobs consumer");

    jobs_consumer
        .subscribe(&[JOBS_TOPIC])
        .expect("Failed to subscribe to jobs topic");

    // Producer for results and DLQ
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", BROKERS)
        .set("message.timeout.ms", "10000")
        .set("acks", "all")
        .set("enable.idempotence", "true")
        .create()
        .expect("Failed to create producer");

    println!("Worker running. Waiting for jobs...");

    loop {
        let msg = match jobs_consumer.recv().await {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Kafka error: {e}");
                continue;
            }
        };

        let payload = match msg.payload_view::<str>() {
            Some(Ok(s)) => s,
            _ => {
                eprintln!("Bad payload, skipping");
                let _ = jobs_consumer.commit_message(&msg, rdkafka::consumer::CommitMode::Async);
                continue;
            }
        };

        let job: Job = match serde_json::from_str(payload) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("Bad JSON: {e}");
                let _ = jobs_consumer.commit_message(&msg, rdkafka::consumer::CommitMode::Async);
                continue;
            }
        };

        // Check Redis for idempotency (simple EXISTS check)
        let exists: bool = redis_conn.exists(&job.job_id).await.unwrap_or(false);
        if exists {
            println!("Job {} already processed, skipping", job.job_id);
            let _ = jobs_consumer.commit_message(&msg, rdkafka::consumer::CommitMode::Async);
            continue;
        }

        // Process job
        let result = run_wat(&job.wat, job.input);

        match result {
            Ok(output) => {
                // Success: publish result
                let result_msg = ResultMsg {
                    job_id: job.job_id.clone(),
                    worker_id: worker_id.clone(),
                    ok: true,
                    output: Some(output),
                    error: None,
                };

                // Publish result
                if let Err(e) = publish_result(&producer, &result_msg).await {
                    eprintln!("Failed to publish result: {e}");
                    continue; // Don't commit, will retry
                }

                // Mark as processed in Redis with TTL
                let _: () = redis_conn
                    .set_ex(&job.job_id, "completed", REDIS_TTL_SECONDS)
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to set Redis key: {}", e);
                    });

                // Clear retry tracker
                retry_tracker.remove(&job.job_id);

                // Commit offset
                let _ = jobs_consumer.commit_message(&msg, rdkafka::consumer::CommitMode::Async);

                println!("Processed job {} -> {}", job.job_id, output);
            }
            Err(e) => {
                // Failure: check retry count
                let attempts = retry_tracker.increment(&job.job_id);

                eprintln!(
                    "Job {} failed (attempt {}/{}): {}",
                    job.job_id, attempts, MAX_RETRIES, e
                );

                if attempts >= MAX_RETRIES {
                    // Max retries reached: send to DLQ
                    let dlq_msg = DLQMessage {
                        job_id: job.job_id.clone(),
                        job: job.clone(),
                        attempts,
                        last_error: e.to_string(),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    };

                    if let Err(e) = publish_to_dlq(&producer, &dlq_msg).await {
                        eprintln!("Failed to publish to DLQ: {e}");
                        continue; // Don't commit, will retry
                    }

                    // Mark as failed in Redis with TTL
                    let _: () = redis_conn
                        .set_ex(&job.job_id, "failed", REDIS_TTL_SECONDS)
                        .await
                        .unwrap_or_else(|e| {
                            eprintln!("Failed to set Redis key: {}", e);
                        });

                    // Clear retry tracker
                    retry_tracker.remove(&job.job_id);

                    // Commit offset (unblock partition)
                    let _ =
                        jobs_consumer.commit_message(&msg, rdkafka::consumer::CommitMode::Async);

                    println!("Job {} sent to DLQ after {} attempts", job.job_id, attempts);
                } else {
                    // Don't commit: will retry on next poll
                    println!("Job {} will retry (attempt {})", job.job_id, attempts);
                }
            }
        }
    }
}

async fn publish_result(
    producer: &FutureProducer,
    result: &ResultMsg,
) -> Result<(), rdkafka::error::KafkaError> {
    let payload = serde_json::to_string(result).unwrap();
    producer
        .send(
            FutureRecord::to(RESULTS_TOPIC)
                .key(&result.job_id)
                .payload(&payload),
            Duration::from_secs(0),
        )
        .await
        .map(|_| ())
        .map_err(|(e, _)| e)
}

async fn publish_to_dlq(
    producer: &FutureProducer,
    dlq_msg: &DLQMessage,
) -> Result<(), rdkafka::error::KafkaError> {
    let payload = serde_json::to_string(dlq_msg).unwrap();
    producer
        .send(
            FutureRecord::to(DLQ_TOPIC)
                .key(&dlq_msg.job_id)
                .payload(&payload),
            Duration::from_secs(0),
        )
        .await
        .map(|_| ())
        .map_err(|(e, _)| e)
}
