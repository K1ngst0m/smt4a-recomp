#!/usr/bin/env bash
# builds smt4a from your own copy of Shin Megami Tensei IV: Apocalypse (USA)
# usage: scripts/build.sh <decrypted .3ds/.cci> [--force]
#   1. builds 3dsrecomp at the version pinned below, in .build/
#   2. recompiles the game's code into code/ with the hints in hints/ and the
#      functions written by hand in overrides/ (a few minutes, about 2.5 GB
#      of memory), skipped when code/ is up to date
#   3. cargo build --release, target/release/smt4a
# nothing it makes is meant to be shared: code/ and the program hold the game's code
set -eu
RECOMP_URL=https://github.com/fearkov/3dsrecomp
# the 3dsrecomp the code was generated and tested with (interface version 5,
# the one the pinned Zakuro's recomp-abi reads)
RECOMP_REV=73b8f9c
TITLE=000400000019A200
rom=$(realpath "$1") force=${2:-}
root=$(cd "$(dirname "$0")/.." && pwd)
work=$root/.build
mkdir -p "$work"

[ "$(dd if="$rom" bs=1 skip=256 count=4 status=none)" = NCSD ] ||
    { echo "$rom is not a decrypted .3ds/.cci"; exit 1; }
id=$(od -An -tx8 -j 0x108 -N 8 "$rom" | tr -d ' ' | tr a-f A-F)
[ "$id" = "$TITLE" ] || { echo "$rom is title $id, this is for $TITLE"; exit 1; }

echo "== 3dsrecomp $RECOMP_REV"
[ -d "$work/3dsrecomp/.git" ] || git clone --quiet "$RECOMP_URL" "$work/3dsrecomp"
git -C "$work/3dsrecomp" rev-parse --verify --quiet "$RECOMP_REV^{commit}" >/dev/null ||
    git -C "$work/3dsrecomp" fetch --quiet origin
git -C "$work/3dsrecomp" checkout --quiet "$RECOMP_REV"
(cd "$work/3dsrecomp" && cargo build --release --quiet)
recomp=$work/3dsrecomp/target/release/3dsrecomp

echo "== recompiling the game's code"
stamp=$( { echo "$RECOMP_REV"; stat -c %s "$rom"; cat "$root"/hints/*.hints "$root"/overrides/*.c; } | sha256sum | cut -d' ' -f1)
if [ "$force" != --force ] && [ -f "$root/code/librecomp.a" ] && [ "$(cat "$root/code/.stamp" 2>/dev/null)" = "$stamp" ]; then
    echo "code/ is up to date, skipping (--force redoes it)"
else
    # 3dsrecomp reads a title's hints from where Zakuro writes them, so hand
    # it this project's through a data dir of its own
    rm -rf "$work/xdg" "$work/port"
    mkdir -p "$work/xdg/3dsrecomp"
    cp "$root/hints/$TITLE.hints" "$work/xdg/3dsrecomp/"
    XDG_DATA_HOME=$work/xdg "$recomp" port "$rom" "$work/port" --name smt4a --overrides "$root/overrides"
    rm -rf "$root/code"
    mv "$work/port/code" "$root/code"
    rm -rf "$work/port"
    echo "$stamp" > "$root/code/.stamp"
fi

echo "== building the program"
(cd "$root" && cargo build --release)
echo "run it with: $root/target/release/smt4a \"$rom\""
