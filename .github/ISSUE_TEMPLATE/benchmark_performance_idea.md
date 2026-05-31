---
name: Benchmark or performance idea
about: Suggest a reproducible benchmark, dataset, or performance comparison
title: "benchmark: "
labels: benchmark
assignees: ""
---

## What should be measured?

- Matched workload:
  - [ ] pure parse
  - [ ] parse plus validation
  - [ ] parse plus tokenization
  - [ ] model-input construction
  - [ ] workflow or pipeline orchestration
  - [ ] dataset inspect
  - [ ] package validation or verification
- Surface:
  - [ ] core library only
  - [ ] CLI end-to-end
  - [ ] Python binding
  - [ ] WASM/JavaScript binding
  - [ ] MCP or service contract
  - [ ] optional Candle backend
- Metrics:
  - [ ] mean time
  - [ ] residues/sec
  - [ ] MB/sec
  - [ ] binding or request overhead
  - [ ] package/runtime validation latency

## Why does this benchmark matter?


## Dataset or input

- Source:
- Size:
- License or access notes:

## Baseline to compare against


## Completion criteria

- [ ] Benchmark input is documented
- [ ] Command is reproducible
- [ ] Result is recorded without overclaiming
- [ ] Surface and non-claim boundaries are explicit
