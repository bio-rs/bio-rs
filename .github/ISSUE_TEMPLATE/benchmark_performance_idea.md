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
  - [ ] fixed-length model-input construction
  - [ ] no-padding model-input construction
  - [ ] workflow or pipeline orchestration
  - [ ] pipeline config execution
  - [ ] dataset inspect
  - [ ] package validation or verification
  - [ ] package artifact validation
  - [ ] package fixture verification
  - [ ] binding round-trip overhead
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
- Benchmark purpose:
  - [ ] release claim
  - [ ] regression guard
  - [ ] smoke benchmark
  - [ ] exploratory measurement

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
