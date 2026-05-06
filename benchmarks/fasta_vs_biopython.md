# FASTA Benchmark: bio-rs vs Biopython

Benchmarked with [hyperfine](https://github.com/sharkdp/hyperfine).

| Workload | bio-rs | Biopython | Speedup |
| --- | --- | --- | --- |
| parse | 44.10ms | 197.06ms | **4.5x** |
| validate | 35.03ms | 589.90ms | **16.8x** |
| tokenize | 57.66ms | 599.19ms | **10.4x** |
