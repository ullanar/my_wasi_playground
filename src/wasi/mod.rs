use std::sync::mpsc::{Receiver, Sender};
use wasmtime::{
    Engine, Store,
    component::{Component, HasSelf, Linker, bindgen},
};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxView, WasiView};

use crate::Command;

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

const COMPONENTS: &[(&str, &str)] = &[
    ("Go", "./components/go/component.wasm"),
    ("Rust", "./components/rust/component.wasm"),
];

struct State {
    wasi_ctx: WasiCtx,
    resource_table: ResourceTable,
}

impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}

impl wasi::playground::host::Host for State {
    fn log(&mut self, msg: String) {
        println!("    [log] {}", msg);
    }
}

struct ComponentInstance {
    name: &'static str,
    store: Store<State>,
    instance: Playground,
}

pub fn run(cmd_rx: Receiver<Command>, done_tx: Sender<()>) {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);

    wasmtime_wasi::p2::add_to_linker_sync(&mut linker).unwrap();
    wasi::playground::host::add_to_linker::<State, HasSelf<State>>(&mut linker, |s| s).unwrap();

    let mut instances: Vec<ComponentInstance> = COMPONENTS
        .iter()
        .filter_map(|(name, path)| {
            let component = Component::from_file(&engine, path).ok()?;
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
            let instance = Playground::instantiate(&mut store, &component, &linker).ok()?;

            println!("[WASI] Loaded: {}", name);
            Some(ComponentInstance {
                name,
                store,
                instance,
            })
        })
        .collect();

    println!("[WASI] {} components ready\n", instances.len());

    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            Command::Tick(tick) => {
                println!("[WASI] Tick {}", tick);
                let msg = format!("tick:{}", tick);
                for c in &mut instances {
                    let result = c.instance.call_process(&mut c.store, &msg).unwrap();
                    println!("  {}: {}", c.name, result);
                }
            }
            Command::Benchmark => {
                println!("\n[WASI] Running benchmarks...");
                let configs = [(64, 100), (128, 50), (256, 10)];
                for c in &mut instances {
                    println!("  {}:", c.name);
                    for (size, iterations) in configs {
                        let nanos = c
                            .instance
                            .call_matrix_bench(&mut c.store, size, iterations)
                            .unwrap();
                        let ops = (size as u64).pow(3) * 2 * iterations as u64;
                        let gflops = ops as f64 / (nanos as f64 / 1e9) / 1e9;
                        let ms = nanos as f64 / 1_000_000.0;
                        println!(
                            "    {}x{} x{}: {:.1}ms, {:.2} GFLOPS",
                            size, size, iterations, ms, gflops
                        );
                    }
                }
            }
        }
        done_tx.send(()).unwrap();
    }

    println!("[WASI] Shutdown");
}
