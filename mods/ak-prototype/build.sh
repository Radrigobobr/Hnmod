#!/usr/bin/env bash
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
SDK="$ROOT/horizon-sdk"

: "${ANDROID_NDK_HOME:?ANDROID_NDK_HOME is not set}"

if [ ! -f "$SDK/crates/horizon-mod-sdk/Cargo.toml" ]; then
    echo "Horizon SDK not found: $SDK"
    exit 1
fi

HORIZON_REV="$(git -C "$SDK" rev-parse HEAD)"

rustup target add aarch64-linux-android
cargo install cargo-ndk --locked

cd "$HERE"

cargo ndk -t arm64-v8a -P 26 build --release

LIB="$HERE/target/aarch64-linux-android/release/libak_prototype.so"
test -f "$LIB"

OUT="$HERE/target/pack"
rm -rf "$OUT"

mkdir -p "$OUT/bin/aarch64-linux-android"

cp "$LIB" "$OUT/bin/aarch64-linux-android/libak_prototype.so"
cp "$HERE/horizon-native.json" "$OUT/"
cp "$HERE/horizon-mod.toml" "$OUT/"
cp -r "$HERE/assets" "$HERE/data" "$OUT/"

python3 - "$OUT/horizon-native.json" "$HORIZON_REV" <<'PY'
import json, sys

path, revision = sys.argv[1:]
with open(path, encoding="utf-8") as f:
    data = json.load(f)

data["horizon"] = revision
data["binaries"] = {
    "aarch64-linux-android":
    "bin/aarch64-linux-android/libak_prototype.so"
}

with open(path, "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2)
PY

python3 - "$OUT" "$HERE/target/ak_prototype.hnmod" <<'PY'
import os, sys, zipfile

src, dst = sys.argv[1:]

with zipfile.ZipFile(dst, "w", zipfile.ZIP_DEFLATED) as z:
    for root, dirs, files in os.walk(src):
        dirs.sort()
        for name in sorted(files):
            path = os.path.join(root, name)
            z.write(path, os.path.relpath(path, src))

with zipfile.ZipFile(dst) as z:
    bad = z.testzip()
    if bad:
        raise SystemExit("Archive error: " + bad)

    required = [
        "horizon-native.json",
        "horizon-mod.toml",
        "bin/aarch64-linux-android/libak_prototype.so",
    ]

    for name in required:
        if name not in z.namelist():
            raise SystemExit("Missing: " + name)

print("Built:", dst)
print("Size:", os.path.getsize(dst), "bytes")
PY
