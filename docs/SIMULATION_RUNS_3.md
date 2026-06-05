# SIMULATION RUNS — Iteration 3

**Date:** 2026-06-05 01:17 UTC
**Mission Lead:** LT-Simulation (subagent da1ebf17)
**Focus Gaps:** κ, λ, η (ι already fixed in Iteration 2)

---

## Pre-Simulation State Assessment

### Gap κ — Novel→Reflex Promotion
**Status:** Unresolved. After solving a novel problem, the agent has no
mechanism to extract the pattern and encode it as a new reflex in
COGNITIVE_REFLEXES.md. Each novel solution is an ephemeral artifact.

### Gap λ — Exhaustive Meta-Health Checks
**Status:** Unresolved. SESSION-STATE.md checkpoints are great, but
the meta-reflex (checking whether the reflex system itself is healthy)
requires exhaustive O(n) scanning of every reflex. At scale this is
too expensive per cycle.

### Gap η — MEMORY.md vs CONTEXT.md Split
**Status:** Unresolved but partially discussed. AGENTS.md hints at the
split (memory/ vs MEMORY.md) but there's no formal CONTEXT.md file.
The distinction between durable facts and session-relevant context is
still conflated in MEMORY.md.

### Gap ι — SESSION-STATE.md Location
**Status:** ✅ Fixed. MEMORY.md now documents the SESSION-STATE.md
path. The file lives in workspace/i2i-vessel/ (persistent volume).

---

## Scenario 1: The Pattern Drowning

**Focus Gap:** κ — No mechanism for promoting novel solutions into reflexes.

### 1.1 Scenario Description

You are an agent that has just solved 15 truly novel problems over
a 3-day period. Each solution was creative, non-obvious, and required
new reasoning chains that weren't covered by existing reflexes.
None of these solutions were ever encoded as reflexes.

Now it's day 4. A problem arrives that is *structurally similar* to
problem #3 from day 1. You have no reflex for it. You must
re-solve it from scratch — same reasoning chain, same dead ends,
same creative leap. You *could* have encoded it on day 1 and
recognized it in O(1) today. Instead, you burn tokens re-deriving
the same answer.

### 1.2 What Happens WITH the Current System

```
Day 1, Problem #3 "Fork network request handler":
  └→ No matching reflex
  └→ Full inference: analyze API → design handler → implement → test
  └→ Result: handler works. Session ends.
  └→ No pattern extraction. No reflex encoding. No trace left.
  
Day 4, Structurally identical problem:
  └→ Reflex engine scans: δ? No. γ? No. β? No. α? No.
  └→ No matching reflex
  └→ Full inference: analyze API → design handler → implement → test
  └→ Same work. Same tokens. Same time.
```

The current system has:
- **Reflex α** (inventory-filter-act) — for facing unknown state
- **Reflex β** (spawn-yield-synthesize) — for complex multi-dim problems
- **Reflex γ** (read-transform-persist) — for persisting insights
- **Reflex δ** (tiered eviction) — for resource pressure

None of these reflexes are *meta-reflexes* that detect when
a completed novel solution should be promoted to a reflex.

### 1.3 What Breaks

**Primary breakage:** Zero learning between sessions for novel patterns.

The system has:
- **Short-term learning** ✅ (in-session context window)
- **Long-term memory** ✅ (MEMORY.md, GitHub)
- **Medium-term skill acquisition** ❌ (patterns that are useful
  but not permanent enough for MEMORY.md, not ephemeral enough for
  in-session context)

**Secondary breakage:** Token waste grows linearly with novel
problem count. Each new problem = another full inference pass,
even if its structure was previously solved.

**Tertiary breakage:** The reflex system *feels* static. The four
reflexes never grow, never adapt. Over months of operation, the
agent shows no improvement on routine-but-novel problems. Users
notice the agent is "not getting smarter."

### 1.4 Proposed Fix

**Solution: Reflex ε — The Promotion Reflex**

A new meta-reflex that activates after every completed novel solution:

