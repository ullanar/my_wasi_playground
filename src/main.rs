mod wasi;

use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

// Commands: Main -> WASI
#[derive(Debug, Clone)]
pub enum Command {
    Tick(u64),
}

// Events: WASI -> Main
#[derive(Debug)]
pub enum Event {
    GetEntities,
    SpawnEntity { name: String, x: f32, y: f32 },
    TickDone,
}

// Responses: Main -> WASI (for queries)
#[derive(Debug)]
pub enum Response {
    Entities(String),
    Spawned(u64),
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: u64,
    pub name: String,
    pub x: f32,
    pub y: f32,
}

pub struct GameState {
    pub entities: HashMap<u64, Entity>,
    next_id: u64,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn spawn(&mut self, name: String, x: f32, y: f32) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.entities.insert(id, Entity { id, name, x, y });
        id
    }

    pub fn serialize(&self) -> String {
        self.entities
            .values()
            .map(|e| format!("{}:{}:{},{}", e.id, e.name, e.x, e.y))
            .collect::<Vec<_>>()
            .join(";")
    }
}

fn main() {
    let mut game_state = GameState::new();
    game_state.spawn("tree".into(), 10.0, 20.0);
    game_state.spawn("rock".into(), 15.0, 25.0);
    let npc_id = game_state.spawn("npc_merchant".into(), 100.0, 100.0);

    println!("[MAIN] Initial entities:");
    for e in game_state.entities.values() {
        println!("  {} (id={}): ({}, {})", e.name, e.id, e.x, e.y);
    }

    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
    let (event_tx, event_rx) = mpsc::channel::<Event>();
    let (resp_tx, resp_rx) = mpsc::channel::<Response>();

    let wasi_handle = thread::spawn(move || {
        wasi::run(cmd_rx, event_tx, resp_rx);
    });

    let tick_rate = Duration::from_secs(1);
    let total_ticks = 5;

    println!("\n[MAIN] Starting game loop\n");

    for tick in 1..=total_ticks {
        let start = Instant::now();

        // Move NPC to random position each tick
        if let Some(npc) = game_state.entities.get_mut(&npc_id) {
            npc.x = (tick as f32 * 10.0) % 200.0;
            npc.y = (tick as f32 * 15.0) % 200.0;
            println!("[MAIN] Moved npc_merchant to ({}, {})", npc.x, npc.y);
        }

        // Send tick command
        cmd_tx.send(Command::Tick(tick)).unwrap();

        // Process events from WASI until tick is done
        loop {
            match event_rx.recv().unwrap() {
                Event::GetEntities => {
                    let data = game_state.serialize();
                    resp_tx.send(Response::Entities(data)).unwrap();
                }
                Event::SpawnEntity { name, x, y } => {
                    let id = game_state.spawn(name.clone(), x, y);
                    println!("[MAIN] Spawned '{}' with id={}", name, id);
                    resp_tx.send(Response::Spawned(id)).unwrap();
                }
                Event::TickDone => break,
            }
        }

        let elapsed = start.elapsed();
        if elapsed < tick_rate {
            thread::sleep(tick_rate - elapsed);
        }
    }

    drop(cmd_tx);
    wasi_handle.join().unwrap();

    println!("\n[MAIN] Final entities:");
    for e in game_state.entities.values() {
        println!("  {} (id={}): ({}, {})", e.name, e.id, e.x, e.y);
    }

    println!("\n[MAIN] Done");
}
