#!/usr/bin/env bash
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
: "${ANDROID_NDK_HOME:?Set ANDROID_NDK_HOME to your Android NDK directory}"
rustup target add aarch64-linux-android
cargo install cargo-ndk --locked
cargo ndk -t arm64-v8a -P 26 build --release --manifest-path "$HERE/Cargo.toml"
LIB="$HERE/target/aarch64-linux-android/release/libak_prototype.so"
test -f "$LIB"
OUT="$HERE/target/pack"
rm -rf "$OUT"
mkdir -p "$OUT/bin/aarch64-linux-android"
cp "$LIB" "$OUT/bin/aarch64-linux-android/libak_prototype.so"
cp "$HERE/horizon-mod.toml" "$HERE/horizon-native.json" "$OUT/"
cp -r "$HERE/data" "$HERE/assets" "$OUT/"
python3 - "$OUT/horizon-native.json" <<'PY'
import json,sys
p=sys.argv[1]; d=json.load(open(p))
d["binaries"]={"aarch64-linux-android":"bin/aarch64-linux-android/libak_prototype.so"}
json.dump(d,open(p,"w"),indent=2)
PY
python3 - "$OUT" "$HERE/target/ak_prototype.hnmod" <<'PY'
import os,sys,zipfile
src,dst=sys.argv[1:]
with zipfile.ZipFile(dst,"w",zipfile.ZIP_DEFLATED) as z:
    for root,dirs,files in os.walk(src):
        dirs.sort()
        for name in sorted(files):
            p=os.path.join(root,name); z.write(p,os.path.relpath(p,src))
print("Built",dst,os.path.getsize(dst),"bytes")
PY
