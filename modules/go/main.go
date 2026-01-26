package main

import (
	"go_rust_wasi_test/internal/myapp/component/host"
	myworld "go_rust_wasi_test/internal/myapp/component/my-world"
)

func process(input string) string {
	host.Log("Processing: " + input)
	return "Processed: " + input
}

func main() {
	myworld.Exports.Process = process
}
