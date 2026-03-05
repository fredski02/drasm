use rdkafka::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;

const BROKERS: &str = "localhost:9092";
const RESULTS_TOPIC: &str = "wasm_results";

#[tokio::main]
async fn main() {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", BROKERS)
        .set("group.id", "results-printer")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("consumer");

    consumer.subscribe(&[RESULTS_TOPIC]).expect("subscribe");

    println!("Results consumer running...");

    loop {
        let msg = consumer.recv().await.expect("recv");
        if let Some(Ok(s)) = msg.payload_view::<str>() {
            println!("RESULT: {s}");
        }
    }
}

