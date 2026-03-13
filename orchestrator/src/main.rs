use common::{BROKERS, JOBS_TOPIC, RESULTS_TOPIC, Job, ResultMsg};
use rdkafka::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message as KafkaMessage;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;
use worker_api::Message;

// Request type matching the echo example
#[derive(Serialize, Deserialize, Debug)]
struct Request {
    data: String,
}

#[tokio::main]
async fn main() {
    // Producer: Send a job
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", BROKERS)
        .set("message.timeout.ms", "10000")
        .set("acks", "all")
        .set("enable.idempotence", "true")
        .create()
        .expect("Failed to create producer");

    // Create a request message
    let request = Request {
        data: "hello from orchestrator!".to_string(),
    };
    let message = Message::from(&request);

    let job = Job {
        job_id: Uuid::new_v4().to_string(),
        module_id: "echo".to_string(),
        message,
    };

    let payload = serde_json::to_string(&job).unwrap();

    let delivery = producer
        .send(
            FutureRecord::to(JOBS_TOPIC)
                .key(&job.job_id)
                .payload(&payload),
            Duration::from_secs(0),
        )
        .await;

    match delivery {
        Ok(delivery) => {
            let partition = delivery.partition;
            let offset = delivery.offset;
            println!(
                "Submitted job_id={} module={} partition={} offset={}",
                job.job_id, job.module_id, partition, offset
            );
        }
        Err((e, _msg)) => {
            eprintln!("Failed to submit: {e}");
        }
    }

    // Consumer: Listen for results
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", BROKERS)
        .set("group.id", "orchestrator-results")
        .set("auto.offset.reset", "latest")
        .create()
        .expect("Failed to create consumer");

    consumer
        .subscribe(&[RESULTS_TOPIC])
        .expect("Failed to subscribe");

    println!("Orchestrator listening for results...");

    loop {
        let msg = consumer.recv().await.expect("Failed to receive message");
        if let Some(Ok(payload)) = msg.payload_view::<str>() {
            // Parse ResultMsg
            match serde_json::from_str::<ResultMsg>(payload) {
                Ok(result) => {
                    if result.ok {
                        if let Some(output_bytes) = result.output {
                            // Deserialize the Message from bytes
                            match serde_json::from_slice::<Message>(&output_bytes) {
                                Ok(response_msg) => {
                                    // Parse the inner payload as JSON for display
                                    let payload_json: serde_json::Value =
                                        serde_json::from_slice(&response_msg.payload)
                                            .unwrap_or(serde_json::json!({}));

                                    println!(
                                        "✓ RESULT [worker={}] job_id={} type={} payload={}",
                                        result.worker_id,
                                        result.job_id,
                                        response_msg.type_name,
                                        payload_json
                                    );
                                }
                                Err(e) => {
                                    println!(
                                        "✓ RESULT [worker={}] job_id={} (failed to parse Message: {})",
                                        result.worker_id, result.job_id, e
                                    );
                                }
                            }
                        } else {
                            println!(
                                "✓ RESULT [worker={}] job_id={} (no output)",
                                result.worker_id, result.job_id
                            );
                        }
                    } else {
                        println!(
                            "✗ RESULT [worker={}] job_id={} error={}",
                            result.worker_id,
                            result.job_id,
                            result.error.as_deref().unwrap_or("unknown")
                        );
                    }
                }
                Err(e) => {
                    println!("RESULT (parse error): {}", e);
                }
            }
        }
    }
}