use wasmtime::{
    Engine,
    Store,
    component::{
        Component,
        HasSelf,
        Linker,
        bindgen,
    },
};
use wasmtime_wasi::{
    ResourceTable,
    WasiCtx,
    WasiCtxView,
    WasiView,
};

bindgen!({
    inline: r#"
        package wasi:playground@0.1.0;

        interface host {
            log: func(msg: string);
        }

        world playground {
            import host;
            export process: func(input: string) -> string;
            export matrix-bench: func(size: u32, iterations: u32) -> u64;
        }
    "#,
});

pub struct ComponentRunStates {
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
}

impl WasiView for ComponentRunStates {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}

impl wasi::playground::host::Host for ComponentRunStates {
    fn log(&mut self, msg: String) {
        println!("[HOST LOG] {}", msg);
    }
}

pub fn run() {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);

    wasmtime_wasi::p2::add_to_linker_sync(&mut linker).unwrap();
    wasi::playground::host::add_to_linker::<ComponentRunStates, HasSelf<ComponentRunStates>>(
        &mut linker,
        |state| state,
    )
    .unwrap();

    let wasi = WasiCtx::builder()
        .inherit_stdio()
        .inherit_args()
        .inherit_env()
        .build();

    let state = ComponentRunStates {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
    };

    let mut store = Store::new(&engine, state);

    let component = Component::from_file(&engine, "./components/go/component.wasm").unwrap();
    let instance = Playground::instantiate(&mut store, &component, &linker).unwrap();

    let result = instance
        .call_process(&mut store, "Hello from Rust!")
        .unwrap();
    println!("Go component returned: {}", result);

    // Matrix multiplication benchmark
    println!("\n=== Matrix Multiplication Benchmark ===");

    let configs = [
        (64, 100), // Small: 64x64, 100 iterations
        (128, 50), // Medium: 128x128, 50 iterations
        (256, 10), // Large: 256x256, 10 iterations
        (512, 3),  // XL: 512x512, 3 iterations
    ];

    for (size, iterations) in configs {
        let start = std::time::Instant::now();
        let wasm_nanos = instance
            .call_matrix_bench(&mut store, size, iterations)
            .unwrap();
        let total_elapsed = start.elapsed();

        let wasm_duration = std::time::Duration::from_nanos(wasm_nanos);
        let per_iter = wasm_duration / iterations;
        let ops = (size as u64).pow(3) * 2 * iterations as u64; // 2*n^3 FLOPs per matmul
        let gflops = ops as f64 / wasm_duration.as_secs_f64() / 1e9;

        println!(
            "{}x{} x{}: {:?} total, {:?}/iter, {:.2} GFLOPS (call overhead: {:?})",
            size,
            size,
            iterations,
            wasm_duration,
            per_iter,
            gflops,
            total_elapsed - wasm_duration
        );
    }
}
