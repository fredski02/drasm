use common::Job;
use rdkafka::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use uuid::Uuid;

const BROKERS: &str = "localhost:9092";
const JOBS_TOPIC: &str = "wasm_jobs";

#[tokio::main]
async fn main() {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", BROKERS)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("producer");

    // Simple WAT module: run(x) = x + 1
    let wat = r#"
    (module
      (func (export "run") (param i32) (result i32)
        local.get 0
        i32.const 1
        i32.add
      )
    )
    "#;

    let job = Job {
        job_id: Uuid::new_v4().to_string(),
        wat: wat.to_string(),
        input: 41,
    };

    let payload = serde_json::to_string(&job).unwrap();

    let delivery = producer
        .send(
            FutureRecord::to(JOBS_TOPIC)
                .key(&job.job_id)
                .payload(&payload),
            Duration::from_secs(3),
        )
        .await;

    match delivery {
        Ok(delivery) => {
            let partition = delivery.partition;
            let offset = delivery.offset;
            println!(
                "Submitted job_id={} partition={} offset={}",
                job.job_id, partition, offset
            );
        }
        Err((e, _msg)) => {
            eprintln!("Failed to submit: {e}");
        }
    }
}
