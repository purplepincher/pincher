# Embedding Benchmark — RTX 4050 / WSL2

**Date:** 2026-06-03
**Hardware:** RTX 4050 6GB VRAM (not accessible from WSL2)
**Python:** 3.10 | ONNX Runtime 1.23.2
**Rust crate:** `ort` 2.0.0-rc.12 (feature-gated behind `onnx`)
**Model:** sentence-transformers/all-MiniLM-L6-v2 (FP32 ONNX, 87MB)

## Results

| Method | Median | p99 | Notes |
|---|---|---|---|
| Random `np.random.randn` | 6–9 µs | 15–35 µs | **Broken** — random vectors, no semantic meaning |
| Deterministic hash (pincherOS fallback) | 54–55 µs | 112–200 µs | SHA-256 trigram + word hashing; deterministic but not semantic |
| ONNX MiniLM-L6-v2 (CPU) | **8.1 ms** | **44.4 ms** | Real semantic embeddings via ONNX Runtime CPU |
| Cosine similarity (384-dim) | 2.5 µs | 3.5 µs | Post-embedding comparison is cheap |

### Similarity Quality

- **Same text → cosine sim = 1.000** (deterministic hash: perfect recall for identical input)
- **Different text → cosine sim = -0.016** (deterministic hash: near zero, as expected — no semantic understanding)

The hash fallback produces *deterministic* embeddings (same text → same vector) but has **no semantic understanding** — "turn on lights" and "switch on the lights" get completely different vectors.

## Findings

### 1. Random Embedding is Broken — Not Acceptable for MVP
Random `np.random.randn` produces unique vectors each call. Teaching "turn on lights" and then querying "turn on lights" would **never match**. PincherOS correctly avoids this — it uses deterministic hash fallback instead.

### 2. Deterministic Hash Fallback: Works for MVP, Not for Semantic Search
The SHA-256 trigram/word hash fallback in `onnx.rs` is:
- ✅ **Deterministic** — same input → same vector, so teach-then-match works
- ✅ **Fast** — ~55µs per embedding
- ✅ **Zero dependencies** — no model download needed
- ❌ **No semantic similarity** — "turn on the lights" ≠ "switch on lights"
- ❌ **Not transferable** — can't match paraphrased intents

**Verdict:** Acceptable for MVP if you only support exact/near-exact intent matching. Not sufficient for fuzzy semantic search.

### 3. ONNX MiniLM-L6-v2: Real Semantic Embeddings
- Model size: 87MB (FP32) — the `model_int8.onnx` variant doesn't exist on HF; only `model.onnx` (FP32) and `model_O1.onnx` through `model_O4.onnx` (quantized)
- Latency: **8ms median** on CPU — very usable
- Dimension: 384 — compact
- Quality: True semantic embeddings; "turn on lights" ≈ "switch on lights"

### 4. GPU / CUDA EP: Not Available in Current Environment
- ONNX Runtime 1.23.2 has **only `CPUExecutionProvider`** and `AzureExecutionProvider`
- **No `CUDAExecutionProvider`** — WSL2 doesn't expose the RTX 4050 via `nvidia-smi`
- To get CUDA EP: need `onnxruntime-gpu` Python package OR `ort` Rust crate compiled with CUDA support
- On bare-metal Linux (not WSL2), CUDA EP would reduce latency to ~1-2ms

### 5. Rust `ort` Crate
- PincherOS uses `ort = "2.0.0-rc.12"` behind `features = ["onnx"]`
- Default build (no `onnx` feature) falls back to hash embeddings
- The `ort` crate supports CUDA EP via feature flags (`cuda`)

## Recommendations

### For MVP
1. **Ship deterministic hash embedding as default** — it works for teach-then-match with exact/near-exact phrases
2. **Make ONNX an optional feature** — already done via `features = ["onnx"]`
3. **Document that ONNX model enables semantic matching** — users can opt in

### For v0.2
1. **Bundle the quantized ONNX model** (~23MB with O4 quantization) to avoid download at runtime
2. **Try `model_O4.onnx`** (INT8 quantized) — should be ~23MB and faster on CPU
3. **Enable CUDA EP** on bare-metal deploys — compile `ort` with `cuda` feature for ~1-2ms inference

### For bare-metal RTX 4050 deploy
```toml
# In Cargo.toml for GPU-accelerated build
ort = { version = "2.0.0-rc.12", features = ["cuda"] }
```

This would enable `CUDAExecutionProvider` and reduce embedding latency from ~8ms to ~1-2ms.

## Model Storage

The full FP32 ONNX model is saved at:
```
~/.pincher/models/all-MiniLM-L6-v2-int8.onnx (87MB, FP32)
```

Note: Despite the filename, this is the FP32 model. The INT8 quantized variant at HF is `model_O4.onnx` — worth downloading separately for smaller size.
