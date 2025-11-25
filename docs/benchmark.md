# Performance Benchmark

## Test Environment

### System
- OS: Darwin 23.6.0
- CPU: arm64 (Apple Silicon)
- Ruby: 3.4.5 (arm64-darwin20)

### Tools
- rfmt: 0.2.4
- RuboCop: 1.81.7

### Test Project
- Total files: 111 Ruby files
- Total lines: 3,241 lines
- Type: Rails application

### Methodology
- Runs per test: 10
- Metrics: Average, Median, Standard Deviation
- Test date: 2025-11-25

## Results

### 1. Single File Performance

Test file: `20250202060108_articles.rb` (234 bytes, 11 lines)

| Tool | Average | Median | Std Dev | Min | Max |
|------|---------|--------|---------|-----|-----|
| rfmt | 191.2ms | 189.0ms | 22.7ms | 162.5ms | 243.7ms |
| RuboCop | 1.38s | 1.25s | 418.6ms | 1.19s | 2.56s |

**Ratio**: 7.21x

### 2. Directory Performance

Test directory: `app/models` (14 files)

| Tool | Average | Median | Std Dev | Min | Max |
|------|---------|--------|---------|-----|-----|
| rfmt | 175.8ms | 172.9ms | 8.2ms | 167.7ms | 192.1ms |
| RuboCop | 1.682s | 1.656s | 162.6ms | 1.54s | 2.10s |

**Ratio**: 9.57x

### 3. Full Project (Check Mode)

All 111 files in check mode (no file modifications)

| Tool | Average | Median | Std Dev | Min | Max |
|------|---------|--------|---------|-----|-----|
| rfmt | 171.9ms | 174.5ms | 9.3ms | 150.1ms | 184.9ms |
| RuboCop | 4.357s | 3.33s | 3.134s | 3.01s | 13.23s |

**Ratio**: 25.35x

## Observations

### Execution Time Consistency

rfmt shows consistent execution times across all tests:
- Standard deviation: 8-23ms
- Low variance between runs

RuboCop shows higher variance:
- Standard deviation: 163-3,134ms
- Notable outliers in full project test (13.23s max vs 3.33s median)

### Scalability

| Test Type | Files | rfmt | RuboCop | Ratio |
|-----------|-------|------|---------|-------|
| Single File | 1 | 191ms | 1.38s | 7.21x |
| Directory | 14 | 176ms | 1.68s | 9.57x |
| Full Project | 111 | 172ms | 4.36s | 25.35x |

rfmt execution time remains relatively constant (172-191ms) regardless of file count.

RuboCop execution time increases with file count.

## Use Cases

### Single File Formatting
- Editor save operations
- Pre-commit hooks for modified files
- Time difference: ~1.2 seconds per file

### Directory Formatting
- Module or component-level formatting
- Partial codebase updates
- Time difference: ~1.5 seconds per directory

### Full Project Validation
- CI/CD pipelines
- Pre-release checks
- Time difference: ~4.2 seconds per project

## Raw Data

Full benchmark results:
- [`docs/benchmark/results.json`](./benchmark/results.json)

## Benchmark Script

Source: [`scripts/benchmark.rb`](../scripts/benchmark.rb)

Run benchmark:
```bash
ruby scripts/benchmark.rb /path/to/rails/project
```
