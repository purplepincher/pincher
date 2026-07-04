# Task: remove the orphaned files shipping a hardcoded secret

**No Rust toolchain is available in this environment.** This task is
scoped so that shouldn't matter: you're removing files that are already
confirmed unreferenced, not editing compiled code, so there's nothing
to compile-check.

## Context

The crates.io publish-readiness audit (`docs/CRATES_IO_READINESS.md`)
flagged `pincher-core/src/daemon.rs`, `registry.rs`, and `updater.rs`
as "not referenced by `pincher-core/src/lib.rs`... but would still be
included in the published tarball," and specifically flagged
`updater.rs:129`'s hardcoded secret:
`b"SUPER_INSTANCE_SHARED_SECRET_KEY_FOR_NAIL_INTEGRITY"`.

This has already been independently re-verified before this task was
written: `pincher-core/src/lib.rs` declares its modules with `pub mod
...;` lines near the top of the file — `daemon`, `registry`, and
`updater` are **not** among them. A repo-wide grep for `mod updater`,
`mod daemon`, `mod registry`, and any `#[path = ...]` attribute that
might reference these files unusually, and a check of
`pincher-core/Cargo.toml` for any `[[bin]]` target that might use them
as a separate crate root, all came back empty. **These three files are
not part of the compiled crate in any way** — they're orphaned source
text that would ship in a published tarball without ever being built.

## What to do

1. **Verify this independently yourself first** — don't just trust this
   brief. Re-run the same checks: confirm `daemon`/`registry`/`updater`
   are absent from `pincher-core/src/lib.rs`'s module declarations,
   confirm no `#[path]` attribute or `[[bin]]` target references them.
   If you find something this brief missed and one of these files
   genuinely IS reachable some way, **stop and report that instead of
   deleting** — don't remove a file that turns out to be load-bearing.
2. If your independent check confirms they're truly orphaned: delete
   `pincher-core/src/daemon.rs`, `pincher-core/src/registry.rs`, and
   `pincher-core/src/updater.rs`. This removes the hardcoded secret
   from the published surface entirely (the cleanest fix — there's no
   reason to "fix" a secret embedded in code nothing runs), and
   resolves the readiness audit's dead-code concern in the same move.
3. Update `docs/CRATES_IO_READINESS.md`'s `pincher-core` table: change
   the "Hardcoded secrets / dead code" row from 🔴 needs fix to ✅
   resolved, with a one-line note that the files were removed (not
   gated) since they were confirmed unreachable.

## Scope

Do not touch any other file. Do not attempt to "fix" the secret by
replacing it with an environment variable or similar — that would imply
this code is meant to run, which it currently isn't (it's not wired
into the crate at all). If you think there's a reason someone might
want this functionality back later, note that in your commit message,
but still remove it — dead code with a hardcoded secret sitting in a
soon-to-be-public crate is a real liability regardless of future intent,
and it can always be recovered from git history if genuinely needed.
