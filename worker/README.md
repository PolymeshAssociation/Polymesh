# Polymesh Worker

`polymesh-worker` is a high-performance protocol execution engine for the Polymesh blockchain. It provides a flexible, multi-backend system for executing protocol-specific work requests (cryptographic proofs, verifications, and computations) outside the runtime, enabling fast transaction processing while maintaining security and correctness.

## Overview

The polymesh-worker crate serves as the core execution engine that:

- **Executes protocol-specific work** (e.g., confidential asset verification, proof generation for benchmarks/testing)
- **Manages multiple execution backends** (PolkaVM, Wasmtime, Wasmer, Native)
- **Caches compiled protocol modules** for fast reuse across work requests
- **Caches work request results** to avoid redundant computation
- **Integrates with Substrate** via a native runtime interface
- **Supports multiple protocol versions** with flexible initialization strategies

## Architecture

### Design Overview

```
┌─────────────────────────────────────────────────────────────┐
│              Substrate Runtime (polymesh-runtime)           │
├─────────────────────────────────────────────────────────────┤
│              Substrate Extension Interface                  │
│        (NativePolymeshWorker trait, runtime_interface)      │
├─────────────────────────────────────────────────────────────┤
│                    PolymeshWorker                           │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  WorkerSession (per-block session management)        │   │
│  │  - Tracks pending work requests and results          │   │
│  │  - Manages request->response mapping                 │   │
│  │  - Thread pool for parallel work execution           │   │
│  └──────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    Cache Layer                              │
│  ┌──────────────────┐  ┌──────────────────────────────┐     │
│  │ Module Cache     │  │ Work Request Result Cache    │     │
│  │ - Per-protocol   │  │ - LRU (10,000 entries)       │     │
│  │ - Per-backend    │  │ - Keyed by request hash      │     │
│  │ - 32 instances   │  │ - Avoids recomputation       │     │
│  │   per protocol   │  │                              │     │
│  └──────────────────┘  └──────────────────────────────┘     │
├─────────────────────────────────────────────────────────────┤
│                    Backend Layer                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │   PolkaVM    │  │  Wasmtime    │  │   Wasmer     │  ...  │
│  │ (RISC-V VM)  │  │ (WASM VM)    │  │  (WASM VM)   │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
├─────────────────────────────────────────────────────────────┤
│              Compiled Protocol Modules                      │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  DART v1: polymesh-worker-protocol-dart-v1           │   │
│  │  Available as: PolkaVM, WASM (+ Native fallback)     │   │
│  │  Stored as: Compressed binary in crate (zst)         │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Backends

The worker supports multiple execution backends:

#### Native (Default)
- **Direct CPU execution** for maximum performance (can use rayon/threads)
- Can't be upgraded from the chain, so only available when the node binary has the same compatible version.

#### PolkaVM (Fastest VM backend)
- **RISC-V based execution engine** designed specifically for fast compilation (RISC-V -> native) and execute.
- **Deterministic and sandboxed** execution environment
- Supports upgradable protocol modules.  Managed by the chain runtime.
- Compiled target: `riscv64emac-unknown-none-polkavm`

#### Wasmer (Fastest Wasm backend when LLVM is enabled)
- **High-performance WASM runtime** with JIT compilation
- **Flexible compilation backends** (Cranelift, optional LLVM)
- Supports upgradable protocol modules.  Managed by the chain runtime.
- Requires: `wasm32-unknown-unknown` target

#### Wasmtime (Slowest backend, but faster then chain runtime)
- **WebAssembly runtime** with support for WASM v1 features
- **Widely adopted and stable** ecosystem
- Supports upgradable protocol modules.  Managed by the chain runtime.
- Requires: `wasm32-unknown-unknown` target

### Work Request Lifecycle

```
1. Submit Work Request
   ├─ Via session: session.submit_request(work_request)
   └─ Or directly: execute_request(work_request)

2. Cache Lookup (optional)
   ├─ Compute request hash
   └─ Check work result cache
       └─ Hit: Return cached result immediately
       └─ Miss: Proceed to execution

3. Load Protocol Module
   ├─ Determine protocol and version
   ├─ Select backend (from bitmask)
   ├─ Load module from cache
   └─ Or load and compile from module definition

4. Create Module Instance
   ├─ Get from instance pool or create new
   ├─ Allocate scratch space
   └─ Initialize module (if needed)

5. Execute Work
   ├─ Call module's execute function
   ├─ Provide work request and seed
   └─ Receive response (result or error)

6. Cache Result (optional)
   ├─ If request is cacheable (has hash)
   └─ Store in work result cache

7. Return Response
   ├─ Via session: response stored, retrieved via polling
   └─ Or direct: returned immediately