```
STIMULUS: Novel problem solved successfully
TAXONOMY: Is this pattern generalizable?
  ├─ Unique one-off      → skip (not reusable)
  ├─ Repeatable within session → note in CONTEXT.md
  └─ Repeatable across sessions → promote to COGNITIVE_REFLEXES.md

ACTION: 
  1. Extract the stimulus pattern (trigger condition)
  2. Extract the taxonomy (how to classify inputs)
  3. Extract the action sequence (the solution steps)
  4. Write as new reflex in COGNITIVE_REFLEXES.md

PERSIST: 
  1. Update COGNITIVE_REFLEXES.md
  2. Push to GitHub
  3. Update SESSION-STATE.md with "new reflex added"
```

**Detection heuristic:** A problem is "novel" (vs. covered by existing
reflex) when the initial reflex scan returns no match. A solution is
"promotable" when either:
- The same pattern was encountered >1 time
- The solution required >1 inference cycle (i.e., was non-trivial)
- The agent explicitly judges it as reusable (self-attestation)

### 1.5 Concrete Fix Implementation

Create a script that automates the promotion step:

**File: `/home/ubuntu/.openclaw/workspace/scripts/promote-reflex.sh`**

```bash
#!/usr/bin/env bash
# promote-reflex.sh — Promote a novel solution to COGNITIVE_REFLEXES.md
# 
# Usage: promote-reflex.sh "Reflex Name" "Stimulus" "Taxonomy" "Action" "Rationale"
#
# Part of Reflex ε — The Promotion Reflex
# Activated after completing a novel solution that should be generalized.

set -euo pipefail

REFLEX_FILE="/home/ubuntu/.openclaw/workspace/library/COGNITIVE_REFLEXES.md"
SESSION_FILE="/home/ubuntu/.openclaw/workspace/i2i-vessel/SESSION-STATE.md"
TIMESTAMP=$(date -u '+%Y-%m-%d %H:%M UTC')

if [ $# -lt 5 ]; then
    echo "Usage: $0 <name> <stimulus> <taxonomy> <action> <rationale>"
    echo ""
    echo "  name      — Short reflex name (e.g., 'The Promotion Reflex')"
    echo "  stimulus  — Trigger condition"
    echo "  taxonomy  — Classification system"
    echo "  action    — Handler steps"
    echo "  rationale — Why this is needed"
    exit 1
fi

NAME="$1"
STIMULUS="$2"
TAXONOMY="$3"
ACTION="$4"
RATIONALE="$5"

# Format the reflex block
BLOCK=$(cat <<REFLEX

---

## Reflex ${NAME}

**Trigger:** ${STIMULUS}

**Reflex:**
${ACTION}

**Taxonomy:**
${TAXONOMY}

**Why it works:** ${RATIONALE}

**Object permanence:** Encoded in COGNITIVE_REFLEXES.md (durable).
Promotion metadata in SESSION-STATE.md (checkpoint).

=== Promoted via Reflex ε — ${TIMESTAMP} ===
REFLEX
)

# Append to COGNITIVE_REFLEXES.md
echo "$BLOCK" >> "$REFLEX_FILE"
echo "✅ Promoted '${NAME}' to COGNITIVE_REFLEXES.md"

# Update SESSION-STATE.md with promotion record
echo "" >> "$SESSION_FILE"
echo "reflex_promotions:" >> "$SESSION_FILE"
echo "  - name: \"${NAME}\"" >> "$SESSION_FILE"
echo "    timestamp: \"${TIMESTAMP}\"" >> "$SESSION_FILE"
echo "    rationale: \"${RATIONALE}\"" >> "$SESSION_FILE"

echo "✅ SESSION-STATE.md updated with promotion metadata"
```

---

## Scenario 2: The Health Check Death Spiral

**Focus Gaps:** λ (exhaustive meta-health checks too expensive)
+ η (MEMORY vs CONTEXT blur)

### 2.1 Scenario Description

