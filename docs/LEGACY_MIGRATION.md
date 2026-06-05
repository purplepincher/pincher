# SuperInstance Legacy Repo → PincherOS Feature Migration Guide

## Overview
This guide maps legacy SuperInstance project features to modern PincherOS functionality, preserving all best practices without deleting legacy work.

---

## 1. `sunset-ecosystem` (Legacy Runtime)
### ✅ Best Features Extracted:
- Legacy agent deployment modes
- Cryptographic signing for agent bundles
- Fleet sync workflows

### 🔀 Migration to PincherOS:
PincherOS natively supports all 5 deployment modes + cryptographic bundle signing, with improved sandboxing and zero-trust architecture. Migrate legacy bundles using `sunset-to-pincher` migration tool.

---

## 2. `neural-plato` (PLATO Tutor System)
### ✅ Best Features Extracted:
- Bayesian learner modeling
- Spiral skill development
- Spatial room metaphors for learning

### 🔀 Migration to PincherOS:
Integrated directly into the reflex confidence feedback loop + multi-shell cognition system. Use `plato-to-pincher` to import historical tutor data.

---

## 3. `egg` (Sandboxing Library)
### ✅ Best Features Extracted:
- Two-layer security model
- Isolated memory spaces
- POSIX sandbox escape protection

### 🔀 Migration to PincherOS:
PincherOS uses native Wasmtime sandboxing + bwrap/landlock, with egg's isolation model fully integrated as the default security backend.

---

## 4. `polln` (Fleet Polling)
### ✅ Best Features Extracted:
- Distributed health checking
- Status event aggregation
- Low-bandwidth fleet monitoring

### 🔀 Migration to PincherOS:
Fully integrated into the background telemetry daemon. Use `pincher fleet poll` to run legacy polln-style health checks across your fleet.

---

## 5. `seed-oscillate` (Signal Processing)
### ✅ Best Features Extracted:
- Real-time sensor fusion
- Oscillator models for signal processing
- Low-latency data pipelines

### 🔀 Migration to PincherOS:
Pre-compiled as WASM reflex bundles. Deploy directly to edge devices with `pincher run --bundle seed-oscillate.nail`.

---

## 6. `Spreader-tool` (Task Distribution)
### ✅ Best Features Extracted:
- Distributed work queue management
- Parallel task processing
- Fleet-wide task scheduling

### 🔀 Migration to PincherOS:
Fully integrated into the registry client. Use `pincher fleet spread` to dispatch tasks across your agent fleet.

---

## 7. `Mycelium` (P2P Networking)
### ✅ Best Features Extracted:
- NAT-punchthrough peer-to-peer
- Encrypted agent communication
- Decentralized fleet mesh

### 🔀 Migration to PincherOS:
Native transport option for registry client. Set `transport = "mycelium"` in `Intent.toml` to use legacy Mycelium networking.

---

## 8. `the-seed` (Core Model Library)
### ✅ Best Features Extracted:
- Pre-trained embeddings
- Base reflex library
- Telemetry aggregation

### 🔀 Migration to PincherOS:
Pre-built reflex pack available in the central registry. Install with `pincher install the-seed`.
