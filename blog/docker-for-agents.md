# I Accidentally Built Docker for AI Agents

I was trying to make my AI agent stop costing money on repeated tasks. I ended up building something that lets you pack an agent's entire learned brain into a file, copy it to a Raspberry Pi, and run it with zero internet.

This is the story of PincherOS — and why I really need to stop naming things.

---

## The Cost Problem

Here's a thing nobody talks about with AI agents: they're *amnesiacs*.

Every time you ask an agent "list the docker containers," it goes through the full reasoning pipeline. It parses the intent. It reasons about what tool to use. It formats the call. It gets the result. It reasons about the result. It formats the response.

**2000 tokens. 2 seconds. $0.003.**

That's fine once. But I was running agents that did this dozens of times a day. "Check memory." "List files." "Show processes." "Disk usage." The same ten commands, over and over, each burning tokens like a space heater burning kerosene.

I did the math. If you run an agent that does "list files" 100 times a day at $0.003 per call, that's **$9/month just for `ls`**. That's insane. I'm paying an LLM to be a very expensive shell alias.

The caching solutions weren't much better. Semantic caches like GPTCache have fuzzy matching and cache invalidation problems. Prompt templates are rigid. Function calling still routes through the LLM every time. Nobody was solving the fundamental problem: **the LLM was doing work it already knew how to do.**

---

## The Reflex Insight

One night, staring at yet another $40 API bill for a personal agent that mostly checked the weather and listed files, it hit me:

**What if the second time you ask "list files," the agent just... runs `ls`?**

No LLM call. No reasoning chain. No tokens. Just 50 milliseconds and $0.00.

Not caching the *response* — that breaks when the directory changes. Caching the *behavior*. The agent learned what to do. It should just do it.

This is how biological systems work. You don't consciously reason through every step of driving a car. Your cortex learns the pattern, then it gets pushed down to faster, cheaper neural pathways. Muscle memory. Reflexes.

What if AI agents had reflexes?

---

## How It Works

The architecture is four steps, and it's deceptively simple:

### Teach

You tell the agent: "When I say 'list docker containers', run `docker ps`." The intent gets embedded into a 384-dimensional vector using MiniLM-L6 (a small sentence transformer). The embedding and the action get stored in SQLite.

```bash
$ pincher teach --intent "list docker containers" --action "docker ps"
✓ Reflex stored! (2.1ms)
```

### Match

When a new intent comes in — say, "show me my containers" — it gets embedded into the same 384-dimensional space. Then we compute cosine similarity against every stored reflex. The best match wins.

### The Confidence Loop

Here's where it gets interesting. Every reflex has a confidence score, and it follows a three-tier path:

```
similarity > 0.80  →  DIRECT   →  execute immediately, ~50ms, $0
similarity > 0.55  →  CONFIRM  →  execute but flag for review
similarity < 0.55  →  NOVEL    →  route to LLM, compile new reflex
```

And confidence isn't static. Every execution updates it:

- **Success**: confidence += 5% of the gap to 1.0
- **Failure**: confidence -= 10% of current value

This means reflexes get stronger when they work and weaker when they don't. A reflex that keeps succeeding approaches 0.99. A reflex that keeps failing drops toward 0.01 and eventually gets re-compiled by the LLM.

The system **self-decomposes bad habits and self-reinforces good ones.** You don't tune anything. It just works.

### Execute

When a direct match fires, there's no LLM in the loop at all. The reflex action runs in a sandboxed environment. The result comes back in milliseconds. You paid nothing.

```bash
$ pincher do "show me my containers"
✓ Matched reflex: "list docker containers" (confidence 0.92, 48ms)
CONTAINER ID   IMAGE          STATUS
a1b2c3d4       nginx:latest   Up 3 hours
e5f6g7h8       redis:7        Up 3 hours
```

The first time you ask, it's an LLM call. The second time, it's free. The hundredth time, it's still free. Your agent develops muscle memory.

---

## The Migration Moment

This is where I went from "useful tool" to "oh no, I'm building an operating system."

I was looking at the data model. Reflexes are stored in SQLite. Embeddings are stored alongside them as BLOBs. Confidence scores, invocation counts, timestamps — all in one database file. The agent's entire learned behavior is a single SQLite database.

**If the agent's entire state is a SQLite database of embeddings, I can just... copy it.**

