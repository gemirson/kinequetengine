# KCE Load Tests

## Prerequisites
- Install k6: https://k6.io/docs/getting-started/installation/
- Start KCE server: `cargo run --release --bin kce-server`

## Run
```bash
k6 run tests/load/query-load.js
```

## Expected Results
- P99 < 10ms
- P95 < 5ms
- P50 < 2ms
- Error rate < 1%
