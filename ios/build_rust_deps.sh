#!/usr/bin/env bash
# Builds the Rust executable for the architecture(s) Xcode is targeting, then
# combines them into the executable Xcode expects.
#
# Based on Bevy's iOS build script (examples/mobile/build_rust_deps.sh).

set -eux

APP_NAME="OxideEngine"
PATH="$PATH:$HOME/.cargo/bin:/opt/homebrew/bin"

PROFILE=debug
RELFLAG=
if [[ "${CONFIGURATION}" != "Debug" ]]; then
    PROFILE=release
    RELFLAG=--release
fi

# Keep Cargo output inside Xcode's derived data, so clean builds are isolated.
export CARGO_TARGET_DIR="${DERIVED_FILE_DIR}/cargo"

# Xcode puts its toolchain first on PATH, which can confuse the Rust linker.
export PATH="/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:${PATH}"

IS_SIMULATOR=0
if [ "${LLVM_TARGET_TRIPLE_SUFFIX-}" = "-simulator" ]; then
    IS_SIMULATOR=1
fi

EXECUTABLES=
for arch in ${ARCHS}; do
    case "${arch}" in
        x86_64)
            if [ "${IS_SIMULATOR}" -eq 0 ]; then
                echo "error: x86_64 is only supported for the iOS simulator" >&2
                exit 2
            fi
            TARGET=x86_64-apple-ios
            ;;
        arm64)
            if [ "${IS_SIMULATOR}" -eq 0 ]; then
                TARGET=aarch64-apple-ios
            else
                TARGET=aarch64-apple-ios-sim
            fi
            ;;
        *)
            echo "error: unsupported iOS architecture ${arch}" >&2
            exit 2
            ;;
    esac

    if ! rustup target list --installed | grep -qx "${TARGET}"; then
        echo "error: Rust target ${TARGET} is not installed." >&2
        echo "       Run: rustup target add ${TARGET}" >&2
        exit 1
    fi

    cargo build ${RELFLAG} --target "${TARGET}" --bin "${APP_NAME}"
    EXECUTABLES="${EXECUTABLES} ${CARGO_TARGET_DIR}/${TARGET}/${PROFILE}/${APP_NAME}"
done

# shellcheck disable=SC2086
lipo -create -output "${TARGET_BUILD_DIR}/${EXECUTABLE_PATH}" ${EXECUTABLES}
