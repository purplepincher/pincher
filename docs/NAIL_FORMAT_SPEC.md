# .nail Format Specification (verified against implementation)

This document describes the `.nail` format as it is **actually
implemented** in `pincher-core/src/migration/pack.rs` and
`unpack.rs`, verified by reading the source directly (no Rust
toolchain was available to build/run this project — verification is by
code reading only). A pre-existing doc, `docs/nail-format.md`,
describes a similar but not identical format and contains at least one
field (`format_version`) that does not exist anywhere in the real
implementation — treat this document as authoritative over that one
until it's reconciled or removed.

## File structure

A `.nail` file is a `tar.zst` archive (`pack.rs:255`,
`create_tar_zst`) containing exactly four files, staged in a temp
directory before archiving (`pack.rs:194-256`, `pack_nail`):

| File | Format | Contents |
|---|---|---|
| `manifest.json` | JSON | `NailManifest` struct — see below |
| `reflexes.db` | SQLite (binary copy) | Direct copy of the source reflex database (`pack.rs:212`, `fs::copy`) |
| `identity.json` | JSON | `AgentIdentity` struct — see below |
| `config.toml` | TOML | `AgentConfig` struct — see below |

## `manifest.json` — `NailManifest` (`pack.rs:52-63`)

```rust
pub struct NailManifest {
    pub version: String,       // hardcoded "0.1.0" at pack time (pack.rs:242)
    pub fingerprint: String,   // hardware fingerprint of the source shell
    pub timestamp: String,     // RFC3339, pack time
    pub reflex_count: u64,     // counted from reflexes.db at pack time
    pub checksums: NailChecksums,
}

pub struct NailChecksums {
    pub reflexes_db: String,   // BLAKE3 hex digest
    pub identity_json: String, // BLAKE3 hex digest
    pub config_toml: String,   // BLAKE3 hex digest
}
```

Checksums are computed with BLAKE3 (`pack.rs:160-180`,
`blake3_hash_file`) over each staged file's bytes before archiving.

## `identity.json` — `AgentIdentity` (`pack.rs:78-94`)

```rust
pub struct AgentIdentity {
    pub name: String,               // default: "Pinchy"
    pub preferences: AgentPreferences,
    pub created_at: String,         // RFC3339
}

pub struct AgentPreferences {
    pub preferred_shell: String,    // default: "bash"
    pub preferred_editor: String,   // default: "vim"
    pub language: String,           // default: "en"
    pub verbosity: u8,              // default: 1, range implied 0-2
}
```

**Note**: `pack_nail` always writes `AgentIdentity::default()`
(`pack.rs:216-217`) — it does not currently read or preserve any
existing identity from the source shell. Any real agent name/
preferences set before packing are not carried into the `.nail` file
as implemented today; this is either a real gap or a not-yet-wired
feature, not something the current code contradicts, so it's reported
here rather than assumed intentional.

## `config.toml` — `AgentConfig` (`pack.rs:123-158`)

```rust
pub struct AgentConfig {
    pub resource_thresholds: ResourceThresholdsConfig,
    pub veto_rules_path: String,    // default: ".pincher/veto_rules.toml"
    pub model_path: String,         // default: ".pincher/models/all-MiniLM-L6-v2-int8.onnx"
    pub rpc_socket_path: String,    // default: "/tmp/pincher.sock"
}

pub struct ResourceThresholdsConfig {
    pub ram_light: f64,     // default: 70.0
    pub ram_critical: f64,  // default: 85.0
    pub cpu_light: f64,     // default: 60.0
    pub cpu_critical: f64,  // default: 80.0
}
```

Same note as identity: `pack_nail` writes `AgentConfig::default()`
(`pack.rs:225`), not the source shell's actual live config.

## Versioning — a real, currently-unenforced gap

`manifest.version` is written as the literal string `"0.1.0"`
(`pack.rs:242`) — there is no format-version-negotiation scheme, and
no field resembling the pre-existing doc's claimed `"format_version": 1`
exists anywhere in this code.

More importantly: `unpack_nail` (`unpack.rs:262-329`) **never reads or
checks `manifest.version` at all** — it parses the manifest purely to
recover the checksums for integrity verification (see below), and
does nothing with the version string. **There is currently no version
compatibility check on unpack.** A `.nail` file packed by a future,
incompatible version of this format would be accepted and unpacked
without any warning.

## Integrity verification on unpack

`unpack_nail` extracts the archive, then — only if `manifest.json`
exists in the extracted output — recomputes the BLAKE3 hash of each of
`reflexes.db`, `identity.json`, and `config.toml`, and compares against
the manifest's recorded checksums, returning `PackError::ChecksumMismatch`
on any mismatch (`unpack.rs:283-321`). **If `manifest.json` is
missing**, verification is silently skipped entirely (a `warn!` log
line is emitted, but unpacking still succeeds) — a `.nail` archive
without a manifest is accepted as-is, unverified.

## Test coverage

`pincher-core`'s test suite includes
`migration::pack::tests::test_pack_unpack_roundtrip` and
`test_verify_nail` (both passing per this repo's CI, per the separate
`docs/prep-notes/SANDBOX_TEST_FINDING.md` investigation) — these
exercise the pack→unpack round-trip and the standalone `verify_nail`
function respectively. This spec's structural claims above are
consistent with what those tests exercise, based on reading the test
file names and the functions they call; the test bodies themselves
weren't traced line-by-line for this document.

## Known limitations, as implemented today

1. **No version compatibility check on unpack** (see above) — a real
   gap, not a documentation error, worth fixing before this format is
   relied upon across format revisions.
2. **Identity and config are not preserved across pack/unpack** —
   `pack_nail` always writes fresh defaults rather than the source
   shell's actual state for these two files. Only `reflexes.db` (the
   actual learned reflex data) round-trips faithfully.
3. **Manifest-less archives unpack without verification**, silently.
