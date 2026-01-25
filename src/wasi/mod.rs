use wasmtime::{
    Caller,
    Engine,
    Linker,
    Module,
    Store,
};
use wasmtime_wasi::{
    WasiCtxBuilder,
    p1::WasiP1Ctx,
};

pub fn run() {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);

    wasmtime_wasi::p1::add_to_linker_sync(&mut linker, |s| s).unwrap();

    linker
        .func_wrap(
            "host",
            "host_func",
            |_: Caller<'_, WasiP1Ctx>, param: i32| {
                println!("Got {} from WebAssembly", param);
            },
        )
        .expect("failed to define host_func");

    let module = Module::from_file(&engine, "./modules/go/module.wasm").unwrap();

    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_env()
        .inherit_args()
        .build_p1();

    let mut store = Store::new(&engine, wasi);

    let instance = linker.instantiate(&mut store, &module).unwrap();
    let start = instance
        .get_typed_func::<(), ()>(&mut store, "_start")
        .unwrap();

    // Handle WASI exit properly
    match start.call(&mut store, ()) {
        Ok(_) => println!("WASM completed successfully"),
        Err(e) => {
            if let Some(exit_code) = e.downcast_ref::<wasmtime_wasi::I32Exit>() {
                println!("WASM exited with code: {}", exit_code.0);
            } else {
                panic!("WASM error: {}", e);
            }
        },
    }

    let test_multiply = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, "multiply")
        .unwrap();

    let result = test_multiply.call(&mut store, (42, 2));
    println!("Result of multiplication: {}", result.unwrap());
}
