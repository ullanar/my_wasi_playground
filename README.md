# My WASI Playground

## What is this

Here you can see (probably you don't want to) my playground for testing WASM/WASI integration with a Rust application.

Basically, we have a Rust application which serves as the main entry point. It's meant to be run as a normal executable. Then it imports WASI modules and runs them.

*EXPECT BUNCH OF UNWRAPS IN RUST CODE FOR NOW. THERE IS YET (!) NO TIME FOR IT - WILL HANDLE THEM ALL LATER*

I want to understand several HowTo things:
- Import and run WASI modules from a Rust application
- Interact with WASI modules from the Rust application
- Interact with the Rust application from WASI modules
- Interact between several WASI modules (?) probably via the core app
- Async

And I want to try several languages for WASI modules.

Also, it would be great if I could run one sync thread of the core Rust app + one thread with async WASI and understand how I can better communicate between the sync thread and WASI apps.

## WHY IS IT EVEN OPEN SOURCE

Because:
1. It's fun
2. It forces me to be a bit more careful with my code
3. Probably someone will find it useful
4. Probably someone would like to help me with better practices


# HOW TO RUN

Firstly u either need to have `devbox` installed and then u can just `devbox shell` OR u need to install all deps specified in `./devbox.json`

Then u can simply use `just b go` and then `just run` OR u can read `./justfile` and call commands manually

Availiable just commands (hope I will not forget to update it)
```shell
just -l
Available recipes:
    b module   # Build a module by name, or "all" for all modules
    build-host # Build the Rust host
    clean      # Clean build artifacts
    run        # Run the host
```


To run Rust core executable you anyway need firstly build all WASI modules.

So

```shell
devbox shell
```

```shell
just b all
```

```shell
just run
```

### If you have any questions or improvements, feel free to open an issue
