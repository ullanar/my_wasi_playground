package main

import (
	"fmt"
	"strings"
	"time"

	"go_rust_wasi_test/internal/wasi/playground/host"
	"go_rust_wasi_test/internal/wasi/playground/playground"
)

func process(input string) {
	host.Log("Processing: " + input)

	// On tick 4, run benchmark and ask client to run theirs
	if input == "tick:4" {
		host.Log("Running server benchmark...")
		runBenchmark()
		host.Log("Asking client to run benchmark...")
		host.RPCCall("client", "run_benchmark", "")
	}
}

func runBenchmark() {
	configs := []struct {
		size       uint32
		iterations uint32
	}{
		{64, 100},
		{128, 50},
		{256, 10},
	}

	for _, cfg := range configs {
		nanos := matrixBench(cfg.size, cfg.iterations)
		ms := float64(nanos) / 1_000_000.0
		ops := uint64(cfg.size) * uint64(cfg.size) * uint64(cfg.size) * 2 * uint64(cfg.iterations)
		gflops := float64(ops) / (float64(nanos) / 1e9) / 1e9
		host.Log(fmt.Sprintf("  %dx%d x%d: %.1fms, %.2f GFLOPS", cfg.size, cfg.size, cfg.iterations, ms, gflops))
	}
}

func onRpcRequest(caller string, method string, args string) string {
	host.Log(fmt.Sprintf("RPC from %s: %s(%s)", caller, method, args))

	switch method {
	case "ready_to_spawn":
		playerName := args
		if playerName == "" {
			playerName = "player"
		}
		id := host.SpawnEntity(playerName, 50.0, 50.0)
		host.Log(fmt.Sprintf("Spawned player '%s' with id=%d", playerName, id))
		return fmt.Sprintf("ok:%d", id)

	case "ping":
		return "pong:" + args

	default:
		return "error:unknown_method"
	}
}

func onRpcResponse(requestId uint64, response string) {
	host.Log(fmt.Sprintf("RPC response %d: %s", requestId, response))
}

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

func matrixBench(size uint32, iterations uint32) uint64 {
	n := int(size)
	iters := int(iterations)

	a := make([]float64, n*n)
	b := make([]float64, n*n)
	c := make([]float64, n*n)

	for i := range n * n {
		a[i] = float64(i%17) + 0.5
		b[i] = float64(i%13) + 0.5
	}

	start := time.Now()
	for range iters {
		matrixMultiply(a, b, c, n)
		a, c = c, a
	}
	elapsed := time.Since(start)

	return uint64(elapsed.Nanoseconds())
}

func init() {
	playground.Exports.Process = process
	playground.Exports.OnRPCRequest = onRpcRequest
	playground.Exports.OnRPCResponse = onRpcResponse
	playground.Exports.MatrixBench = matrixBench
}

var _ = strings.Split

func main() {}
