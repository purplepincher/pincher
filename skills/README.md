# PincherOS Starter Skills

This directory contains **starter skill definitions** for PincherOS agents.
Each skill is a self-contained TOML + Markdown pair that can be loaded into
the reflex vector database and matched against user intents at runtime.

## Why Skills?

PincherOS treats the LLM as a compiler, not a runtime.  Skills are the
**pre-compiled output** — ready-made reflex templates that an agent can use
without ever calling the LLM.  By bundling a starter set, a fresh PincherOS
installation can handle common tasks from day one.

These skill definitions are also designed to be **vectorDB-native**: each one
includes a canonical intent phrase, tags, and a description optimized for
embedding-based retrieval.

## Skill Format

Each skill consists of two files:

```
skills/
├── <skill-name>.toml    # Machine-readable skill contract
└── <skill-name>.md      # Human-readable documentation / prompt template
```

### TOML Schema

```toml
[skill]
name = "system-monitor"
version = "0.1.0"
description = "Monitor system resources and report status"
category = "system"           # system | network | file | devops | data | agent
tags = ["monitoring", "resources", "status", "health"]

[skill.intent]
canonical = "check system resource usage"
variations = [
    "how much RAM am I using",
    "show me CPU usage",
    "system status",
    "is memory running low",
]

[skill.capabilities]
required = ["subprocess"]     # Capabilities the reflex needs
optional = ["network"]        # Nice-to-have capabilities

[skill.reflex]
action_template = "free -h && df -h && uptime"
confidence_seed = 0.70        # Starting confidence for the reflex
resource_mode = "light"       # normal | light | critical
```

## Bundled Skills

| Skill | Category | Description |
|-------|----------|-------------|
| `system-monitor` | system | Check RAM, CPU, disk, and uptime |
| `file-search` | file | Find files by name, content, or pattern |
| `package-manage` | system | Install, remove, and query packages |
| `git-ops` | devops | Common git operations and status |
| `docker-ops` | devops | Container management commands |
| `text-process` | data | Text transformation and analysis |
| `agent-teach` | agent | Teach the agent a new reflex |
| `agent-migrate` | agent | Pack/unpack agent state for migration |

## Using Skills

### Load all starter skills into a fresh agent:

```bash
pincher skill-load --all skills/
```

### Load a single skill:

```bash
pincher skill-load skills/system-monitor.toml
```

### List loaded skills:

```bash
pincher reflexes --category starter
```

## Creating Custom Skills

1. Create a new `<name>.toml` following the schema above
2. Create a matching `<name>.md` with usage examples and edge cases
3. Test with `pincher skill-validate <name>.toml`
4. Load with `pincher skill-load <name>.toml`

Custom skills are great for:
- Team-specific workflows (e.g., "deploy to staging")
- Domain-specific queries (e.g., "query the patient database")
- Environment-specific commands (e.g., "restart the pi service")

## vectorDB Integration

The `intent.canonical` and `intent.variations` fields are designed for
embedding and indexing in a vector database.  The skill loader:

1. Embeds each variation using the configured embedder (ONNX MiniLM or hash)
2. Stores the resulting vectors in sqlite-vec alongside the reflex database
3. Associates each vector with the skill's action template and metadata

This means skills are **first-class citizens** in the reflex matching pipeline
— they compete with user-taught reflexes for intent matching, and their
confidence scores evolve the same way.
