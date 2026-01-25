# Build a module by name, or "all" for all modules
b module:
    @if [ "{{module}}" = "all" ]; then just _build-all; else just _build-{{module}}; fi

_build-all: _build-go

# Build Go module
_build-go:
    cd modules/go && tinygo build -o module.wasm -target=wasi -opt=2 main.go

# Build the Rust host
build-host:
    cargo build

# Run the host
run: build-host
    cargo run

# Clean build artifacts
clean:
    cargo clean
    find modules -name "*.wasm" -delete
