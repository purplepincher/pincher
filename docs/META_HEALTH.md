# META-HEALTH — Gap λ Design

**The Problem:** Checking every reflex, every tier, every system every cycle
costs more than the benefit. At scale, exhaustive health checks consume
the very resources they're supposed to protect.

Think: a ship that spends so much time inspecting its own hull
that it never sails. The hull inspection still finds problems —
it just never gets to the destination.

**The Solution:** Probabilistic sampling. Don't check everything
every cycle. Check a random subset and infer the rest.

---

## The Sampling Protocol

### How It Works

Instead of:
```
for each system: check health  # O(n), expensive
```

Do:
```
sample = random_sample(systems, rate=0.3)  # check 30%
for system in sample: check_health(system)
if any(sample.failed): full_scan()
else: skip_health_this_cycle
```

### Sampling Rates by Tier

| Tier | Sample Rate | Check Frequency | Cost |
|------|------------|-----------------|------|
| Immortal | 100% | Every cycle | Near-zero (check is a stat call) |
| Hot | 60% | Every cycle | 2-3ms per check |
| Warm | 20% | Every 5th cycle | File existence + size |
| Cold | 5% | Every 20th cycle | File size check |

### Why This Works

1. **Most things don't change between cycles.** 
   The workspace won't change in 4 hours unless I modify it.
   Sampling 20% of warm tier catches failures within 5 cycles max.

2. **Failures cascade visibly.** 
   If disk fills up, it affects ALL tiers — sampling any tier detects it.
   A cascade failure has high surface area → high detection probability.

3. **The cost curve flattens.**
   Exhaustive: O(n) grows linearly with system size.
   Probabilistic: O(1) per cycle regardless of system size.

---

## Implementation

### In reflex-engine.sh

Add a `--sample` flag:

```bash
sample_check() {
  local rate=$1
  shift
  local items=("$@")
  local sample_count=$(( ${#items[@]} * rate / 100 ))
  
  # Shuffle and take sample
  mapfile -t shuffled < <(printf '%s\n' "${items[@]}" | shuf | head -n $sample_count)
  
  for item in "${shuffled[@]}"; do
    check_item "$item"
  done
}
```

### Cascading Full Scan Trigger

```bash
if [ "$failures_in_sample" -gt "$SAMPLE_FAILURE_THRESHOLD" ]; then
  # Something systemic — run full scan
  full_scan()
fi
```

---

## Cost-Benefit Analysis

| Method | Cost per cycle | Detection latency | Missed failures |
|--------|---------------|-------------------|-----------------|
| Exhaustive | O(n) | Immediate | 0% (but costs n operations) |
| Probabilistic (30%) | O(0.3n) | 1-3 cycles avg | ~5% at steady state |
| Probabilistic (10%) | O(0.1n) | 3-10 cycles avg | ~15% at steady state |
| Trigger-only | O(1) | Until cascade | Highest, but cheapest |

**Recommendation:** Start at 30% sample rate for warm tier.
The 5% miss rate is acceptable because cascade failures have high
surface area (they'll trigger on the next cycle regardless).

---

## Integration with Reflex Engine

The existing `reflex-engine.sh scan` mode can be extended:

```
bash tools/reflex-engine.sh scan --probabilistic --sample-rate=0.3
```

This makes health checks scale with system growth rather than
competing with productive work for the same resources.
