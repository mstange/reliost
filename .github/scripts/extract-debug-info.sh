#!/usr/bin/env bash
# Extract debug info from the dist binary, generate Breakpad symbols, strip the
# binary, and repack the distribution archive with the stripped binary.
#
# Usage: extract-debug-info.sh <target> <git-rev> <source-root>
#
# Arguments:
#   target       - Rust target triple, e.g. x86_64-unknown-linux-gnu
#   git-rev      - Full git commit hash for symbol path mapping
#   source-root  - Absolute path to the repository checkout (GITHUB_WORKSPACE)
#
# Required environment variables:
#   GITHUB_OUTPUT - Path to the GitHub Actions output file (set by the runner)

set -euo pipefail

TARGET="$1"
GIT_REV="$2"
SOURCE_ROOT="$3"

BINARY="target/${TARGET}/dist/reliost"
DBG_FILE="target/distrib/reliost-${TARGET}.so.dbg"
SYM_FILE="target/distrib/reliost-${TARGET}.sym"

# Generate Breakpad symbols before stripping
dump_syms "$BINARY" \
    --inlines \
    --mapping-var="rev=${GIT_REV}" \
    --mapping-src="${SOURCE_ROOT}/(.*)" \
    --mapping-dest="git:github.com/mstange/reliost:{1}:{rev}" \
    > "$SYM_FILE"

# Extract debug info into a separate file
objcopy --only-keep-debug "$BINARY" "$DBG_FILE"

# Strip debug info from the binary in-place, adding a link to the debug file
objcopy --strip-debug --add-gnu-debuglink="$DBG_FILE" "$BINARY"

# Repack the distribution archive with the stripped binary
ARCHIVE=$(ls target/distrib/reliost-*-${TARGET}.tar.xz)
TMPDIR=$(mktemp -d)
tar -C "$TMPDIR" -xJf "$ARCHIVE"
find "$TMPDIR" -name reliost -type f -exec cp "$BINARY" {} \;
SUBDIR=$(ls "$TMPDIR")
tar -C "$TMPDIR" -cJf "$ARCHIVE" "$SUBDIR"
rm -rf "$TMPDIR"

echo "paths<<EOF" >> "$GITHUB_OUTPUT"
echo "$DBG_FILE" >> "$GITHUB_OUTPUT"
echo "$SYM_FILE" >> "$GITHUB_OUTPUT"
echo "EOF" >> "$GITHUB_OUTPUT"
