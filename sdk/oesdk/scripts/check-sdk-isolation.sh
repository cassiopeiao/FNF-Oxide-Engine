#!/usr/bin/env bash
# Fails if any crate under sdk/ depends on an engine crate.
set -euo pipefail
cd "$(dirname "$0")/../../.."
bad=$(cargo tree -p oesdk -p oesdk_api -p oesdk_macro -p oesdk_cli \
    --prefix none --no-dedupe --offline \
  | grep -E '^(oxide_[a-z_]+|fnf_oxide_engine|bevy_animate_atlas)' || true)
if [ -n "$bad" ]; then
    echo "engine crates leaked into the SDK:" >&2
    echo "$bad" >&2
    exit 1
fi
echo "ok: sdk dependency graph is clean"
