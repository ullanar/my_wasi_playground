mod wasi;

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

pub enum Command {
    Tick(u64),
    Benchmark,
}

fn main() {
    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
    let (done_tx, done_rx) = mpsc::channel::<()>();

    let wasi_handle = thread::spawn(move || {
        wasi::run(cmd_rx, done_tx);
    });

    let tick_rate = Duration::from_secs(3);
    let total_ticks = 5;

    println!(
        "[MAIN] Starting game loop ({}s tick, {} ticks)",
        tick_rate.as_secs(),
        total_ticks
    );

    for tick in 1..=total_ticks {
        let start = Instant::now();

        cmd_tx.send(Command::Tick(tick)).unwrap();
        done_rx.recv().unwrap();

        if tick == total_ticks {
            cmd_tx.send(Command::Benchmark).unwrap();
            done_rx.recv().unwrap();
        }

        let elapsed = start.elapsed();
        if elapsed < tick_rate {
            thread::sleep(tick_rate - elapsed);
        }
    }

    drop(cmd_tx);
    wasi_handle.join().unwrap();

    println!("[MAIN] Done");
}
