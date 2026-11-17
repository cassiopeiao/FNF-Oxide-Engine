#!/usr/bin/env bash
# Build the Android app with a JDK that Gradle 8.13 can actually run on.
#
# The distro default may be JDK 25 (class file major version 69), which makes
# Gradle fail while compiling the build scripts:
#   BUG! exception in phase 'semantic analysis' ... Unsupported class file
#   major version 69
# Gradle 8.13 supports running on JDK 17-23, so we pick one of those.
set -euo pipefail
cd "$(dirname "$0")"

java_major() {
  "$1/bin/java" -version 2>&1 \
    | sed -n 's/.*version "\([0-9][0-9]*\).*/\1/p' | head -n1
}

pick_jdk() {
  local root candidate major
  root=$(cd ../../.. && pwd)
  for candidate in \
    "${JAVA_HOME:-}" \
    "$root/.toolchain/jdk21" \
    "$HOME/develop/.toolchain/jdk21" \
    /usr/lib/jvm/java-21-openjdk-amd64 \
    /usr/lib/jvm/java-17-openjdk-amd64 \
    "$HOME/.jdks/temurin-21"; do
    [ -n "$candidate" ] || continue
    [ -x "$candidate/bin/javac" ] || continue   # JRE-only installs cannot compile
    major=$(java_major "$candidate")
    [ -n "$major" ] || continue
    if [ "$major" -ge 17 ] && [ "$major" -le 23 ]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
  return 1
}

if ! JDK=$(pick_jdk); then
  cat >&2 <<'MSG'
error: no JDK 17-23 found.
       Gradle 8.13 cannot run on JDK 24+ (error: Unsupported class file major
       version NN). Install one, or point JAVA_HOME at it, e.g.:
         export JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64
MSG
  exit 1
fi

export JAVA_HOME="$JDK"
export PATH="$JAVA_HOME/bin:$PATH"

# Local, writable caches used by this checkout (optional; falls back to defaults).
root=$(cd ../../.. && pwd)
[ -d "$root/.android-home" ] && export ANDROID_USER_HOME="$root/.android-home"
[ -d "$root/.gradle-home" ] && export GRADLE_USER_HOME="$root/.gradle-home"
[ -d "$root/.cargo-home" ] && export CARGO_HOME="$root/.cargo-home"
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"

echo "JDK:    $JAVA_HOME ($(java_major "$JAVA_HOME"))"
echo "SDK:    $ANDROID_HOME"
exec ./gradlew --no-daemon "$@"
