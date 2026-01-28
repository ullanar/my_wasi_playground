# What is WIT and WKG

## **WIT** (WebAssembly Interface Types)

**An interface definition language for WebAssembly components**

- **What**: IDL (like TypeScript interfaces or Protobuf) for WebAssembly
- **Purpose**: Define component contracts - what functions import/export
- **Syntax**: Simple, readable text format (`.wit` files)
- **Key concepts**:
  - **Interfaces**: Groups of related functions
  - **Worlds**: Complete contract (imports + exports)
  - **Types**: Records, variants, enums, lists, etc.
  - **Packages**: Namespaced collections of interfaces

**Example**:
```wit
package myapp:calculator@1.0.0;

interface math {
    add: func(a: s32, b: s32) -> s32;
}

world calculator {
    import math;
    export calculate: func() -> s32;
}
```

**Think of it as**: TypeScript `.d.ts` files for WebAssembly

---

## **WKG** (WebAssembly Package Manager)

**Package manager for WIT definitions and WebAssembly components**

- **What**: Like `npm` for WebAssembly, manages WIT packages
- **Pronunciation**: "wackage"
- **Purpose**: Fetch, bundle, and publish WIT dependencies

**Key functions**:

1. **Fetch dependencies**:
   ```bash
   wkg wit fetch
   ```
   Downloads external WIT packages (e.g., `wasi:http`, `wasi:cli`)

2. **Bundle WIT into WASM**:
   ```bash
   wkg wit build
   ```
   Creates `namespace:package@version.wasm` with all dependencies

3. **Publish packages**:
   ```bash
   wkg publish myapp:api@1.0.0.wasm
   ```
   Shares your WIT interfaces via OCI registries

**Storage**: Uses OCI registries (GitHub Container Registry, Docker Hub, etc.)

**Think of it as**: `npm install` + `npm publish` for WebAssembly interfaces

---

## **How They Work Together**

```
1. Define API in WIT
   ↓
2. Bundle with wkg
   ↓
3. Publish to registry
   ↓
4. Others fetch and implement
```

**Example workflow**:
```bash
# Create WIT definition
cat > wit/api.wit << 'EOF'
package myapp:plugin@1.0.0;
world plugin {
    export process: func(data: string) -> string;
}
EOF

# Bundle with wkg
wkg wit build

# Publish
wkg publish myapp:plugin@1.0.0.wasm

# Others can now fetch it
wkg get myapp:plugin@1.0.0
```

---

## **Why This Matters**

- **Language-agnostic**: Write plugins in any language (Rust, Go, JS, C++)
- **Versioned contracts**: Semantic versioning for interfaces
- **Distributed easily**: Use existing OCI infrastructure
- **Type-safe**: Compile-time guarantees across languages
- **Plugin systems**: Build moddable apps where plugins are sandboxed WASM components

**Real-world use cases**:
- Plugin systems (like VS Code extensions)
- Microservices with different language components
- Shared libraries across language boundaries
- Game mods (implement in any language)
- Serverless functions with standard interfaces
