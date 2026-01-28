# My WASI Playground

## What is this

Here you can see (probably you don't want to) my playground for testing WASM/WASI integration with a Rust application.

> Important note: current state of WASI is preview 2 (WASI Components). When I will be able to work on p3/stable release then all of that I hope will be updated.

Basically, we have a Rust application which serves as the main entry point. It's meant to be run as a normal executable. Then it imports WASI components and runs them.

*EXPECT BUNCH OF UNWRAPS IN RUST CODE FOR NOW. THERE IS YET (!) NO TIME FOR IT - WILL HANDLE THEM ALL LATER*

I want to understand several HowTo things:
- Import and run WASI components from a Rust application
- Interact with WASI components from the Rust application
- Interact with the Rust application from WASI components
- Interact between several WASI components (?) probably via the core app
- Async
- Profile WASM [actually here some docs](https://docs.wasmtime.dev/examples-profiling.html)
- Compare WASM heavy math performance with native languages performance

And I want to try several languages for WASI components.

Also, it would be great if I could run one sync thread of the core Rust app + one thread with async WASI and understand how I can better communicate between the sync thread and WASI components.

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
    b target='go'   # Build WASI components (go or all)
    build-all
    build-go
    build-host      # Rust host utilities
    clean
    run
    wkg target='go' # Bundle WIT dependencies (go or all)
    wkg-all
    wkg-go
```


To run Rust core executable you anyway need firstly build all WASI components.

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

## [Docs with my observations](docs/README.md)


## EXTREMELY (!) useful links
There are just so many possibilities and such a small documentation. So it is a blessing to find something like this:
- [Component model by Bytecode Alliance](https://component-model.bytecodealliance.org/introduction.html)
- [WasmCloud tools and docs](https://wasmcloud.com/docs/intro/) (CNCF btw!)
- [Wasmtime Rust docs](https://docs.wasmtime.dev/)
