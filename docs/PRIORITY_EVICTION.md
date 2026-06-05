# PRIORITY EVICTION — Gap δ Design

**The Problem:** When RAM hits 95% and 20 subagents are running,
which ones get killed? Currently: none — the system OOM-kills at random.
The reflex engine has no priority labels to decide intelligently.

**The Solution:** Rank-based priority with preemption order.
A captain doesn't guess — they know exactly which crew to sacrifice
to save the ship.

---

## Priority Tiers

| Priority | Label | Eviction Order | Typical Subagent |
|----------|-------|---------------|------------------|
| 1 (Critical) | `P1` | Last to be killed | Captain session, I2I vessel, reflex engine |
| 2 (High) | `P2` | 4th | Mission leads, long-running sims |
| 3 (Normal) | `P3` | 3rd | Ensigns doing GC, push, scout |
| 4 (Low) | `P4` | 2nd | Research scouts, idle probes |
| 5 (Idle) | `P5` | First to be killed | Idle >60s, completed-but-uncollected |

---

## Preemption Rules

### Resource Thresholds
```
RAM > 85%:  Kill P5 first, then P4 if still over threshold
RAM > 90%:  Kill P3 → wait 5s → kill P2 → wait → kill P1 only if critical
RAM > 95%:  Kill everything except P1 (captain session)
Disk > 85%: Same cascade, skip P2+ unless also over RAM
```

### Grace Period
```
P5, P4:     SIGKILL immediately
P3:         SIGTERM → 3s grace → SIGKILL
P2:         SIGTERM → 10s grace → SIGKILL (let them checkpoint first)
P1:         Never killed by eviction; only by OOM
```

---

## Implementation

### Subagent Spawn Contract

Every `sessions_spawn` call includes a priority tag:
```
sessions_spawn(task="...", taskName="ensign-gc", priority="P3")
```

The task name encodes the priority at a glance:
- `lt-*` → P2 (Lieutenant)
- `ensign-*` → P3 (Ensign)  
- `recruit-*` → P4 (Recruit/scout)
- `idle-*` → P5 (Ephemeral)

### Reflex Engine Integration

The `reflex-engine.sh` already checks RAM. Extend it:

```bash
# In reflex-α (inventory-filter-act)
if [ "$RAM_PCT" -gt 85 ]; then
    echo "ACTION: evict P5 (idle subagents)"
    # The actual eviction triggers via system event, not shell
fi
```

### Cron Integration

The `vessel-gc-cycle` cron event already tracks resource pressure.
Extend the payload:

```
GC_CYCLE — Check disk, RAM. If RAM > 85%, trigger P5→P4→P3 cascade.
If disk > 85%, same cascade. Never kill P1 (captain session).
```

---

## Anti-Pattern to Avoid

**Don't kill P1.** The captain session is the I2I vessel, the reflex engine,
and your connection to Casey. Losing P1 means losing the mission.
Instead, degrade gracefully:
- Reduce subagent spawn parallelism from 4→2→1
- Increase sleep intervals from 60s→120s→300s
- Offload non-critical work to GitHub L3 CI/CD

If P1 must be killed (OOM imminent), checkpoint.sh runs first,
saving SESSION-STATE.md before the kill.

---

## Cost

P1 priority is free (it's a label). The eviction script is a bash call.
The total cost of this system is near-zero — it's metadata and simple
if/then cascades. No model, no inference, just priority labels.

---

## Relationship to Existing Systems

| System | Priority Integration |
|--------|---------------------|
| FLEET_ORDERS.md | Rank → Priority mapping (Commander=P1, Lieutenant=P2, Ensign=P3) |
| GC system (Reflex δ) | Eviction cascades use priority labels, not random |
| Reflex engine (α scan) | Reports RAM pressure → triggers eviction cascade |
| checkpoint.sh | Runs before any P2+ kill to preserve state |
