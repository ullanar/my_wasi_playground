# How to Write a Go WASI Component

## **Prerequisites**

Need to have: tinygo, wasm-tools, wkg, wasmtime

---

## **Quick Start: Simple Component**

### **1. Create Project**
```bash
mkdir my-component && cd my-component
go mod init my-component
```

### **2. Write Code**
```go
// main.go
package main

import "fmt"

func main() {
    fmt.Println("Hello from WASI!")
}
```

### **3. Build**
```bash
tinygo build -target=wasip2 -o app.wasm main.go
```

### **4. Run**
```bash
wasmtime app.wasm
```

**Done!** ✅

---

## **Advanced: Custom WIT Interface**

### **Project Structure**
```
my-component/
├── go.mod
├── tools.go          # Tool dependencies
├── main.go
├── wit/
│   └── world.wit     # WIT definition
└── internal/         # Generated bindings (gitignored)
```

### **Step 1: Setup go.mod**
```bash
go mod init my-component

# Add wit-bindgen-go as tool dependency
cat > tools.go << 'EOF'
//go:build tools

package main

import (
    _ "go.bytecodealliance.org/cmd/wit-bindgen-go"
)
EOF

go get go.bytecodealliance.org/cmd/wit-bindgen-go
go mod tidy
```

### **Step 2: Create WIT Definition**
```bash
mkdir -p wit

cat > wit/world.wit << 'EOF'
package myapp:component@0.1.0;

interface host {
    log: func(msg: string);
}

world my-world {
    // REQUIRED for TinyGo wasip2
    include wasi:cli/imports@0.2.0;
    
    import host;
    export process: func(input: string) -> string;
}
EOF
```

**⚠️ Critical**: Always include `wasi:cli/imports@0.2.0` for TinyGo wasip2!

### **Step 3: Resolve Dependencies (if using includes)**
```bash
# If using external deps like wasi:cli
cd wit
wkg wit build
cd ..

# Creates: myapp:component@0.1.0.wasm
```

### **Step 4: Generate Go Bindings**
```bash
go run go.bytecodealliance.org/cmd/wit-bindgen-go generate \
  --world my-world \
  --out internal \
  myapp:component@0.1.0.wasm
```

### **Step 5: Implement Component**
```go
// main.go
package main

import (
    "my-component/internal/myapp/component/host"
)

// Implement exported function
//export process
func process(input string) string {
    // Call imported host function
    host.Log("Processing: " + input)
    return "Processed: " + input
}

func main() {}
```

**Note**: `main()` must exist but stays empty for wasip2 target.

### **Step 6: Build**
```bash
tinygo build \
  -target=wasip2 \
  --wit-package ./myapp:component@0.1.0.wasm \
  --wit-world my-world \
  -o component.wasm \
  main.go
```

**For production** (smaller binary):
```bash
tinygo build \
  -target=wasip2 \
  --wit-package ./myapp:component@0.1.0.wasm \
  --wit-world my-world \
  -no-debug \
  -opt=2 \
  -o component.wasm \
  main.go
```

---

## **HTTP Server Example**

### **Using wasmCloud SDK**
```bash
go get go.wasmcloud.dev/component
```

```go
//go:generate go run go.bytecodealliance.org/cmd/wit-bindgen-go generate --world hello --out gen ./wit

package main

import (
    "net/http"
    "go.wasmcloud.dev/component/net/wasihttp"
)

func init() {
    wasihttp.HandleFunc(handler)
}

func handler(w http.ResponseWriter, r *http.Request) {
    w.Write([]byte("Hello from WASI!"))
}

func main() {}
```

**Build & Run**:
```bash
go generate
tinygo build -target=wasip2 -o server.wasm main.go
wasmtime serve server.wasm
```

---

## **Common Patterns**

### **Exporting Functions**
```go
//export functionName
func functionName(param string) string {
    return "result"
}
```

### **Importing Host Functions**
```go
import "my-component/internal/namespace/package"

// Use generated bindings
namespace.PackageFunctionName("argument")
```

### **Working with Complex Types**
```go
// WIT: record user { name: string, age: u32 }
type User struct {
    Name string
    Age  uint32
}

//export getUser
func getUser() User {
    return User{Name: "Alice", Age: 30}
}
```

---

## **Best Practices**

✅ **Do**:
- Always include `wasi:cli/imports@0.2.0` in wasip2 worlds
- Use `go.mod` tool dependencies for `wit-bindgen-go`
- Gitignore `internal/` and `wit/deps/` directories
- Keep `main()` function (even if empty)
- Use `-no-debug -opt=2` for production builds

❌ **Don't**:
- Don't use `net/http` standard library (use wasihttp instead)
- Don't use `encoding/json` with reflection (limited support)
- Don't use standard Go compiler (use TinyGo)
- Don't forget `//export` directive

---

## **Troubleshooting**

**"failed to resolve import"**:
- Add `include wasi:cli/imports@0.2.0` to WIT world
- Run `wkg wit build` to bundle dependencies

**"undefined: encoding/json"**:
- TinyGo has limited reflection support
- Use `github.com/mailru/easyjson` or similar alternatives

**Binary too large**:
- Add `-no-debug -opt=2` flags
- Remove unused imports

---

## **Useful Links**

**Official Documentation**:
- Component Model Go Guide: https://component-model.bytecodealliance.org/language-support/building-a-simple-component/go.html
- TinyGo Installation: https://tinygo.org/getting-started/install/

**Tools & Repos**:
- wasm-tools: https://github.com/bytecodealliance/wasm-tools
- wit-bindgen-go: https://github.com/bytecodealliance/go-modules
- wkg: https://github.com/bytecodealliance/wasm-pkg-tools
- wasmCloud Go SDK: https://github.com/wasmCloud/go

**Community Resources**:
- wasmCloud Go Docs: https://wasmcloud.com/docs/developer/languages/go/components/
- Component Model Tutorial: https://component-model.bytecodealliance.org/tutorial.html
- Bytecode Alliance Blog: https://bytecodealliance.org/articles

---

## **Quick Reference**

```bash
# Simple build
tinygo build -target=wasip2 -o app.wasm main.go

# With custom WIT
tinygo build \
  -target=wasip2 \
  --wit-package ./wit/pkg@0.1.0.wasm \
  --wit-world world-name \
  -o app.wasm \
  main.go

# Generate bindings
go run go.bytecodealliance.org/cmd/wit-bindgen-go generate \
  --world world-name \
  --out internal \
  ./wit/pkg@0.1.0.wasm

# Bundle WIT
wkg wit build

# Inspect component
wasm-tools component wit app.wasm

# Run
wasmtime app.wasm
```
