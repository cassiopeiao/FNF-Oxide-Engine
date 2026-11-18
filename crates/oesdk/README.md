# oesdk

Mod SDK and build tool for **FNF: Oxide Engine**.

A mod is a WebAssembly *component* plus a manifest, packed into a signed
`.fnfm` container. This crate provides:

* `oesdk` (lib) — the Rust guest API mods link against. It is an OOP facade in
  the shape modders know from v-slice (`Character`, `Camera`, `PlayState`), but
  it never exposes an ECS type: every call is a host import.
* `oesdk` (bin) — the CLI: scaffold, validate, (later) compile, pack and sign.

The ABI is defined exactly once, in [`wit/oesdk.wit`](wit/oesdk.wit). Rust,
C/C++ and TypeScript shells are generated from that file, so the contract cannot
drift between languages.

## Status

| Part | State |
| --- | --- |
| `wit/oesdk.wit` | drafted (v0.1.0), review before freezing |
| Rust guest API | types + dispatch work; wit-bindgen glue is a TODO |
| CLI `new` / `inspect` / `build` (validation) | working |
| CLI `pack` / `sign` / `verify` | TODO (zstd frames + Ed25519 + Merkle root) |
| `oxide_mod_runtime` (host) | `crates/oxide_mod_runtime` in this workspace, wasmtime configured |

## Quick start

```bash
cargo run --bin oesdk -- new ../my-mod
cd ../my-mod
cargo run --bin oesdk -- build .
```

A mod looks like this:

```rust
use oesdk::prelude::*;

#[derive(Default)]
struct MyMod;

impl Mod for MyMod {
    fn on_note_hit(&mut self, ctx: &mut Ctx<'_>, hit: &Hit) {
        if hit.judgement == Judgement::Sick {
            ctx.camera().shake(0.05, 0.2);
        }
    }
}

oesdk::register_mod!(MyMod);
```

## Design rules

1. **No ECS in the ABI.** No `Entity`, no `Component`, no `Handle`.
2. **Characters are addressed by name or by `(side, slot)`**, never by a global
   `boyfriend`/`opponent` singleton — a stage may hold several characters per
   side (needed for versus mode).
3. **Writes are buffered.** `Camera::shake` and friends enqueue a command that
   the host applies at a safe point; reads may return snapshots immediately.
4. **A failing mod is disabled, not fatal.** Traps are caught by the host.

## Layout

```
oesdk/
├── wit/oesdk.wit      # the contract
├── src/lib.rs         # guest API
├── src/bin/oesdk.rs   # CLI
├── examples/basic.rs  # runs a mod against a mock host
└── LICENSE-{MIT,APACHE}
```

## License

Dual-licensed under **MIT OR Apache-2.0**. The engine itself is
AGPL-3.0-or-later; a permissively licensed SDK lets mod authors ship their work
under whatever terms they prefer.

## Publishing

Only `oesdk` is published. Every other crate in the workspace is marked
`publish = false` so an accidental `cargo publish --workspace` cannot leak the
engine (AGPL) to crates.io.

### Dependency isolation

`oesdk` must stay free of engine crates, otherwise the MIT/Apache license would
be contaminated by the workspace's AGPL. The guard enforces it:

```bash
./scripts/check-sdk-isolation.sh
# ok: oesdk dependency graph is clean
#   oesdk v0.1.0
```

### Release checklist

```bash
# 0. clean tree (crates.io rejects dirty packages by default)
git add -A && git commit -m "oesdk: v0.1.1"

# 1. bump the version in crates/oesdk/Cargo.toml
#    (a published version can never be reused, even after yanking)

# 2. inspect exactly what will be uploaded
cargo package -p oesdk --list

# 3. build the packaged crate and validate metadata
cargo publish -p oesdk --dry-run --registry crates-io

# 4. authenticate once per machine (token from crates.io > Account Settings)
cargo login --registry crates-io

# 5. publish
cargo publish -p oesdk --registry crates-io
```

Notes:

- `--registry crates-io` is required here because `~/.cargo/config.toml` replaces
  crates.io with the USTC mirror for dependency downloads; without the flag
  Cargo refuses with `crates-io is replaced with non-remote-registry source`.
  The flag affects only the upload target, not the mirror used for builds.
- `cargo package` only picks up files listed in `include` (sources, WIT,
  examples, README, both LICENSE files). Engine code and `assets/` can never
  end up in the archive.
- docs.rs builds automatically for every published version; the
  `[package.metadata.docs.rs]` table enables `--all-features`.
- Versions are permanent: to fix a mistake, publish `0.1.1`; use `cargo yank
  --version 0.1.0` to hide a broken release without freeing the number.
- Keep the crates.io token scoped to `publish-new` + `publish-update` for this
  crate only, and store it in `~/.cargo/credentials.toml` rather than in the
  repo.

## Modding API design

See [docs/modding-api.md](docs/modding-api.md) for the stage / character /
song / module contracts modelled on the FNF 0.8 `.hxc` scripted-class API.

## Content schema

See [`docs/content-schema.md`](docs/content-schema.md): every JSON registry in the
original game mapped to a TOML file with identical field names.
