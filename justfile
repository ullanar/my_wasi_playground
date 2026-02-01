GO_DIR := "components/go"
GO_WIT_DIR := "{{GO_DIR}}/wit"
GO_WIT_PACKAGE := "wasi:playground@0.1.0.wasm"
GO_WORLD := "playground"
GO_OUTPUT := "component.wasm"

RUST_DIR := "components/rust"
RUST_OUTPUT := "component.wasm"

# Build WASI components (go, rust, or all)
b target='go':
	@case "{{target}}" in \
		go) just build-go ;; \
		rust) just build-rust ;; \
		all) just build-all ;; \
		*) echo "Unknown component target: {{target}}" >&2; exit 1 ;; \
	esac

build-all: build-go build-rust

build-go: wkg-go
	cd {{GO_DIR}} && tinygo build -target=wasip2 --wit-package ./{{GO_WIT_PACKAGE}} --wit-world {{GO_WORLD}} -no-debug -opt=2 -o {{GO_OUTPUT}} main.go

build-rust: wkg-rust
	cd {{RUST_DIR}} && cargo build --target wasm32-wasip2 --release
	cp {{RUST_DIR}}/target-wasm/wasm32-wasip2/release/wasi_playground.wasm {{RUST_DIR}}/{{RUST_OUTPUT}}

# Bundle WIT dependencies (go, rust, or all)
wkg target='go':
	@case "{{target}}" in \
		go) just wkg-go ;; \
		rust) just wkg-rust ;; \
		all) just wkg-all ;; \
		*) echo "Unknown wkg target: {{target}}" >&2; exit 1 ;; \
	esac

wkg-all: wkg-go wkg-rust

wkg-go:
	cd {{GO_DIR}} && wkg wit build

wkg-rust:
	cd {{RUST_DIR}} && wkg wit fetch

# Rust host utilities
build-host:
	cargo build

run: build-host
	cargo run

clean:
	cargo clean
	find components -name "{{GO_OUTPUT}}" -delete
