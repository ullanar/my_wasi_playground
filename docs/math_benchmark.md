# Matrix Multiplication Benchmark

Comparing WASM components running in wasmtime.

## Results

### Go (TinyGo) vs Rust

| Size | Iterations | Go WASM | Rust WASM | Rust speedup |
|------|------------|---------|-----------|--------------|
| 64x64 | 100 | 62ms / 0.84 GFLOPS | 35ms / 1.51 GFLOPS | 1.8x |
| 128x128 | 50 | 241ms / 0.87 GFLOPS | 142ms / 1.48 GFLOPS | 1.7x |
| 256x256 | 10 | 412ms / 0.81 GFLOPS | 361ms / 0.93 GFLOPS | 1.15x |
| 512x512 | 3 | 970ms / 0.83 GFLOPS | 833ms / 0.97 GFLOPS | 1.17x |

### Call Overhead

| Size | Go overhead | Rust overhead |
|------|-------------|---------------|
| 64x64 | ~1.4ms | ~0.17ms |
| 512x512 | ~16ms | ~2.3ms |

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
- Rust WASM component has significantly lower call overhead than Go
