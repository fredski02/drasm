use common::Job;
use rdkafka::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::Serialize;
use std::time::Duration;
use wasmtime::{Engine, Instance, Module, Store};
const BROKERS: &str = "localhost:9092";
const JOBS_TOPIC: &str = "wasm_jobs";
const RESULTS_TOPIC: &str = "wasm_results";
const GROUP_ID: &str = "wasm-workers";

#[derive(Serialize)]
struct ResultMsg {
    job_id: String,
    ok: bool,
    output: Option<i32>,
    error: Option<String>,
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
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", BROKERS)
        .set("group.id", GROUP_ID)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("consumer");

    consumer.subscribe(&[JOBS_TOPIC]).expect("subscribe");

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", BROKERS)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("producer");

    println!("Worker running. Waiting for jobs...");

    loop {
        let msg = match consumer.recv().await {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Kafka error: {e}");
                continue;
            }
        };

        let payload = match msg.payload_view::<str>() {
            Some(Ok(s)) => s,
            _ => {
                eprintln!("Bad payload (not utf-8), skipping");
                // commit skip
                let _ = consumer.commit_message(&msg, rdkafka::consumer::CommitMode::Async);
                continue;
            }
        };

        let job: Job = match serde_json::from_str(payload) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("Bad JSON: {e}");
                let _ = consumer.commit_message(&msg, rdkafka::consumer::CommitMode::Async);
                continue;
            }
        };

        let result = match run_wat(&job.wat, job.input) {
            Ok(out) => ResultMsg {
                job_id: job.job_id.clone(),
                ok: true,
                output: Some(out),
                error: None,
            },
            Err(e) => ResultMsg {
                job_id: job.job_id.clone(),
                ok: false,
                output: None,
                error: Some(e.to_string()),
            },
        };

        let result_json = serde_json::to_string(&result).unwrap();

        // publish result
        let delivery = producer
            .send(
                FutureRecord::to(RESULTS_TOPIC)
                    .key(&job.job_id)
                    .payload(&result_json),
                Duration::from_secs(3),
            )
            .await;

        if let Err((e, _)) = delivery {
            eprintln!("Failed to publish result: {e}");
            // do NOT commit; will retry job on restart (basic at-least-once)
            continue;
        }

        // commit after successful publish (classic at-least-once)
        let _ = consumer.commit_message(&msg, rdkafka::consumer::CommitMode::Async);

        println!(
            "Processed job_id={} input={} -> {:?}",
            job.job_id, job.input, result.output
        );
    }
}

