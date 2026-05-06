# FASTA Benchmark: bio-rs vs Biopython

Benchmarked with [hyperfine](https://github.com/sharkdp/hyperfine).

## Visual summary

| Workload | bio-rs | Biopython | Speedup |
| --- | --- | --- | --- |
| parse | 44.73ms | 197.66ms | **4.4x** |
| validate | 35.16ms | 603.45ms | **17.2x** |
| tokenize | 57.97ms | 598.02ms | **10.3x** |

## Short narrative

In this recorded FASTA benchmark, bio-rs is faster than the matched
Biopython loops for parse, validation, and tokenization work. The largest
gap is `validate`, where bio-rs is `17.2x` faster in the committed
artifact.

## Claim boundary

These numbers support a narrow claim: bio-rs is materially faster on the
committed matched FASTA workloads represented in
`benchmarks/fasta_vs_biopython.json`. They do not claim universal FASTA
or Biopython superiority across every workload.

Regenerate this Markdown from the JSON artifact with:

```bash
python3 scripts/render_benchmark_report.py > benchmarks/fasta_vs_biopython.md
```
