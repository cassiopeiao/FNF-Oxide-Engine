#!/usr/bin/env bash
# Guard for the license boundary: `oesdk` is published under MIT OR Apache-2.0
# and must never link an AGPL engine crate. Run this before `cargo publish`.
set -euo pipefail
cd "$(dirname "$0")/../../.."

tree=$(cargo tree -p oesdk --prefix none --no-dedupe --offline 2>/dev/null \
  | sed 's/ (.*//' | sed '/^$/d' | sort -u)

bad=$(printf '%s\n' "$tree" \
  | grep -E '^(oxide_[a-z_]+|fnf_oxide_engine|bevy_animate_atlas|oesdk_host)' || true)

if [ -n "$bad" ]; then
  echo "error: oesdk must not depend on engine crates (license contamination):" >&2
  printf '  %s\n' $bad >&2
  exit 1
fi

echo "ok: oesdk dependency graph is clean"
printf '  %s\n' "$tree"
