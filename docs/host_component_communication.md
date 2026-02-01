# Host-Component Communication

Components can interact with shared game state and communicate with each other through the host.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Main Thread                         │
│  ┌─────────────┐                                        │
│  │ GameState   │  entities: HashMap<u64, Entity>        │
│  └─────────────┘                                        │
│         │ passed to WASI thread                         │
│         ↓                                               │
│  ┌─────────────────────────────────────────────────┐    │
│  │              WASI Thread                        │    │
│  │                                                 │    │
│  │  ┌─────────┐              ┌─────────┐          │    │
│  │  │ Client  │←─── RPC ────→│ Server  │          │    │
│  │  │ (Rust)  │              │  (Go)   │          │    │
│  │  └─────────┘              └─────────┘          │    │
│  │       │                        │               │    │
│  │       └──── Host Functions ────┘               │    │
│  │            (get-entities, spawn-entity)        │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

## WIT Interface

```wit
package wasi:playground@0.1.0;

interface host {
    log: func(msg: string);
    get-entities: func() -> string;
    spawn-entity: func(name: string, x: f32, y: f32) -> u64;
    rpc-call: func(target: string, method: string, args: string) -> u64;
}

world playground {
    import host;
    
    export process: func(input: string) -> string;
    export on-rpc-request: func(caller: string, method: string, args: string) -> string;
    export on-rpc-response: func(request-id: u64, response: string);
    export matrix-bench: func(size: u32, iterations: u32) -> u64;
}
```

## Host Functions

### `log(msg: string)`
Print a message to console with component name prefix.

### `get-entities() -> string`
Returns all entities as semicolon-separated string: `id:name:x,y;id:name:x,y;...`

### `spawn-entity(name, x, y) -> u64`
Creates new entity, returns its ID.

### `rpc-call(target, method, args) -> u64`
Sends RPC request to another component. Returns request ID.
Response delivered via `on-rpc-response` callback after current tick processing.

## RPC Flow

1. Component A calls `rpc-call("component_b", "method", "args")`
2. Host queues the request, returns request ID
3. After all components finish their `process()` call, host processes RPC queue
4. Host calls Component B's `on-rpc-request("component_a", "method", "args")`
5. Component B can call host functions (spawn-entity, etc.) and returns result
6. Host calls Component A's `on-rpc-response(request_id, result)`

## Examples

### Client Spawning via Server (Tick 1)

```
Client.process("tick:1"):
  - calls get-entities() → sees 3 entities
  - calls rpc-call("server", "ready_to_spawn", "player1") → req_id=1
  - returns

Host processes RPC queue:
  - calls Server.on-rpc-request("client", "ready_to_spawn", "player1")
  - Server calls spawn-entity("player1", 50, 50) → id=4
  - Server returns "ok:4"

Host delivers response:
  - calls Client.on-rpc-response(1, "ok:4")
  - Client calls get-entities() → sees 4 entities (including itself)
```

### Server Triggering Client Benchmark (Tick 4)

```
Server.process("tick:4"):
  - runs own benchmark (Go: ~1.45 GFLOPS)
  - calls rpc-call("client", "run_benchmark", "")
  - returns

Host processes RPC queue:
  - calls Client.on-rpc-request("server", "run_benchmark", "")
  - Client runs benchmark (Rust: ~1.60 GFLOPS)
  - Client returns "ok"

Host delivers response:
  - calls Server.on-rpc-response(2, "ok")
```

## Limitations

- RPC is callback-based, not async/await
- All RPC within same tick is processed sequentially
- Component targeting is by name string (no service discovery yet)