```

### Multi-Protocol Support

The worker is designed to support multiple protocol implementations:

- **Protocol Definition**: Each protocol has an ID and version (e.g., PROTOCOL_PDART = v0.1.0)
- **Module Configuration**: Specifies which backends have implementations and their code hashes
- **Initialization Strategy**: Flexible module initialization supporting:
  - `NoInitializationNeeded`: Module ready to use immediately
  - `InitializeNoContext`: Call init function with empty context
  - `SaveContextFromFirstInstance`: Optimize by saving context after first init
  - `ContextData(Vec<u8>)`: Provide explicit context data
  - `ContextHash(Hash)`: Load context from cache/storage

- **Protocol Loaders**: Implement `BackendModuleLoader` trait to provide:
  - Module code bytes for a given protocol and backend
  - Protocol module configuration and hashes
  - Context data for module initialization

## Caching System

### Module Cache

The module cache manages compiled protocol modules per backend:

**Features:**
- **Per-protocol isolation**: Each protocol has separate module instances
- **Per-backend support**: Different backends maintain separate instances
- **Instance pooling**: Up to 32 instances per protocol available in the pool
- **Automatic cleanup**: Instances returned to pool when dropped
- **Lazy initialization**: Modules loaded on-demand

### Work Request Cache

An LRU cache for work request responses:

**Features:**
- **Automatic deduplication**: Identical requests reuse cached results
- **Configurable capacity**: Can be tuned based on memory constraints
- **Thread-safe**: Protected by RwLock for concurrent access
- **Optional**: Can be disabled per session for certain workloads

## Work Sessions

Work sessions provide efficient batching and management of work requests during block processing:

### Session Lifecycle

```rust
// Start at block beginning
let session_id = worker.start_session(flags_and_backends, protocol)?;

// Submit multiple requests in parallel
for request in requests {
    let id = worker.submit_request(session_id, request)?;
    pending_ids.push(id);
}

// Poll for results
for id in pending_ids {
    if let Some(result) = worker.get_response(session_id, id) {
        // Process result
    }
}

// Session auto-cleans up or explicit close
```

### Thread Pool

- **Parallel execution**: Multiple requests processed concurrently
- **Powered by rayon**: Work-stealing thread pool for balanced distribution
- **Per-session coordination**: Each session maintains its own pending work tracking
- **Response channels**: Crossbeam channels for thread-safe result delivery

## Protocol Implementation: DART v1

DART is the first protocol implemented in this worker:

### Module Variants

Two variants are compiled and distributed:

1. **Production**: `polymesh-worker-protocol-dart-v1.{polkavm,wasm}.zst`
   - Optimized for performance
   - Compressed for efficient distribution

2. **Testing**: `polymesh-worker-protocol-dart-v1.testing.{polkavm,wasm}.zst`
   - Includes proof generation support.
   - Used to generate proofs needed when benchmarking proof verification.

### Supported Operations

- **Account asset registration proof verification**
- **Batch registration proof handling**
- **Sender/Receiver affirmation proof verification**
- **Confidential asset verification**
- **Curve-tree operations**

## Tools and Utilities

### Tester (`tester/`)

This is for testing/benchmarking the different worker backends:

```bash
# Build the tester
cargo build -p polymesh-worker-tester

# Run with specific backend
./target/release/polymesh-worker-tester native
./target/release/polymesh-worker-tester polkavm
./target/release/polymesh-worker-tester wasmer
./target/release/polymesh-worker-tester wasmtime
```

**Benchmarks included:**
- `bench_backends.rs`: Benchmark all backends for execution performance
- `bench_msm.rs`: Multi-scalar multiplication performance testing
- Protocol-specific work request testing

**Purpose:**
- Validate backend implementations
- Measure performance across different backends
- Test protocol module compilation and execution
- Compare backend performance characteristics

### CLI Tools (`tools/`)

Protocol module management and utilities:

```bash
# Compress a module
cargo run -p polymesh-worker-tools -- compress <module_path>

# This is used in build scripts to generate .zst files
# Enables efficient distribution of compiled modules
```

**Features:**
- **Module compression**: Zstandard compression for distribution
- **Module inspection**: Analyze compiled module metadata
- **Code hash generation**: Compute Blake2b-256 hashes for verification

### Build Scripts

Automated compilation of protocol modules:

```bash
# Build PolkaVM module
./build_polkavm.sh
# Generates: polymesh-worker-protocol-dart-v1.polkavm.zst

# Build WASM module
./build_wasm.sh
# Generates: polymesh-worker-protocol-dart-v1.wasm.zst

# Build testing variants
./build_polkavm_testing.sh
./build_wasm_testing.sh

# Full rebuild of both PolkaVM/WASM modules and the testing variants.
./rebuild.sh
```

**Process:**
1. Compile protocol crate to backend target
2. Link or process module (backend-specific)
3. Compress with zstandard
4. Output ready for distribution

## Substrate Extension

The Substrate extension (`extension/`) provides a native runtime interface for the Polymesh runtime:

### Runtime Interface Definition

```rust
#[runtime_interface]
pub trait NativePolymeshWorker {
    /// Worker version for compatibility checking
    fn worker_version() -> WorkerVersion;
    
