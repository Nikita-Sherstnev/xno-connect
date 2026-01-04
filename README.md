# XNO-connect

This library provides an interface to the Nano network through RPC and WebSocket APIs, all methods are async. It also supports WASM and local work generation. Contributions/suggestions are welcome.

**The library is in active development and not yet fully functional and tested.**

## Feature Flags

**default**: RPC and WebSocket support, usable on backend side of application. Uses reqwest and tokio runtime.

**rpc**: Enable RPC functionality

**websocket**: Enable WebSocket functionality

**websocket-tls**: Enable WebSocket with TLS support

**work-cpu**: Enable local CPU-based work generation, uses rayon

**full**: Enable all native features


**wasm-rpc**: Enable RPC for WebAssembly

**wasm-websocket**: Enable WebSocket for WebAssembly

**wasm-full**: Enable all WASM features


## Development

Look what variables are needed to run tests in .env.example file.

Running all tests, except for WASM.

```bash
cargo test --features full --release -- --include-ignored --no-capture
```

Run only some test.

```bash
cargo test --features full --release test_send_and_change_representative -- --ignored --no-capture
```

Run only integration tests.

```bash
cargo test --features full --release --test '*' -- --ignored --no-capture
```

Run WASM tests

```bash
source .env && NANO_RPC_URL="$NANO_RPC_URL" NANO_WS_URL="$NANO_WS_URL" NANO_SEED="$NANO_SEED" NANO_DESTINATION="$NANO_DESTINATION" NANO_REPRESENTATIVE="$NANO_REPRESENTATIVE" wasm-pack test --headless --firefox --features wasm-full
```

Test coverage

```bash
cargo llvm-cov --features full --release --html -- --include-ignored
```
