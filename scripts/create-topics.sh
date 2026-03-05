#!/bin/bash
# Create Kafka topics for DRASM

docker exec kafka /opt/kafka/bin/kafka-topics.sh \
  --bootstrap-server kafka:9092 \
  --create --topic wasm_jobs \
  --partitions 3 --replication-factor 1

docker exec kafka /opt/kafka/bin/kafka-topics.sh \
  --bootstrap-server kafka:9092 \
  --create --topic wasm_results \
  --partitions 3 --replication-factor 1

docker exec kafka /opt/kafka/bin/kafka-topics.sh \
  --bootstrap-server kafka:9092 \
  --create --topic wasm_jobs_dlq \
  --partitions 3 --replication-factor 1

echo "Topics created successfully!"
echo "View them at: http://localhost:8080"