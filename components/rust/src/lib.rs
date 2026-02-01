wit_bindgen::generate!({
    path: "wit",
    generate_all,
});

use std::time::Instant;
use wasi::playground::host;

struct Component;

impl Guest for Component {
    fn process(input: String) -> String {
        host::log(&format!("Processing: {}", input));
        format!("Processed: {}", input)
    }

    fn matrix_bench(size: u32, iterations: u32) -> u64 {
        let n = size as usize;
        let iters = iterations as usize;

        host::log(&format!(
            "Starting benchmark: {}x{} matrix, {} iterations",
            n, n, iters
        ));

        // Allocate matrices
        let mut a: Vec<f64> = Vec::with_capacity(n * n);
        let mut b: Vec<f64> = Vec::with_capacity(n * n);
        let mut c: Vec<f64> = vec![0.0; n * n];

        // Initialize with deterministic values (same as Go)
        for i in 0..(n * n) {
            a.push((i % 17) as f64 + 0.5);
            b.push((i % 13) as f64 + 0.5);
        }

        // Run benchmark
        let start = Instant::now();
        for _ in 0..iters {
            matrix_multiply(&a, &b, &mut c, n);
            // Swap a and c to use result as next input (prevents dead code elimination)
            std::mem::swap(&mut a, &mut c);
        }
        let elapsed = start.elapsed();

        // Use result to prevent optimization
        let checksum = a[0] + a[n * n - 1];
        host::log(&format!("Benchmark done: checksum={}", checksum));

        elapsed.as_nanos() as u64
    }
}

/// Multiplies two NxN matrices: c = a * b
/// All matrices are flattened row-major
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
