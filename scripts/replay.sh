#!/usr/bin/env bash
# replays a recorded session on the smt4a program and checks its saves
# usage: scripts/replay.sh <rom> <session dir> <out dir>
#   the session dir holds inputs.txt (ZAKURO_RECORD's), data-start/ (the
#   saves the run started from, may be empty) and data/ (the saves it ended
#   with); prints MATCH or DIFF per save, also in <out dir>/done.txt
set -eu
rom=$1 session=$(realpath "$2") out=$3
root=$(cd "$(dirname "$0")/.." && pwd)
exe=$root/target/release/smt4a
[ -x "$exe" ] || { echo "build it first: cargo build --release"; exit 1; }
[ -e "$out" ] && { echo "$out exists, pick another"; exit 1; }
[ -f "$session/inputs.txt" ] || { echo "no recording in $session"; exit 1; }
# enough frames to get past the last recorded input
frames=$(( $(tail -1 "$session/inputs.txt" | cut -d' ' -f1) + 150 ))
mkdir -p "$out"
out=$(realpath "$out")
cp -a "$session/data-start" "$out/data"
ZAKURO_DATA="$out/data" ZAKURO_REPLAY="$session/inputs.txt" \
    /usr/bin/time -v "$exe" replay "$rom" "$frames" > "$out/replay.log" 2>&1 && rc=0 || rc=$?
echo "rc=$rc frames=$frames" > "$out/done.txt"
sd=user/savedata/000400000019A200
for f in "$session/data/$sd"/*; do
    b=$(basename "$f")
    if cmp -s "$f" "$out/data/$sd/$b"; then echo "$b MATCH"; else echo "$b DIFF"; fi
done | tee -a "$out/done.txt"
