# KRAKEN SOVEREIGN OS KERNEL (`krk-core-v2`)

This is the production-hardened, formally verified, closed-loop stochastic control system of the **Kraken Sovereign Operating System**.

It is implemented as a modular Rust workspace consisting of ~40 core crates working together to maintain topological stability and logical invariants of a distributed event Directed Acyclic Graph (DAG) under active adversarial conditions.

---

## 1. WORKSPACE CORE DESIGN

The OS modeling fits the mathematical archetype of an **adversarially-stressed, partially observed nonlinear control system**.

### System State space
$$x(t) = [\kappa(t), \Phi_\sim(t), \lambda_A(t), \sigma_C(t)]^T$$
- $\kappa(t)$: Graph connectivity sync density.
- $\Phi_\sim(t)$: Semantic equivalence class discovery rate.
- $\lambda_A(t)$: Adversarial innovation residual entropy.
- $\sigma_C(t)$: Topological equivalence collapse rate.

### Closed-Loop Control Loop
```
  [Telemetry Ingestion] ──► [krk-obs (Observability)]
                                    │
                                    ▼
  [krk-kernel (Mutation)] ◄── [krk-psc (Decision Engine)]
```

---

## 2. CRATE DECOMPOSITION

The monorepo contains ~43 modular Rust crates across six architectural layers:

### Core Controller & Observability
- **`krk-psc`**: **Decision Engine.** Phase-Stabilizing Controller v3 with dual-timescale observer and Kalman-style innovation residual test.
- **`krk-obs`**: Telemetry and metrics tracking panel ($\kappa, \Phi_\sim, \lambda_A$).
- **`krk-pressure`**: Monitors system collapse pressure and congestion bounds.
- **`krk-attention`**: Captures external signals (e.g. Digg Attention vectors) and maps them to control parameters.
- **`krk-calibration`**: Measures latency and drift variations between real-world performance and formal physical models.

### Control Plane & Agent Intelligence (NEW)
- **`krk-control-plane`**: **Policy arbitration brain** between PSC and kernel execution. Routes control signals through a prioritized policy stack. Prevents blind PSC drift, agent hallucination, and adversarial signal injection.
- **`krk-agent-swarm`**: **Multi-agent cognitive execution system.** Five specialized agents (Planner, Executor, Critic, Memory, Risk) with byzantine tolerance, PSC kill-switch, and confidence-gated quarantine.
- **`krk-memory-plane`**: **Dual-channel truth persistence** (Hermes + Memarch). Memarch = append-only raw capture; Hermes = canonical truth compression with 7-day pruning. Agents never read raw memory for decisions.

### Verification & Safety Gates
- **`krk-sat`**: Bridge to the Z3 SMT solver. Verifies state transitions using declarative SMT-LIB query templates.
- **`krk-ac`**: Admissibility Policy Engine. Gates transaction admission using four criteria: logical, financial, physical, and consensus.
- **`krk-safety-kernel`**: Projective algebraic operator that executes idempotent state transitions.
- **`krk-pbft`**: Cryptographic multi-signature consensus quorum verifier.
- **`krk-cegar`**: Counterexample-Guided Abstraction Refinement solver to resolve Z3 logical conflicts.
- **`krk-tee`**: Provisioning of Trusted Execution Environments (SGX/SEV) attestation doc signatures.

### Distributed Mesh & Execution
- **`krk-dcmp`**: Distributed Gossip Mesh protocol. Feature-gated to support cross-compilation on restricted enclaves.
- **`krk-wasm`**: Deterministic WASM executor sandbox, feature-gated behind optional compilation.
- **`krk-state`**: Core key-value store and Merkle DAG database.
- **`krk-lattice` / `krk-quotient`**: Algebraic equivalence and semantic lattice compression engines.
- **`krk-byzantine`**: Byzantine fault recovery, equivocation audits, and consensus recovery.
- **`krk-symbolic`**: Symbolic path exploration and reachability index search.
- **`krk-proof-graph`**: Proof dependency graph, proof lineage, and contamination audit.

---

## 3. PSC v3 HARDENING MECHANISMS

The Phase-Stabilizing Controller (PSC) contains two major systems engineering upgrades designed to defeat adversarial observer bypass attacks:

### I. Dual-Timescale Kalman Observer
Traditional Exponential Moving Average (EMA) observers are vulnerable to slow-ramp noise injection. The dual-timescale design operates two parallel filter channels:
- **Fast Channel** ($\alpha = 0.3$): Catches sudden high-frequency burst attacks.
- **Slow Channel** ($\alpha = 0.02$): Remains immune to noise; tracks long-term structural drift.
- The estimates are fused dynamically: $\hat{\lambda} = w_f \hat{\lambda}_f + (1 - w_f) \hat{\lambda}_s$. When high variance or spikes are detected, $w_f$ increases to prioritize immediate response.

### II. Innovation Residual Test
Monitors the prediction error $r(t) = \lambda_{\text{observed}} - \lambda_{\text{predicted}}$:
- **Spike Detection**: Flags immediate 3-sigma bursts ($|r(t)| > 3\sigma$) and forces a transition to `CollapsePrevention` mode.
- **Autocorrelation Detection**: Tracks consecutive same-sign residual steps over a 5-tick window to expose slow, insidious drift attacks before they corrupt the normalization.

---

## 4. COMMAND GUIDE

### Hermetic Compilation
To compile the entire workspace (including feature-gated environments for enclaves):
```bash
# Standard compilation
cargo build --workspace

# Restricted environments (disables native quinn/wasmtime dependencies)
cargo build --workspace --no-default-features
```

### Running Test Suite
Validates the mathematical stability models, observers, and SMT verification:
```bash
cargo test --workspace
```

### Running the Wind Tunnel Simulator
To run the adversarial simulator scenario benchmarks:
```bash
cargo test -p krk-adversarial-bench
```

**STATUS: PRODUCTION HARDENED / 28 TESTS GREEN / 43 CRATES COMPILE**
