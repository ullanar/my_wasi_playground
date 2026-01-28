# Wasmtime P2 Rust Host

How to write a Rust host that runs WASM components using Wasmtime's Component Model (P2).

## Cargo.toml

```toml
[dependencies]
wasmtime = "41.0"
wasmtime-wasi = "41.0"
anyhow = "1.0"
```

## Minimal Example

```rust
use wasmtime::{
    Engine, Store,
    component::{Component, HasSelf, Linker, bindgen},
};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxView, WasiView};

// Generate bindings from inline WIT
bindgen!({
    inline: r#"
        package myns:mypackage@0.1.0;

        interface host {
            log: func(msg: string);
        }

        world myworld {
            import host;
            export process: func(input: string) -> string;
        }
    "#,
});

// Host state - holds WASI context
pub struct State {
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
}

// Required for WASI P2
impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}

// Implement your host interface
// Path: {namespace}::{package}::{interface}::Host
impl myns::mypackage::host::Host for State {
    fn log(&mut self, msg: String) {
        println!("[LOG] {}", msg);
    }
}

fn main() -> anyhow::Result<()> {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);

    // Add WASI P2 functions
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

    // Add your host functions
    // Path: {namespace}::{package}::{interface}::add_to_linker
    myns::mypackage::host::add_to_linker::<State, HasSelf<State>>(
        &mut linker,
        |state| state,
    )?;

    // Build WASI context
    let wasi = WasiCtx::builder()
        .inherit_stdio()
        .inherit_args()
        .inherit_env()
        .build();

    let state = State {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
    };

    let mut store = Store::new(&engine, state);
    let component = Component::from_file(&engine, "./component.wasm")?;
    
    // World name becomes struct: MyWorld, Playground, etc.
    let instance = Myworld::instantiate(&mut store, &component, &linker)?;

    // Call exports: call_{export_name}
    let result = instance.call_process(&mut store, "hello")?;
    println!("Result: {}", result);

    Ok(())
}
```

## Key Points

### bindgen! macro

Generates from WIT:
- World struct (e.g., `Myworld`) with `instantiate()` and `call_*` methods
- Host trait at `{ns}::{pkg}::{iface}::Host`
- `add_to_linker` function at same path

WIT naming → Rust naming:
- `my-world` → `MyWorld`
- `my-func` → `my_func` (methods), `call_my_func` (exports)
- `my-package` → `my_package`

### State requirements

Your state struct must implement:
1. `WasiView` - for WASI P2 support
2. All `Host` traits for imported interfaces

### Linker setup order

```rust
// 1. WASI first
wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

// 2. Your interfaces
myns::mypackage::host::add_to_linker::<State, HasSelf<State>>(&mut linker, |s| s)?;
```

### HasSelf wrapper

Required for `add_to_linker`. Pattern is always:
```rust
add_to_linker::<YourState, HasSelf<YourState>>(&mut linker, |state| state)
```

## WIT from file instead of inline

```rust
bindgen!({
    world: "myworld",
    path: "wit",  // reads from wit/ directory
    with: {
        "wasi": wasmtime_wasi::p2::bindings,
    },
});
```

Requires WIT deps in `wit/` directory. Use `wkg wit fetch` to get them.

## Host function return types

WIT functions without `-> result<T, E>`:
```rust
fn log(&mut self, msg: String) {
    // no return, or return ()
}
```

WIT functions with results:
```rust
fn compute(&mut self, x: i32) -> wasmtime::Result<i32> {
    Ok(x * 2)
}
```

## Type mappings

| WIT | Rust |
|-----|------|
| `string` | `String` |
| `list<T>` | `Vec<T>` |
| `option<T>` | `Option<T>` |
| `result<T, E>` | `Result<T, E>` |
| `u8`-`u64` | `u8`-`u64` |
| `s8`-`s64` | `i8`-`i64` |
| `f32`, `f64` | `f32`, `f64` |
| `bool` | `bool` |
| `tuple<A, B>` | `(A, B)` |

## Debug generated code

```bash
WASMTIME_DEBUG_BINDGEN=1 cargo build
```

Writes generated code to `target/` for inspection.

## Common errors

**"failed to find function export"**: Component doesn't export expected function. Check WIT world matches component.

**"failed to find import"**: Missing host impl or `add_to_linker` call.

**"trait bound `HasData` not satisfied"**: Use `HasSelf<State>` wrapper in `add_to_linker`.

**"package not found" with inline WIT**: Can't use `include wasi:cli/...` in inline WIT. Either use file-based WIT with deps, or don't include WASI in your WIT (add WASI separately via `wasmtime_wasi::p2::add_to_linker_sync`).
