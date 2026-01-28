package main

import (
	"fmt"
	"time"

	"go_rust_wasi_test/internal/wasi/playground/host"
	"go_rust_wasi_test/internal/wasi/playground/playground"
)

func process(input string) string {
	host.Log("Processing: " + input)
	return "Processed: " + input
}

// matrixMultiply multiplies two NxN matrices in-place: c = a * b
// All matrices are flattened row-major
func matrixMultiply(a, b, c []float64, n int) {
	for i := range n {
		for j := range n {
			var sum float64
			for k := range n {
				sum += a[i*n+k] * b[k*n+j]
			}
			c[i*n+j] = sum
		}
	}
}

// matrixBench runs matrix multiplication benchmark
func matrixBench(size uint32, iterations uint32) uint64 {
	n := int(size)
	iters := int(iterations)

	host.Log(fmt.Sprintf("Starting benchmark: %dx%d matrix, %d iterations", n, n, iters))

	// Allocate matrices once
	a := make([]float64, n*n)
	b := make([]float64, n*n)
	c := make([]float64, n*n)

	// Initialize with deterministic values
	for i := range n * n {
		a[i] = float64(i%17) + 0.5
		b[i] = float64(i%13) + 0.5
	}

	// Run benchmark
	start := time.Now()
	for range iters {
		matrixMultiply(a, b, c, n)
		// Swap a and c to use result as next input (prevents dead code elimination)
		a, c = c, a
	}
	elapsed := time.Since(start)

	// Use result to prevent optimization
	checksum := a[0] + a[n*n-1]
	host.Log(fmt.Sprintf("Benchmark done: checksum=%f", checksum))

	return uint64(elapsed.Nanoseconds())
}

func init() {
	playground.Exports.Process = process
	playground.Exports.MatrixBench = matrixBench
}

func main() {}
