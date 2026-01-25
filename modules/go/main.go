package main

import "fmt"

//export add
func add(a, b int32) int32 {
	fmt.Printf("Go: Adding %d + %d\n", a, b)
	return a + b
}

//export multiply
func multiply(a, b int32) int32 {
	return a * b
}

//go:wasmimport host host_func
func hostFunc(value int32)

func main() {
	fmt.Println("Go WASI module initialized!")
	hostFunc(42)
}
