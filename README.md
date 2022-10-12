## Workload Generator

This repository can be used for generating transactional workload modelled after the [TPC-C benchmark](https://www.tpc.org/tpc_documents_current_versions/pdf/tpc-c_v5.11.0.pdf).

### Usage
```
 ./up.sh -l ./workload/ -t customer -t neworder -t orders -t orderline -t stock -t item -t district -t warehouse
```

This command will to the following:
1. Start a `redpanda`, `kafka connect` and `postgres` docker container. The `postgres` container will be initialized with data that can be found in `./debezium/tpcc-benchmark/init/`.
2. Create a postgres to debezium connector.
3. Run the transactional workload.
4. For each TPC-C data table collect the transactional workload in workload files that can be consumed into a kafka topic.
