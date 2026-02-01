use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use wasmtime::{
    Engine, Store,
    component::{Component, HasSelf, Linker, bindgen},
};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxView, WasiView};

use crate::{Command, Event, Response};

bindgen!({
    inline: r#"
        package wasi:playground@0.1.0;

        interface host {
            log: func(msg: string);
            get-entities: func() -> string;
            spawn-entity: func(name: string, x: f32, y: f32) -> u64;
            rpc-call: func(target: string, method: string, args: string) -> u64;
        }

        world playground {
            import host;
            export process: func(input: string);
            export on-rpc-request: func(caller: string, method: string, args: string) -> string;
            export on-rpc-response: func(request-id: u64, response: string);
            export matrix-bench: func(size: u32, iterations: u32) -> u64;
        }
    "#,
});

const COMPONENTS: &[(&str, &str)] = &[
    ("server", "./components/go/component.wasm"),
    ("client", "./components/rust/component.wasm"),
];

#[derive(Debug)]
struct RpcRequest {
    id: u64,
    from: String,
    to: String,
    method: String,
    args: String,
}

struct SharedState {
    event_tx: Sender<Event>,
    resp_rx: Receiver<Response>,
    rpc_queue: Vec<RpcRequest>,
    next_rpc_id: u64,
    current_component: String,
}

struct State {
    wasi_ctx: WasiCtx,
    resource_table: ResourceTable,
    shared: Arc<Mutex<SharedState>>,
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
        let shared = self.shared.lock().unwrap();
        println!("    [{}] {}", shared.current_component, msg);
    }

    fn get_entities(&mut self) -> String {
        let shared = self.shared.lock().unwrap();
        shared.event_tx.send(Event::GetEntities).unwrap();
        match shared.resp_rx.recv().unwrap() {
            Response::Entities(data) => data,
            _ => panic!("unexpected response"),
        }
    }

    fn spawn_entity(&mut self, name: String, x: f32, y: f32) -> u64 {
        let shared = self.shared.lock().unwrap();
        shared
            .event_tx
            .send(Event::SpawnEntity { name, x, y })
            .unwrap();
        match shared.resp_rx.recv().unwrap() {
            Response::Spawned(id) => id,
            _ => panic!("unexpected response"),
        }
    }

    fn rpc_call(&mut self, target: String, method: String, args: String) -> u64 {
        let mut shared = self.shared.lock().unwrap();
        let id = shared.next_rpc_id;
        shared.next_rpc_id += 1;

        let from = shared.current_component.clone();
        println!(
            "    [host] RPC {} -> {}: {}({})",
            from, target, method, args
        );

        shared.rpc_queue.push(RpcRequest {
            id,
            from,
            to: target,
            method,
            args,
        });
        id
    }
}

struct ComponentInstance {
    store: Store<State>,
    instance: Playground,
}

pub fn run(cmd_rx: Receiver<Command>, event_tx: Sender<Event>, resp_rx: Receiver<Response>) {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);

    wasmtime_wasi::p2::add_to_linker_sync(&mut linker).unwrap();
    wasi::playground::host::add_to_linker::<State, HasSelf<State>>(&mut linker, |s| s).unwrap();

    let shared = Arc::new(Mutex::new(SharedState {
        event_tx,
        resp_rx,
        rpc_queue: Vec::new(),
        next_rpc_id: 1,
        current_component: String::new(),
    }));

    let mut instances: HashMap<String, ComponentInstance> = COMPONENTS
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
                shared: Arc::clone(&shared),
            };
            let mut store = Store::new(&engine, state);
            let instance = Playground::instantiate(&mut store, &component, &linker).ok()?;

            println!("[WASI] Loaded: {}", name);
            Some((name.to_string(), ComponentInstance { store, instance }))
        })
        .collect();

    println!("[WASI] {} components ready\n", instances.len());

    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            Command::Tick(tick) => {
                println!("[WASI] Tick {}", tick);

                let names: Vec<_> = instances.keys().cloned().collect();
                for name in &names {
                    shared.lock().unwrap().current_component = name.clone();

                    let c = instances.get_mut(name).unwrap();
                    let msg = format!("tick:{}", tick);
                    c.instance.call_process(&mut c.store, &msg).unwrap();
                }

                // Process RPC queue
                let requests: Vec<_> = shared.lock().unwrap().rpc_queue.drain(..).collect();
                for req in requests {
                    if let Some(target) = instances.get_mut(&req.to) {
                        shared.lock().unwrap().current_component = req.to.clone();

                        println!("  [RPC] {} -> {}: {}", req.from, req.to, req.method);
                        let result = target
                            .instance
                            .call_on_rpc_request(
                                &mut target.store,
                                &req.from,
                                &req.method,
                                &req.args,
                            )
                            .unwrap();
                        println!("  [RPC] {} <- {}: {}", req.from, req.to, result);

                        if let Some(caller) = instances.get_mut(&req.from) {
                            shared.lock().unwrap().current_component = req.from.clone();
                            caller
                                .instance
                                .call_on_rpc_response(&mut caller.store, req.id, &result)
                                .unwrap();
                        }
                    }
                }

                // Signal tick done
                shared
                    .lock()
                    .unwrap()
                    .event_tx
                    .send(Event::TickDone)
                    .unwrap();
            }
        }
    }

    println!("[WASI] Shutdown");
}
