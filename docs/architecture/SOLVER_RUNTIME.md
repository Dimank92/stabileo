# One Engine Across Browser, Desktop and Server

Stabileo provides a single, unified structural mechanics engine (`dedaliano-engine`) written in
pure Rust that runs across three operational environments:
1. **Browser (WASM)**: Interactive 2D/3D visual modeling and live calculations directly in the client (`wasm32-unknown-unknown`).
2. **Desktop (Native CLI)**: Standalone high-performance execution without web dependencies via `stabileo-engine`.
3. **Server Daemon (HTTP RPC)**: Scalable backend compute service for large-scale building models, parametric sweeps, and cloud CI verification.

This document describes the runtime execution path, the large-model workflow, cross-platform determinism guarantees, and server endpoints (Issue #486, Roadmap: Solver performance and capability).

---

## 1. Architectural Principles

```
  ┌─────────────────────────────────────────────────────────────┐
  │                 Shared Rust Core (engine/)                   │
  │  - Direct Stiffness Assembly (2D/3D Frames, Plates, Shells)  │
  │  - Solvers: Linear, Modal, Buckling, P-Delta, Time-History  │
  │  - Verification: Certificates, Bounds, Sanity Diagnostics    │
  └──────────────────────────────┬──────────────────────────────┘
                                 │
         ┌───────────────────────┼────────────────────────┐
         ▼                       ▼                        ▼
  ┌──────────────┐       ┌───────────────┐       ┌─────────────────┐
  │ Browser WASM │       │ Desktop CLI   │       │ Server RPC      │
  │ (wasm-bindgen│       │ (Native x86_64│       │ (Daemon HTTP,   │
  │  in Web App) │       │  stabileo-eng)│       │  stabileo-eng)  │
  └──────────────┘       └───────────────┘       └─────────────────┘
```

1. **Zero Divergence**: Every calculation (stiffness matrix construction, degree-of-freedom ordering, factorization, back-substitution, and internal force extraction) is implemented in one single codebase. There are no separate JavaScript or Python implementations of solver mechanics.
2. **Bit-Identical Determinism**: The same model serialized as JSON yields identical output across platforms and builds down to the last bit, ensuring reproducible engineering deliverables and auditable certificates.
3. **No Silent Fallbacks**: If a platform runtime hits a limit, it returns an explicit diagnostic message or offloads to the server path rather than substituting an unconservative approximate method.

---

## 2. The Large-Model Execution Path

While WebAssembly executes within the browser at near-native speed for typical structural models (up to ~30,000 DOFs), real-world multi-storey building complexes, dense shell meshes, and non-linear dynamic time-history analyses require computing resources beyond browser sandbox limits:

- **Browser Limits**:
  - Memory ceiling: 4 GB WASM32 linear memory space.
  - Single-threaded main or web-worker thread execution without cross-origin isolation headers.
  - Browser watchdog timers that abort long-running computational tasks.

- **Large-Model Path**:
  When a structural model exceeds browser thresholds (or when explicitly requested by an engineer):
  1. **Payload Serialization**: The model is serialized as standard `SolverInput3D` JSON.
  2. **Transport**: The payload is dispatched to the native engine daemon (`stabileo-engine --server`) or spawned desktop worker via HTTP POST `/solve`.
  3. **Parallel Compute**:
     - Multi-threaded element stiffness assembly across CPU cores using Rayon.
     - 64-bit address space utilizing modern Sparse Cholesky, PCG with incomplete factor preconditioning, or eigensolvers without memory constraints.
  4. **Result Ingestion**: The resulting `AnalysisResults` JSON and Lean/interval certificate bundles are returned to the client and rendered in the interactive 3D viewport.

---

## 3. Cross-Platform Determinism

Structural engineering calculations must be deterministic. To guarantee that native and server builds match the browser WASM engine:
- **Ordered Collections**: Keyed inputs and DOF mappings iterate over deterministic hash and index orderings (`ordered.rs`), eliminating hash-seed variation.
- **Strict IEEE-754 Arithmetic**: All calculations run with consistent floating-point semantics. Subnormal flushing and rounding modes are kept consistent across native and WASM SIMD targets.
- **Shortest Round-Trip Serialization**: Results are encoded using `serde_json` with `float_roundtrip` enabled, ensuring that numbers deserialize to the exact IEEE-754 bit representation.
- **Automated Verification**:
  - `engine/tests/determinism.rs` verifies bit-identical outputs across independent random hash permutations.
  - `engine/tests/cross_platform_determinism.rs` enforces parity across native, server, and WASM pipelines.

---

## 4. Server Daemon & CLI Reference

The native and server engine binary is compiled from `engine/src/bin/stabileo_engine.rs`:

```bash
# Build the binary
cargo build --release --bin stabileo-engine
```

### CLI Mode
```bash
# Solve a 2D or 3D model from file or stdin
stabileo-engine --solve model.json

# Force 3D linear analysis
stabileo-engine --solve model.json --3d

# Run structural sanity checks
stabileo-engine --sanity model.json

# Print build info and commit metadata
stabileo-engine --build-info
```

### Server RPC Daemon Mode
```bash
# Start background RPC daemon on port 8088
stabileo-engine --server --port 8088
```

#### API Endpoints:
- `GET /health`: Health status, version, and commit information.
- `GET /build-info`: JSON build provenance metadata.
- `POST /solve`: Solves the submitted structural model payload (`Content-Type: application/json`) and returns `AnalysisResults` JSON with HTTP 200.
