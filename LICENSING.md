# Licensing

This repository contains two independently licensed parts.

## 1. The engine — AGPL-3.0-or-later

Everything that makes up the game itself is licensed under the GNU Affero
General Public License v3.0 or later. That includes:

- the root crate `fnf-oxide-engine` (`src/`, `android/`, `ios/`, `tools/`)
- `crates/oxide_core`, `crates/oxide_data`, `crates/oxide_game`,
  `crates/oxide_ui`, `crates/oxide_mod_runtime`
- the vendored `crates/bevy_animate_atlas` (upstream GPL-3.0; GPL-3.0 code is
  compatible with, and redistributed here under, AGPL-3.0-or-later)
- all game assets and art in `assets/`, which come from *Friday Night Funkin'*
  and remain the property of their original authors

See [`LICENSE`](LICENSE) for the full text.

## 2. The mod SDK — MIT OR Apache-2.0

`crates/oesdk` is a **separate, self-contained crate** that mod authors link
against. It is dual-licensed under MIT or Apache-2.0, at your option:

- [`crates/oesdk/LICENSE-MIT`](crates/oesdk/LICENSE-MIT)
- [`crates/oesdk/LICENSE-APACHE`](crates/oesdk/LICENSE-APACHE)

`crates/oesdk/Cargo.toml` declares its own `license = "MIT OR Apache-2.0"` and
does **not** inherit the workspace license. It must stay free of engine
dependencies; `tools/check-sdk-isolation.sh` fails the build if `oesdk` ever
gains a dependency on `oxide_*`, `fnf_oxide_engine` or `bevy_animate_atlas`. The intent is that the SDK can be
extracted, published, or vendored into a third-party mod without pulling the
AGPL obligations of the engine along with it.

### What this means for mod authors

- A mod that only *uses* `oesdk` (types, `Mod` trait, `Ctx`, the CLI) is not a
  derivative work of the engine; you may license your mod however you like.
- A mod that copies engine source code (for example `oxide_game` internals) is
  a derivative work of the engine and must comply with AGPL-3.0-or-later.
- The engine can load and run your mod regardless of your mod's license; the
  WebAssembly component boundary is a runtime interface, not a linking
  exception.

## Third-party notices

- `crates/bevy_animate_atlas` — vendored from
  <https://github.com/FunkinRustUp/bevy_animate_atlas>, GPL-3.0. Local changes:
  dependency on the workspace Bevy version only.
- Game assets — © Funkin' Crew Inc. and contributors; used here for a
  fan-made engine and not redistributed under either license above.
