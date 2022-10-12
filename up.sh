#!/bin/bash

# ./up.sh -l ./workload/ -t customer -t neworder -t orders -t orderline -t stock -t item -t district -t warehouse

while getopts q:l:t: flag
do
    case "${flag}" in
        l) location=${OPTARG};;
        t) multi+=("$OPTARG");;
    esac
done

docker-compose -f debezium/redpanda-debezium.compose.yml up -d && sleep 6
sh debezium/tpcc-benchmark/connectors/create-postgres-debezium-connector.sh && sleep 2 # >/dev/null 2>&1
cargo run

wait

for table in "${multi[@]}"; do
    echo "table: $table"
    rm ${location}${table}.1
    docker exec -t debezium-redpanda-1 rpk topic consume postgres.public.${table} --format '%v\n' >> ${location}${table}.1 &
done