    /// Execute single request without session
    fn execute_request(
        protocol: ProtocolNumber,
        backends: BackendBitmask,
        request: Vec<u8>,
    ) -> Result<WorkResponseResult, WorkerError>;
    
    /// Start a new session.  This is normally done at the start of a block.
    fn start_session(
        flags_and_backends: WorkerConfigFlagsAndBackends,
        default_protocol: ProtocolNumber,
    ) -> WorkerSessionId;

    /// End the session with the given session id.  This is normally done at the end of a block.
    fn end_session(session_id: WorkerSessionId);
    
    /// Execute a protocol-specific work request for the given session id.
    fn session_execute_request(
        session_id: WorkerSessionId,
        flags: WorkerConfigFlags,
        request: Vec<u8>,
    ) -> WorkStatusFlagsAndId;
    
    /// Get the request result for the given session id and request id.
    fn session_get_results(
        session_id: WorkerSessionId,
        request_id: WorkRequestId,
    ) -> Result<WorkResponseResult, WorkerError>;
}
```

### Protocol Module Loading

Support for custom module loaders via the `BackendModuleLoader` trait, to support loading config/modules from
chain storage (allowing the runtime to manage upgrading protocols).

Hashes are used to support caching protocol config and modules.  The hashes are also used to prevent one malicious
validator from producing a block that would taint the cache of other nodes in the network ahead of a protocol upgrade.

When loading a protocol the config hash is always loaded from the chain storage of the current block to ensure the
correct modules are used (either from cache or loaded from chain storage).

```rust
pub trait BackendModuleLoader {
    /// Try loading the protocol module config hash for the given protocol.
    ///
    /// The config hash can be used to check if the protocol module has been updated and needs to be reloaded.
    fn get_protocol_module_config_hash(
        &mut self,
        protocol: Protocol,
    ) -> Option<ProtocolModuleConfigHash>;

    /// Try loading the prtocol module config for the given protocol and config hash.
    fn get_protocol_module_config(
        &mut self,
        protocol: Protocol,
        config_hash: ProtocolModuleConfigHash,
    ) -> Option<ProtocolModuleConfig>;

    /// Try loading a module for the given protocol.
    fn get_module_code_bytes(
        &mut self,
        protocol: Protocol,
        kind: BackendModuleKind,
        code_hash: BackendCodeHash,
    ) -> Option<Vec<u8>>;

    /// Try loading the protocol context for the given protocol and context hash.
    fn get_module_context_bytes(
        &mut self,
        protocol: Protocol,
        ctx_hash: BackendContextHash,
    ) -> Option<Vec<u8>>;

}
```

**Built-in implementations:**

- **StaticModules**: Loads DART v1 protocol from embedded binaries
- **SubstrateModuleLoader**: Loads modules from Substrate storage

## Features

- **`polkavm`** (default): Enable PolkaVM backend support
- **`wasmtime`** (default): Enable Wasmtime backend support
- **`wasmer`** (default): Enable Wasmer backend support
- **`testing`** (default): Enable testing protocol variants (proof generation support)
- **`debug_logging`**: Verbose logging for debugging
- **`asm`**: Assembly optimizations (enabled with `std`)
- **`parallel`**: Parallel MSM operations (enabled with `std`)
- **`std`**: Standard library support (default)

## Building and Compilation

### Dependencies

- **Rust toolchain** with support for multiple targets:
  - `x86_64-unknown-linux-gnu` (host)
  - `riscv64emac-unknown-none-polkavm` (PolkaVM)
  - `wasm32-unknown-unknown` (WASM)

- **Tools:**
  - `polkatool`: PolkaVM linking tool

### Building Protocol Modules

```bash
# From worker directory
cd Polymesh/worker

# Build all modules
./rebuild.sh

# Build specific backend modules
./build_polkavm.sh
./build_wasm.sh
```

## Performance Considerations

### Module Caching

- Modules are kept in memory after first load for fast reuse
- Limited to 32 instances per protocol to bound memory usage
- Instances are pooled and reused within the pool

### Work Result Caching

- 10,000 entry LRU cache for work results
- Deduplicates identical work requests
- Significantly improves throughput for repeated verifications

### Thread Pool

- Rayon-based work-stealing pool for load balancing
- Enables parallel execution across CPU cores
- Scales automatically based on available cores

## Development and Testing

### Running Tests

```bash
# Run the tester
cargo run -p polymesh-worker-tester -- <backend>
```

### Debug Logging

Enable debug logging for troubleshooting:

```bash
RUST_LOG=debug cargo run -p polymesh-worker-tester -- polkavm
```

## License

See [LICENSE](../LICENSE) file at repository root.