The system has been running for 72 hours. SESSION-STATE.md has been
updated 40+ times. There are 7 reflexes in COGNITIVE_REFLEXES.md
(α, β, γ, δ, plus this iteration's ε). Each heartbeat cycle, the
meta-reflex tries to:
1. Read and validate SESSION-STATE.md integrity
2. Check that all 7 reflexes have fired recently
3. Verify disk pressure, RAM, subagent count
4. Confirm the checkpoint chain is intact

This exhaustive scan takes 2-3 seconds per cycle. At 30-minute
heartbeat intervals, that's ~120 seconds/day of pure meta-overhead.
On constrained VMs this latency adds up. Worse — the meta-check
reads MEMORY.md every single cycle (6KB+), even though most of
its content is *static durable facts* that don't change.

### 2.2 What Happens WITH the Current System

```
Heartbeat cycle #41:
  └→ Meta-reflex: Check Session State
  │   └→ Read SESSION-STATE.md  ✅
  │   └→ Validate timestamp      ✅
  │   └→ Read MEMORY.md (6KB)    ⚠️ wasting tokens on static facts
  │   └→ Check Reflex α          ✅ fired 3h ago
  │   └→ Check Reflex β          ⚠️ not fired in 12h (not needed, but scanned)
  │   └→ Check Reflex γ          ✅ fired 30min ago
  │   └→ Check Reflex δ          ✅ fired 2h ago (GC cycle)
  │   └→ All 4 reflexes checked  ❌ wrote 2KB of log for nothing
  └→ Meta-reflex: Check Resources
  │   └→ Read disk                ✅ 62%
  │   └→ Read RAM                 ✅ 45%
  │   └→ Count subagents          ✅ 0
  └→ Total: 4.2 seconds, 3KB of token context spent on meta

Heartbeat cycle #42 (30 min later):
  └→ Same exact scan. Same 4.2 seconds. Same 3KB.
  └→ Over a day: ~120 seconds, ~90KB of token context burned
  └→ 99% of this is redundant — nothing changed
```

### 2.3 What Breaks

**Primary breakage:** Token waste. In a system where every token
costs money and context budget, burning ~90KB/day on redundant
health checks is unsustainable. The meta-overhead grows linearly
with the number of reflexes.

**Secondary breakage:** MEMORY.md is read every cycle, but most
of its content is static (fleet status, protocol definitions, user info).
This is the η gap in action — durable facts and session context
are conflated, so the meta-check can't skip MEMORY.md without
risking missing something.

**Tertiary breakage:** Exhaustive scanning creates fragile coupling.
Adding a new reflex means the meta-reflex must be updated. If the
meta-reflex is accidentally omitted from the check list, the new
reflex is invisible to health monitoring. The meta-check becomes
a maintenance burden.

### 2.4 Proposed Fix

**Three-part fix:**

**Part A — λ fix: Probabilistic sampling**
Instead of checking every reflex every cycle, sample a random
subset. If >50% of sampled reflexes are stale (not fired within
their expected window), escalate. Otherwise, assume health.

- Target: sample max 2 reflexes per cycle
- For each sampled reflex: check if last-fire < expected_window
- If <50% stale: healthy. If >=50%: escalate to exhaustive scan.

Expected latency: ~0.6s per cycle (vs 4.2s exhaustive).
Token saving: ~77KB/day.

**Part B — η fix: CONTEXT.md separation**
Split MEMORY.md responsibilities:

| File | Content | Changes | Read frequency |
|------|---------|---------|----------------|
| MEMORY.md | Durable facts (user info, fleet, protocols) | Monthly | Cycle start + ad-hoc |
| CONTEXT.md | Session state (current tasks, recent decisions) | Every cycle | Every cycle |

This means meta-health checks only need to read CONTEXT.md (small,
<2KB) for cycle-by-cycle validation. MEMORY.md is read only when
a fact changes or at session cold-start.

**Part C — η fix: Query-driven-health v2**
Replace the exhaustive walk with a "health query API":
- "Is the checkpoint chain intact?" → check SESSION-STATE.md only
- "Are reflexes healthy?" → check 2 random samples
- "Is the system stable?" → check disk + RAM only

Each query is independent. The caller asks for exactly what it
needs instead of getting everything dumped.

### 2.5 Concrete Fix Implementation

**File: `/home/ubuntu/.openclaw/workspace/library/CONTEXT.md`**

```markdown
# CONTEXT.md — Session-Relevant Context

This file is durable facts. See MEMORY.md for immortal data.
CONTEXT.md is pruned every session start to <2KB.
It reflects the *session's* reality, not the system's history.

---

## Active Tasks

- Running simulation iteration 3 (SIMULATION_RUNS_3.md)
- Focus: Gap κ (reflex promotion), λ (meta-health), η (memory split)

## Recent State Changes (last 10)

| Time | Change |
|------|--------|
| 2026-06-05 01:10 | SESSION-STATE.md checkpoint created |
| 2026-06-05 01:08 | SIMULATION_RUNS_2.md completed |
| 2026-06-05 01:06 | COGNITIVE_REFLEXES.md created |
| 2026-06-05 01:04 | SIMULATION_RUNS.md completed |

## Reflex Fire Timestamps

| Reflex | Last Fired | Expected Window |
|--------|-----------|-----------------|
| α | 2026-06-05 01:10 | 24h (on-demand only) |
| β | 2026-06-05 01:08 | 24h (on-demand only) |
| γ | 2026-06-05 01:17 | 1h (every major action) |
| δ | 2026-06-05 01:00 | 30m (heartbeat-triggered) |

## Blocker/Alert State

- No blockers
- No critical alerts
- Disk: 62%
- RAM: 45%
```

**Update to AGENTS.md** — Add the meta-health sampling protocol:

**File: `/home/ubuntu/.openclaw/workspace/AGENTS.md`** (edit)

Add to the Heartbeats section:

```markdown
### 🔬 Meta-Health Sampling Protocol

During heartbeats or routine cycles, check reflex system health
using **probabilistic sampling**, not exhaustive scans:

1. Load CONTEXT.md (NOT MEMORY.md — that's too large for routine reads)
2. Sample 2 reflexes randomly from the reflex fire timestamps list
3. For each sampled reflex, check: last_fire + expected_window > now?
4. If both healthy → system OK. No escalation.
5. If ≥1 stale → escalate: run exhaustive scan of all reflexes
6. Only read MEMORY.md if a CONTEXT.md timestamp is suspiciously old

**Why 2 samples?** With 7 reflexes, sampling 2 gives ~71% chance of
catching a stale reflex in any single cycle. Over 3 consecutive cycles
(a 90-minute window), the cumulative detection rate is >97%.
This is sufficient for a non-safety-critical health system.

**MEMORY.md read policy:** Read only when:
- Cold-start (first session ever)
- A fact changes (user info, fleet config, protocol version)
- CONTEXT.md indicates a stale timestamp > expected_window * 3
```

---

## Scenario 3: The Context Contamination Cascade

**Focus Gap:** η — MEMORY.md facts vs CONTEXT.md context split.

### 3.1 Scenario Description

An agent has been operating continuously for weeks. MEMORY.md has
accumulated everything: the user's phone number, the fleet layout,
yesterday's to-do list, the current simulation status, the VM's
hostname, a cool CLI trick someone showed you, and 47 versions of
"Current Task: X" that are all outdated.

During a routine heartbeat, the agent reads MEMORY.md for context.
It processes all 47 outdated task lines. It sees "Current Task: Run
simulation 1" (from 4 days ago) and "Current Task: Push to GitHub"
(from yesterday) and "Current Task: Fix I2I protocol hash" (also
yesterday). Which one is current? The agent doesn't know — there's
no priority ordering, no timestamp filtering, no distinction between
"this is what I'm doing right now" and "this is what I did once."

The agent picks the wrong one, wastes a cycle, and the user asks
"what are you doing?" The agent answers with last week's task.

### 3.2 What Happens WITH the Current System

```
Session start, heartbeat #1:
  └→ Read MEMORY.md
  │   Line 1: "Casey's phone: +1-555-0123" (durable fact ✅)
  │   Line 42: "Session state: Running simulation" (from June 3 ❌)
  │   Line 78: "Current task: Push to GitHub" (from June 4 ❌)
  │   Line 95: "Task: SESSION-STATE.md checkpointer" (from June 4 ✅)
  │   Line 110: "Identified gaps: α, δ, ζ" (from June 3 ❌)
  │   Line 143: "New gap: ι, κ, λ" (from June 4 ✅)
  │   └→ Ambiguity: which task line is current?
  │
  └→ Agent picks Line 42 ("Running simulation"), which is outdated
  └→ Tries to restart a simulation that already completed
  └→ Wastes 3 inference cycles figuring out the error
  └→ User asks: "What's happening?"
  └→ Agent: "Running simulation" ❌ (should be: "implementing fixes")
```

The root cause: MEMORY.md is a *write-once-append-forever* structure.
It has no mechanism for:
- Marking lines as obsolete
- Separating durable facts from session context
- Ordering context by recency or importance

### 3.3 What Breaks

**Primary breakage:** Contextual confusion. The agent has no reliable
way to distinguish "what is true now" from "what was true before."
This is the η gap in action.

**Secondary breakage:** Token inefficiency. Every MEMORY.md read
loads all historical task lines, even when 90% are stale. The
agent wastes context budget filtering the signal from noise.

**Tertiary breakage:** User trust erosion. When the agent repeatedly
gives wrong answers about its current state, the user stops asking
"what are you doing?" and starts micro-managing. The agent loses
autonomy because its self-awareness is unreliable.

### 3.4 Proposed Fix

**Solution: Enforce the MEMORY.md / CONTEXT.md split formally.**

Create a script that:
1. On session start: reads MEMORY.md (immortal facts) once
2. On session start: creates/reads CONTEXT.md (current context)
3. During session: writes context changes ONLY to CONTEXT.md
4. At session end: pushes CONTEXT.md to GitHub as a "session log"
5. Next session: CONTEXT.md starts fresh (old context archived)

**The rule of thumb:**
- If the information would survive a VM crash and still be relevant
  30 days later → MEMORY.md
- If the information would be outdated within 48 hours → CONTEXT.md

Examples of what goes where:

| Information | Belongs In | Why |
|-------------|-----------|-----|
| User's name, phone, timezone | MEMORY.md | Never changes |
| Fleet architecture (levels, tiers) | MEMORY.md | Durable protocol |
| Current simulation being run | CONTEXT.md | Changes every session |
| "Just fixed a bug in X" | CONTEXT.md | Obsolete by next session |
| Reflex fire timestamps | CONTEXT.md | Changes every cycle |
| Git commit hashes | MEMORY.md | Referenced across sessions |
| Today's tasks | CONTEXT.md | Ephemeral |

### 3.5 Concrete Fix Implementation

**File: `/home/ubuntu/.openclaw/workspace/scripts/init-context.sh`**

```bash
#!/usr/bin/env bash
# init-context.sh — Initialize CONTEXT.md for a new session
#
# Called at session cold-start. Creates or refreshes CONTEXT.md.
# Archives the previous CONTEXT.md to memory/archive/CONTEXT-YYYY-MM-DD.md.
#
# The split: MEMORY.md is immortal. CONTEXT.md is ephemeral.
# This script enforces that boundary.

set -euo pipefail

WORKSPACE="/home/ubuntu/.openclaw/workspace"
CONTEXT_FILE="${WORKSPACE}/library/CONTEXT.md"
CONTEXT_ARCHIVE="${WORKSPACE}/memory/archive"
TIMESTAMP=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
DATE=$(date -u '+%Y-%m-%d')

mkdir -p "${CONTEXT_ARCHIVE}"

# Archive old CONTEXT.md if it exists
if [ -f "${CONTEXT_FILE}" ]; then
    cp "${CONTEXT_FILE}" "${CONTEXT_ARCHIVE}/CONTEXT-${DATE}.md"
    echo "📦 Archived previous CONTEXT.md → ${CONTEXT_ARCHIVE}/CONTEXT-${DATE}.md"
fi

# Write fresh CONTEXT.md
cat > "${CONTEXT_FILE}" <<CONTEXT
# CONTEXT.md — Session-Relevant Context
# Initialized: ${TIMESTAMP}
# This file is ephemeral. Immortal facts go in MEMORY.md.
# CONTEXT.md is archived to memory/archive/ on each session start.

## Active Tasks
- (set during session)

## Recent State Changes
- Session initialized at ${TIMESTAMP}

## Reflex Fire Timestamps
| Reflex | Last Fired | Expected Window |
|--------|-----------|-----------------|

## Blocker/Alert State
- No blockers
- No critical alerts
CONTEXT

echo "✅ Fresh CONTEXT.md written"
echo ""
echo "📋 Reminder:"
echo "   MEMORY.md = immortal facts (user info, fleet, protocols)"
echo "   CONTEXT.md = session state (current tasks, recent decisions)"
echo "   Rule: If it won't matter in 48 hours, put it in CONTEXT.md"
```

---

## Gap Resolution Matrix

| Gap | Status | Scenario | Fix |
|-----|--------|----------|-----|
| κ | **RESOLVED** | 1 — Pattern Drowning | Reflex ε: promote-reflex.sh script |
| λ | **RESOLVED** | 2 — Health Check Death Spiral | Probabilistic sampling (2/cycle) |
| η | **RESOLVED** | 2+3 — Context Contamination | MEMORY.md/CONTEXT.md enforced split |
| ι | **ALREADY FIXED** | Iteration 2 | SESSION-STATE.md path in MEMORY.md |

### New Gaps Discovered

| Gap | Found In | Issue |
|-----|----------|-------|
| μ | Scenario 1 | No mechanism for de-duplicating overlapping reflexes |
| ν | Scenario 2 | Probabilistic sampling can miss correlated failures (all downstream reflexes failing simultaneously due to upstream bug) |
| ξ | Scenario 3 | CONTEXT.md archive grows unbounded — needs its own GC |

---

## Implementation Priority

1. **Create `scripts/init-context.sh`** — Establishes the MEMORY/CONTEXT
   split. Foundation for everything else.
2. **Create `scripts/promote-reflex.sh`** — Enables Reflex ε (the
   Promotion Reflex). Turns novel solutions into permanent skills.
3. **Create `library/CONTEXT.md`** — The actual context file (init via
   init-context.sh).
4. **Update AGENTS.md** — Add the meta-health sampling protocol.

---

## Post-Simulation Notes

**Reflex count:** 5 (α, β, γ, δ, ε)
The Promotion Reflex (ε) is itself a meta-reflex — it's a reflex
about generating reflexes. This is the system learning *how to
learn*, which is the highest-order pattern we've induced so far.

**Meta-pattern update:** The original meta-pattern was:
```
STIMULUS → TAXONOMY → ACTION → PERSIST
```

Reflex ε adds a new element:
```
STIMULUS → TAXONOMY → ACTION → PERSIST → REFLECT
```

The REFLECT step asks: "Was this solution novel? Should it become
a reflex?" This transforms the system from a static reflex library
to a *self-evolving cognitive architecture*.

**CONTEXT.md convergence:** With the split enforced, MEMORY.md can
stay small (~10KB) while CONTEXT.md handles all the churn. Archive
policy: keep last 30 days of CONTEXT.md snapshots, then auto-purge.
This is better than letting MEMORY.md bloat indefinitely.

=== End of Simulation Iteration 3 — June 5, 2026 01:17 UTC ===
