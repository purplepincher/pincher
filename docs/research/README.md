# Research Directory

This directory contains forward-looking research, design reviews, and
synthesis documents.  Items here are **not** on the critical path for the
12-week MVP unless explicitly marked as MVP blockers.

## Directory Layout

```
docs/research/
├── README.md                 ← you are here
├── RESEARCH_STATUS.yaml      ← authoritative RFC status tracker
├── DESIGN_GEMS.md            ← reusable design patterns extracted from reviews
├── rfcs/                     ← Request for Comments (exploratory / deferred / abandoned)
│   ├── rfc-jepa-integration.md
│   ├── rfc-penrose-memory.md
│   ├── rfc-command-dynamics-mlp.md
│   ├── rfc-no-std-core.md
│   ├── rfc-constitutional-governance.md
│   └── rfc-polyformal-synthesis.md
├── reviews/                  ← multi-perspective design reviews (the canonical copies)
│   ├── pincheros-*.md        ← reviewed outputs from ideation cycles
│   └── PincherOS_*.md        ← synthesis & formalism documents
├── synthesis/                ← cross-cutting research synthesis
│   ├── PincherOS_Master_Research_Synthesis.md
│   ├── PincherOS_MVP_Architecture_Spec.md
│   └── ...
└── prototypes/               ← Rust prototype code (not part of main crate)
    ├── cognitive/            ← context, workspace, phantom, gastrolith
    └── shell/                ← CRDT, thermodynamics, migration, governance, claws
```

## RFC Status

See [RESEARCH_STATUS.yaml](./RESEARCH_STATUS.yaml) for the authoritative list.

| Status | Meaning |
|--------|---------|
| Exploratory | Worth investigating, no timeline |
| Deferred | Good idea, waiting for data or capacity |
| Abandoned | Investigated and rejected |

## Naming Conventions

- **`rfc-*`** — formal proposals not yet implemented
- **`pincheros-*`** — ideation-cycle design reviews
- **`PincherOS_*`** — synthesis / formalism / spec documents
