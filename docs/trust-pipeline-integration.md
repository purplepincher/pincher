# Trust Pipeline Integration — From Reflex Engine to Intent Pipeline

**Source:** Lever-Runner (decommissioned) trust-gated intent pipeline  
**Pattern:** LLM-as-intent-compressor, vector DB as command authority, trust as safety gate

---

## The Analogy

Pincher already has a reflex engine: reflexes are matched by intent, executed with SAEP veto. Lever-Runner had the same fundamental architecture but with a mathematically rigorous trust model on top.

The isomorphism:

| Pincher Component | Lever-Runner Component | What It Does |
|-------------------|----------------------|--------------|
| Reflex | Command | A pre-approved action |
| Intent match | Embedding search | Vector similarity finds the right one |
| SAEP veto | Trust gate | Hard threshold: below this, don't run |
| Confidence threshold | MATCH_SIMILARITY_FLOOR (0.55) | Below this, surface /teach instead |
| SAEP bypass | Auto-promote | Trusted patterns skip re-validation |

---

## What Pincher Can Borrow

### 1. Trust as a Gate, Not a Score

The key mathematical insight from lever-runner v0.2 bugfix:

```python
# BAD (was bug in v0.1):
winner = max(candidates, key=lambda m: m.trust * -m.distance)
# Trust and distance mixed → meaningless composite

# GOOD (v0.2 fix):
eligible = [m for m in matches if m.trust >= MIN_TRUST]
eligible = eligible or matches[:1]  # fallback to top-1 if all below floor
winner = min(eligible, key=lambda m: m.score)  # L2 distance
```

**Rule:** Trust is a binary gate (above/below threshold), not a weighting factor. Once past the gate, the most semantically similar match wins, not the most trusted one.

### 2. Auto-Promote Loop

An hourly background job that:
- Bumps trust on reflexes that succeed repeatedly (+10 when success > 20)
- Rewrites failing reflexes via remote LLM (trust < 30 AND failures ≥ 5)
- Soft-deletes the old entry, inserts the new one at lower trust

This creates a **self-healing reflex table** — reflexes that consistently fail get automatically diagnosed and fixed.

### 3. The LLM Blindfold Contract

The critical safety insight: the LLM should never know about the action space. In lever-runner, the LLM was prompted to "compress this into a 3-8 word phrase" — it didn't know phrases became commands. The embedding model connected the dots.

For pincher's reflex engine, this means:
- The LLM (or intent parser) should output an abstract intent, not a reflex ID
- The vector DB makes the reflex match — the LLM doesn't know about reflexes at all
- This prevents prompt injection from selecting arbitrary reflexes

### 4. Consistency-Immediate Read Mode

LanceDB's `read_consistency_interval=0` forces every read to see the latest writes. In practice: when a user teaches a new reflex (via /teach), the very next request finds it. No index rebuild, no TTL delay, no REST cache.

Pincher's reflex store should use the same pattern: write-then-immediately-read must work.

### 5. Per-Instance Isolation

Lever-runner isolates every Telegram chat into its own LanceDB table. Same pattern for pincher: every sandbox instance gets its own reflex table, seeded from a global template. Instances can /teach without affecting each other.

---

## Implementation Notes

### Token Budget

| Step | Tokens | What Happens |
|------|--------|-------------|
| LLM call | ~60 in, ~8 out | Compress intent to phrase |
| Embedding | ~12 equivalent | MiniLM produces 384-dim normalized vector |
| Search | ~0 (local) | LanceDB cosine search, O(log n) |
| Exec | ~0 | Sandboxed subprocess (20s timeout) |
| Trust update | ~0 | In-place LanceDB row update |
| **Total** | **~80 tokens** | vs ~1,500-8,000 for tool-calling |

### Dependencies

| Library | Size | Purpose |
|---------|------|---------|
| `sentence-transformers/all-MiniLM-L6-v2` | ~80 MB | 384-dim encoder |
| `lancedb` | ~15 MB | Local vector store |
| `torch` | ~800 MB | Embedding backend (shared with pincher if already installed) |

### Optional: Remote LLM Rewrite Path

For the auto-promote loop, a remote LLM (Claude, DeepSeek) proposes corrected commands. If no API key is configured, auto-promote is a no-op — the system still functions, it just stops self-healing.

---

## Tracked in This Document

- [ ] Extract trust model as standalone crate
- [ ] Add `MATCH_SIMILARITY_FLOOR` concept to SAEP configuration
- [ ] Add auto-promote cron job to pincher systemd units
- [ ] Implement per-instance reflex table isolation
- [ ] Move LLM prompt to "compress, not select" for blindfold safety
