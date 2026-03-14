use common::{JOBS_TOPIC, Job};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::sync::Arc;
use std::time::Duration;

pub type SharedKafkaProducer = Arc<FutureProducer>;

pub fn create_kafka_producer(brokers: &str) -> anyhow::Result<FutureProducer> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "10000")
        .set("acks", "all")
        .set("enable.idempotence", "true")
        .create()?;
    
    Ok(producer)
}

pub async fn publish_job(producer: &FutureProducer, job: &Job) -> anyhow::Result<(i32, i64)> {
    let payload = serde_json::to_string(job)?;
    
    let delivery = producer
        .send(
            FutureRecord::to(JOBS_TOPIC)
                .key(&job.job_id)
                .payload(&payload),
            Duration::from_secs(0),
        )
        .await
        .map_err(|(e, _)| e)?;
    
    Ok((delivery.partition, delivery.offset))
}
