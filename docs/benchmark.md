# Performance Benchmark

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

## Raw Data

Full benchmark results:
- [`tmp/benchmark_test/comparison_report.txt`](../tmp/benchmark_test/comparison_report.txt)

## Benchmark Script

Source: [`tmp/benchmark_test/comparison_benchmark.rb`](../tmp/benchmark_test/comparison_benchmark.rb)

Run benchmark:
```bash
ruby tmp/benchmark_test/comparison_benchmark.rb
```
