docker exec -it kafka /opt/kafka/bin/kafka-topics.sh \
  --bootstrap-server localhost:9092 \
  --create --topic wasm_jobs \
  --partitions 3 --replication-factor 1

docker exec -it kafka /opt/kafka/bin/kafka-topics.sh \
  --bootstrap-server localhost:9092 \
  --create --topic wasm_results \
  --partitions 3 --replication-factor 1
