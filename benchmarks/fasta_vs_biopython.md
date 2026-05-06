# FASTA Benchmark: bio-rs vs Biopython

Benchmarked with [hyperfine](https://github.com/sharkdp/hyperfine).

| Workload | bio-rs | Biopython | Speedup |
| --- | --- | --- | --- |
| parse | 44.73ms | 197.66ms | **4.4x** |
| validate | 35.16ms | 603.45ms | **17.2x** |
| tokenize | 57.97ms | 598.02ms | **10.3x** |