This is the Docker insight. Docker works because it packages an app's entire runtime — filesystem, dependencies, config — into a single artifact that runs identically anywhere. What if we did that for AI agents?

That's how the `.nail` format was born. (Yes, hermit crabs. I named the whole project after hermit crabs. The agent is the crab. The hardware is the shell. When the shell gets too small, the crab finds a bigger one. The crab migrates. The shell doesn't.)

```bash
# On your workstation — teach the agent everything
$ pincher teach --intent "check system memory" --action "free -h"
$ pincher teach --intent "list docker containers" --action "docker ps"
$ pincher teach --intent "disk usage" --action "df -h"
# ... 47 more reflexes ...

# Pack it all up
$ pincher pack dev-agent.nail
✓ State packed! (23.4 KB)
```

A `.nail` file is a `tar.zst` archive containing:

- `manifest.json` — version, hardware fingerprint, BLAKE3 checksums
- `reflexes.db` — the full SQLite database with all reflexes and embeddings
- `identity.json` — agent name, preferences
- `config.toml` — resource thresholds, capability defaults

Then you copy it to a Raspberry Pi:

```bash
$ scp dev-agent.nail pi@raspberrypi:~/
```

And unpack it:

```bash
# On the Pi
$ pincher unpack dev-agent.nail
✓ State unpacked! (47 reflexes imported)
✓ Same crab. Bigger shell.
```

Now the Pi has an agent with 47 learned reflexes and it doesn't need internet to use any of them. The LLM sidecar isn't even running. It's pure reflex execution — 50ms per command, $0.00 per command, running on a $35 computer with 4GB of RAM.

I call this QTR — **Quiesce, Transfer, Resume.** The agent finishes any in-flight work, flushes everything to SQLite, packs up, ships out, and wakes up on new hardware with its entire brain intact. The checksums are verified with BLAKE3. If the new hardware doesn't have a tool that a reflex needs (like `apt` on a Mac), that reflex gets flagged for re-compilation.

---

## The PID Controller

Real hardware has real limits. My Pi has 4GB RAM. The LLM sidecar wants 2GB just to load. So I stole a trick from control theory.

PincherOS runs a continuous PID controller — Proportional-Integral-Derivative — that monitors RAM and CPU the same way a thermostat monitors temperature. Three operating modes:

```
RAM < 70%, CPU < 60%  →  NORMAL    →  full LLM access, full context window
RAM 70-85%, CPU 60-80%  →  LIGHT     →  reduced context, skip LLM for high-confidence reflexes
RAM > 85%, CPU > 80%  →  CRITICAL  →  reflex-only mode, no LLM calls at all
```

The PID gains are `Kp=2.0, Ki=0.1, Kd=0.5` — tuned to react strongly to current pressure, slowly eliminate steady-state error, and dampen oscillation. When the Pi's RAM hits 90%, the controller unloads the LLM sidecar entirely. The agent keeps running on pure reflexes. When RAM drops back below 80%, the sidecar loads back up.

This means the agent doesn't crash under resource pressure. It *degrades gracefully*. On a Pi under load, it falls back to reflex-only mode and still handles everything it's already learned. It just can't learn new things until the pressure drops.

The whole system targets about 1GB on a Pi 4. The Rust core uses maybe 30MB. The SQLite database is a few megabytes. The embedder adds another 50MB. The LLM sidecar is the heavyweight, but it unloads after 5 minutes of idle time.

---

## What It Actually Is

Let me be honest about what this is and what it isn't.

**It's not really an operating system.** I called it PincherOS because the hermit crab metaphor was too good, and because "PincherReflexEngineWithMigrationSupport" doesn't fit on a sticker. But the "OS" is aspirational. What ships today:

- ✅ **Teach reflexes** — associate natural language intents with shell commands
- ✅ **Execute reflexes** — match new intents against known ones with cosine similarity
- ✅ **Confidence tracking** — reflexes strengthen on success, weaken on failure
- ✅ **Migration** — pack agent state into a `.nail` file, move it between machines
- ✅ **Resource control** — PID controller degrades gracefully under memory pressure
- ✅ **Security** — veto engine blocks dangerous commands, sandboxing via bwrap+landlock (in progress)

What's real but rough:

- ⚠️ The sandbox (bwrap + landlock) isn't fully wired up yet
- ⚠️ The LLM sidecar integration works but needs a running Ollama or similar
- ⚠️ The "immunology" module (adversarial detection) is research code
- ⚠️ The WASM carapace for portable reflex execution is prototype-stage

What's honest-to-god shipping in the CLI:

```bash
pincher status       # Show shell info, reflex count, resource state
pincher teach        # Teach a new reflex (interactive or CLI args)
pincher do <intent>  # Execute an intent through the reflex engine
pincher match        # Show what would match, without executing
pincher pack         # Pack state into .nail for migration
pincher unpack       # Unpack .nail and merge state
pincher bench        # Run performance benchmarks
pincher reflexes     # List all stored reflexes with confidence
```

The core insight — treat the LLM as a compiler, not a runtime — is real and it works. The agent compiles intents into reflexes once, then executes them forever for free. The migration works — I've moved agents between a workstation and a Pi and they pick up right where they left off.

---

## A Real Demo

Here's an actual terminal session, start to finish:

```bash
$ pincher status

   🦀 PincherOS v0.1.0
  ////////////////////////
 // Shell: devbox       \\
//   Reflexes: 0          \\
\\   State: Normal        //
 \\  RAM: 34.2%         //
  ////////////////////////

$ pincher teach --intent "list docker containers" --action "docker ps"

  Teach PincherOS a new reflex
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ... Storing reflex...

  OK Reflex stored!
    ID: 1
    Intent: list docker containers
    Action: docker ps
    Confidence: 1.00
    Time: 3.21ms

$ pincher do "show me what containers are running"

  Executing: "show me what containers are running"
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  OK Matched reflex:
    Reflex ID: 1
    Match Type: Exact
    Confidence: 0.9234
    Output: CONTAINER ID   IMAGE          STATUS ...

  Time: Completed in 47.82ms

$ pincher pack dev-agent.nail

  Packing state to dev-agent.nail
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  OK State packed!
    File: dev-agent.nail
    Size: 23.41 KB
    Time: Completed in 12.44ms
```

Then on the Pi:

```bash
$ scp dev-agent.nail pi@sensor-hub:~/
$ ssh pi@sensor-hub
$ pincher unpack dev-agent.nail

  Unpacking from dev-agent.nail
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  OK State unpacked!
    Time: Completed in 8.91ms

$ pincher do "what containers do I have"

  Executing: "what containers do I have"
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  OK Matched reflex:
    Reflex ID: 1
    Match Type: Exact
    Confidence: 0.9234
    Output: CONTAINER ID   IMAGE          STATUS ...

  Time: Completed in 52.03ms
```

Same reflexes. Same confidence scores. Different machine. No API key required. No internet required. 52 milliseconds.

---

## What's Next

Three things I'm working on:

**WASM Carapace.** Right now reflexes execute as raw shell commands. The endgame is to compile reflexes into WASM modules that run in a sandboxed runtime. This means a reflex compiled on x86_64 Linux runs on ARM64 macOS without recompilation. True write-once-run-anywhere for agent behaviors. The `carapace/` module in the codebase is the skeleton of this — host/guest bridge with capability tokens — but it's not wired up yet.

**Community Reflex Packs.** If reflexes are portable artifacts, you should be able to share them. Imagine: `pincher install github.com/someone/docker-reflexes.nail` and your agent instantly knows how to manage Docker containers. No teaching required. Like package managers, but for learned behaviors.

**Multi-Agent Migration.** The `.nail` format works for one agent. What if you could pack a *fleet*? An agent that manages your smart home, one that monitors your servers, one that reviews your code — all in one `.nail`, all migrating together. The QTR protocol extends naturally to coordinated quiescence and resumption.

---

## The Honest Pitch

PincherOS is a reflex engine with migration. The "OS" is aspirational. The hermit crab branding is because I think crustaceans are cool. But the core idea is sound: **AI agents shouldn't pay an LLM to do something they already know how to do.**

If you're running agents that repeat themselves — and let's be honest, most of us are — reflexes are the answer. Teach once. Execute forever. Move anywhere.

The code is MIT-licensed, written in Rust, and runs on a Raspberry Pi. The [README](https://github.com/SuperInstance/pincherOS) has the full walkthrough. The [architecture docs](https://github.com/SuperInstance/pincherOS/blob/main/docs/architecture.md) explain the deep bits.

Same crab. Bigger shell. 🦀
