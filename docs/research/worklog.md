# PincherOS Development Worklog

## 2026-06-03 — Repo Recovery & Document Sync

### Incident
The PincherOS GitHub repo was accidentally overwritten when the workspace-level git repo (at `/home/z/my-project/`) was force-pushed to the same remote (`github.com/SuperInstance/pincherOS`). This replaced the proper project tree with workspace artifacts (skills, uploads, research markdown at root level).

### Recovery
- The original commit `d4b7f93` (fix: get PincherOS compiling and passing all tests) was still accessible via the GitHub API as a dangling commit
- Fetched the commit via `git fetch origin d4b7f93:refs/recover/original`
- Reset `main` to the recovered commit and force-pushed back to GitHub
- All project files restored: README.md, pincher-core, pincher-cli, pincher-infer, docs, examples, skills, assets, config

### Lessons Learned
- The workspace directory (`/home/z/my-project/`) and the project directory (`/home/z/my-project/pincherOS/`) must NOT share the same GitHub remote
- Force-push should only be done after verifying the correct repo context
- All important research docs are tracked under `docs/research/reviews/` and `docs/research/synthesis/`

### Current Repo Structure
```
pincherOS/
├── README.md
├── CONTRIBUTING.md
├── LICENSE
├── Cargo.toml / Cargo.lock
├── rust-toolchain.toml
├── .gitignore
├── assets/          — logo, images
├── config/          — pincher.toml
├── docs/
│   ├── architecture.md
│   ├── developer-guide.md
│   ├── contributing.md
│   ├── nail-format.md
│   ├── threats.md
│   ├── MVP_CHECKLIST.md
│   ├── RISKS.md
│   ├── ROADMAP.md
│   ├── adr/
│   ├── agent/       — CAPABILITIES, INTEGRATION, PROTOCOLS, QUICKSTART, STATE_MACHINE
│   └── research/
│       ├── reviews/   — all R1-R3 research reviews
│       ├── synthesis/ — master synthesis documents
│       ├── kimi-audit/
│       ├── prototypes/ — cognitive & shell Rust prototypes
│       └── rfc-*.md
├── examples/        — code-review, deploy-agent, hello-reflex, migration-demo, smart-home
├── pincher-core/    — core Rust library (engine, reflex, immunology, intent, carapace, etc.)
├── pincher-cli/     — CLI binary
├── pincher-infer/   — Python inference service
├── skills/          — agent skill TOML definitions
└── tools/           — Python tools (deepinfra_client, model_router)
```
