package main

import (
	"fmt"
	"time"
)

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

func bench(size, iterations int) time.Duration {
	n := size

	a := make([]float64, n*n)
	b := make([]float64, n*n)
	c := make([]float64, n*n)

	for i := range n * n {
		a[i] = float64(i%17) + 0.5
		b[i] = float64(i%13) + 0.5
	}

	start := time.Now()
	for range iterations {
		matrixMultiply(a, b, c, n)
		a, c = c, a
	}
	elapsed := time.Since(start)

	// Prevent optimization
	checksum := a[0] + a[n*n-1]
	_ = checksum

	return elapsed
}

func main() {
	fmt.Println("=== Native Go Matrix Multiplication Benchmark ===")

	configs := []struct {
		size       int
		iterations int
	}{
		{64, 100},
		{128, 50},
		{256, 10},
		{512, 3},
	}

	for _, cfg := range configs {
		elapsed := bench(cfg.size, cfg.iterations)
		perIter := elapsed / time.Duration(cfg.iterations)
		ops := float64(cfg.size*cfg.size*cfg.size) * 2 * float64(cfg.iterations)
		gflops := ops / elapsed.Seconds() / 1e9

		fmt.Printf("%dx%d x%d: %v total, %v/iter, %.2f GFLOPS\n",
			cfg.size, cfg.size, cfg.iterations, elapsed, perIter, gflops)
	}
}
