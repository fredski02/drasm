use rdkafka::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use common::{ResultMsg, BROKERS, RESULTS_TOPIC};

#[tokio::main]
async fn main() {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", BROKERS)
        .set("group.id", "results-printer")
        .set("auto.offset.reset", "latest")  // Changed from "earliest" - only show new results
        // Note: enable.auto.commit defaults to true with 5s interval.
        // This is acceptable for a results printer that doesn't need
        // guaranteed delivery (ephemeral consumer for monitoring only).
        .create()
        .expect("Failed to create consumer");

    consumer.subscribe(&[RESULTS_TOPIC]).expect("Failed to subscribe");

    println!("Results consumer running (showing new results only)...");

    loop {
        let msg = consumer.recv().await.expect("Failed to receive message");
        if let Some(Ok(payload)) = msg.payload_view::<str>() {
            // Parse and pretty-print result
            match serde_json::from_str::<ResultMsg>(payload) {
                Ok(result) => {
                    if result.ok {
                        println!(
                            "✓ RESULT [worker={}] job_id={} output={}",
                            result.worker_id,
                            result.job_id,
                            result.output.unwrap_or(0)
                        );
                    } else {
                        println!(
                            "✗ RESULT [worker={}] job_id={} error={}",
                            result.worker_id,
                            result.job_id,
                            result.error.as_deref().unwrap_or("unknown")
                        );
                    }
                }
                Err(_) => {
                    // Fallback for non-JSON or old format
                    println!("RESULT (raw): {}", payload);
                }
            }
        }
    }
}
