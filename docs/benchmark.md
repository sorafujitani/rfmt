# Performance Benchmark

## In-Process Formatting Throughput (current)

Parsing and formatting both run inside the Rust extension (the ruby-prism crate, with prism statically linked). In-process throughput, measured with `scripts/bench_format.rb` over rfmt's own `lib/` corpus (15 files, 5 warm rounds) on arm64 macOS, Ruby 3.4:

| Pipeline | In-process format time |
|----------|------------------------|
| Before native parsing (Ruby Prism parse + JSON handoff to Rust; historical, not reproducible from this checkout) | 4.28 ms/file |
| After (parsing and formatting in Rust) | 0.19 ms/file |

Reproduce:

```bash
bundle exec ruby scripts/bench_format.rb
```

Cold CLI wall clock (`bundle exec rfmt --check FILE`, median of 5 runs on the same machine) is about 0.23 s with Bundler, 0.11 s without (`ruby -Ilib exe/rfmt --check FILE`). Ruby VM and Bundler startup dominate; formatting itself is a rounding error. Pipeline changes must therefore be measured in-process.

## Historical CLI Comparison vs RuboCop (rfmt 1.3.3)

Everything below was measured before the native parsing migration, with rfmt 1.3.3. It compares cold CLI wall-clock time, which is dominated by VM startup for both tools. The benchmark scripts lived in a local `tmp/` directory and are not part of the repository, so these numbers are kept for historical context only and cannot be reproduced from a checkout.

## Test Environment

### System
- OS: Darwin 23.6.0
- CPU: arm64 (Apple Silicon)
- Ruby: 3.4.8

### Tools
- rfmt: 1.3.3
- RuboCop: 1.82.1

### Test Data
- Total files: 53 Ruby files
- File types: Models, controllers, libraries, services
- Application type: Rails-style

### Methodology
- Runs per test: 5
- Metrics collected: Average, Median, Standard Deviation
- Test date: 2026-01-17

## Results

### 1. Small Files Performance

Test: `app/models` directory (9 files, ~1,000 lines total)

| Tool | Average | Median | Std Dev | Min | Max |
|------|---------|--------|---------|-----|-----|
| rfmt | 122.1ms | - | 19.1ms | - | - |
| RuboCop | 798.0ms | - | 23.1ms | - | - |

**Ratio**: 6.54x

### 2. Medium Files Performance

Test: `lib` directory (9 files, ~1,700 lines total)

| Tool | Average | Median | Std Dev | Min | Max |
|------|---------|--------|---------|-----|-----|
| rfmt | 120.4ms | - | 5.1ms | - | - |
| RuboCop | 797.0ms | - | 16.3ms | - | - |

**Ratio**: 6.62x

### 3. Large Project Performance

Test: `large_project/app/models` directory (35 files, ~4,200 lines total)

| Tool | Average | Median | Std Dev | Min | Max |
|------|---------|--------|---------|-----|-----|
| rfmt | 121.5ms | - | 9.5ms | - | - |
| RuboCop | 798.1ms | - | 16.0ms | - | - |

**Ratio**: 6.57x

## Measured Performance Characteristics

### Execution Time Statistics

rfmt:
- Standard deviation: 5-19ms
- Average execution time: ~120ms across all test sizes
- Variance between runs: 5.1-19.1ms

RuboCop:
- Standard deviation: 16-23ms  
- Average execution time: ~798ms across all test sizes
- Variance between runs: 16.0-23.1ms

### Performance by Project Size

| Files | Total Lines | rfmt | RuboCop | Ratio |
|-------|-------------|------|---------|-------|
| 9 | ~1,000 | 122ms | 798ms | 6.54x |
| 9 | ~1,700 | 120ms | 797ms | 6.62x |
| 35 | ~4,200 | 122ms | 798ms | 6.57x |

Execution time measurements across different file counts and line counts.

## Execution Time Comparison

| Context | rfmt | RuboCop | Time Difference |
|---------|------|---------|----------------|
| Editor save operation | ~120ms | ~798ms | 678ms |
| Pre-commit hook | ~120ms | ~798ms | 678ms |
| CI pipeline check | ~120ms | ~798ms | 678ms |

## Performance Metrics Summary

- Average speed ratio: 6.54-6.62x
- rfmt average execution: 120-122ms
- RuboCop average execution: 797-798ms
- Execution time difference: ~678ms

## Raw Data and Scripts

The historical comparison above was produced by scripts in a local `tmp/benchmark_test/` directory that is not checked into the repository. Raw CLI timing data from an earlier run is kept in [`benchmark/results.json`](benchmark/results.json).

The reproducible benchmark for the current pipeline is [`scripts/bench_format.rb`](../scripts/bench_format.rb):

```bash
bundle exec ruby scripts/bench_format.rb
```
