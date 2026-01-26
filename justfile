GO_DIR := "modules/go"
GO_WIT_DIR := "{{GO_DIR}}/wit"
GO_WIT_PACKAGE := "myapp:component@0.1.0.wasm"
GO_WORLD := "my-world"
GO_OUTPUT := "component.wasm"

# Build WASI components (go or all)
b target='go':
	@case "{{target}}" in \
		go) just build-go ;; \
		all) just build-all ;; \
		*) echo "Unknown component target: {{target}}" >&2; exit 1 ;; \
	esac

build-all: build-go

build-go: wkg-go
	cd {{GO_DIR}} && tinygo build -target=wasip2 --wit-package ./{{GO_WIT_PACKAGE}} --wit-world {{GO_WORLD}} -no-debug -opt=2 -o {{GO_OUTPUT}} main.go

# Bundle WIT dependencies (go or all)
wkg target='go':
	@case "{{target}}" in \
		go) just wkg-go ;; \
		all) just wkg-all ;; \
		*) echo "Unknown wkg target: {{target}}" >&2; exit 1 ;; \
	esac

wkg-all: wkg-go

wkg-go:
	cd {{GO_DIR}} && wkg wit build

# Rust host utilities
build-host:
	cargo build

run: build-host
	cargo run

clean:
	cargo clean
	find modules -name "{{GO_OUTPUT}}" -delete
