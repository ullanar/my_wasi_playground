wit_bindgen::generate!({
    path: "wit",
    generate_all,
});

use std::cell::Cell;
use std::time::Instant;
use wasi::playground::host;

thread_local! {
    static SPAWN_REQUEST_ID: Cell<Option<u64>> = Cell::new(None);
}

struct Component;

impl Guest for Component {
    fn process(input: String) {
        host::log(&format!("Processing: {}", input));

        if input == "tick:1" {
            let entities = host::get_entities();
            host::log(&format!("Current entities: {}", entities));

            let req_id = host::rpc_call("server", "ready_to_spawn", "player1");
            host::log(&format!("Requested spawn, req_id={}", req_id));
            SPAWN_REQUEST_ID.with(|id| id.set(Some(req_id)));
        }
    }

    fn on_rpc_request(caller: String, method: String, args: String) -> String {
        host::log(&format!("RPC from {}: {}({})", caller, method, args));

        match method.as_str() {
            "ping" => format!("pong:{}", args),
            "run_benchmark" => {
                host::log("Running client benchmark...");
                run_benchmark();
                "ok".into()
            }
            _ => "error:unknown_method".into(),
        }
    }

    fn on_rpc_response(request_id: u64, response: String) {
        host::log(&format!("RPC response {}: {}", request_id, response));

        let expected_id = SPAWN_REQUEST_ID.with(|id| id.get());
        if expected_id == Some(request_id) {
            host::log("Spawn confirmed! Checking entities...");
            let entities = host::get_entities();
            host::log(&format!("Entities after spawn: {}", entities));
        }
    }

    fn matrix_bench(size: u32, iterations: u32) -> u64 {
        let n = size as usize;
        let iters = iterations as usize;

        let mut a: Vec<f64> = Vec::with_capacity(n * n);
        let mut b: Vec<f64> = Vec::with_capacity(n * n);
        let mut c: Vec<f64> = vec![0.0; n * n];

        for i in 0..(n * n) {
            a.push((i % 17) as f64 + 0.5);
            b.push((i % 13) as f64 + 0.5);
        }

        let start = Instant::now();
        for _ in 0..iters {
            matrix_multiply(&a, &b, &mut c, n);
            std::mem::swap(&mut a, &mut c);
        }
        let elapsed = start.elapsed();

        elapsed.as_nanos() as u64
    }
}

fn run_benchmark() {
    let configs = [(64u32, 100u32), (128, 50), (256, 10)];

    for (size, iterations) in configs {
        let nanos = Component::matrix_bench(size, iterations);
        let ms = nanos as f64 / 1_000_000.0;
        let ops = (size as u64).pow(3) * 2 * iterations as u64;
        let gflops = ops as f64 / (nanos as f64 / 1e9) / 1e9;
        host::log(&format!(
            "  {}x{} x{}: {:.1}ms, {:.2} GFLOPS",
            size, size, iterations, ms, gflops
        ));
    }
}

fn matrix_multiply(a: &[f64], b: &[f64], c: &mut [f64], n: usize) {
    for i in 0..n {
        for j in 0..n {
            let mut sum = 0.0;
            for k in 0..n {
                sum += a[i * n + k] * b[k * n + j];
            }
            c[i * n + j] = sum;
        }
    }
}

export!(Component);
