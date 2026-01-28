# Matrix Multiplication Benchmark

Comparing native execution vs WASM components running in wasmtime.

## Results

### Go (TinyGo)

| Size | Iterations | Native | WASM | Overhead |
|------|------------|--------|------|----------|
| 64x64 | 100 | 42ms / 1.23 GFLOPS | 57ms / 0.93 GFLOPS | 1.36x |
| 128x128 | 50 | 174ms / 1.21 GFLOPS | 233ms / 0.90 GFLOPS | 1.34x |
| 256x256 | 10 | 349ms / 0.96 GFLOPS | 401ms / 0.84 GFLOPS | 1.15x |
| 512x512 | 3 | 745ms / 1.08 GFLOPS | 954ms / 0.84 GFLOPS | 1.28x |

## Running Benchmarks

**WASM (wasmtime host):**
```bash
cargo run
```

**Native Go:**
```bash
cd bench/go-native && go run main.go
```

## Notes

- Rust host build mode (debug/release) has no effect on WASM execution speed - JIT compiles WASM regardless
