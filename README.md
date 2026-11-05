# smt4a-recomp

![SMT4A on PC](media/cover.webp)

Shin Megami Tensei IV: Apocalypse (USA, title 000400000019A200) as a native
PC program: the game's ARM code recompiled ahead of time into C by
[3dsrecomp](https://github.com/fearkov/3dsrecomp) `port`, linked into one
executable, with [Zakuro](https://github.com/fearkov/zakuro) running the
rest of the console around it. It still reads your ROM for everything but
the code.

Nothing from the game is in this repository. `code/` and the built program
are made from your own copy and hold the game's code: don't share them.

## Build

Needs Rust, a C compiler (gcc), ar and git.

    scripts/build.sh <your decrypted .3ds/.cci>

It builds 3dsrecomp at the pinned version, recompiles the game into `code/`
(about 5 minutes and 2.5 GB of memory on 16 cores), then runs
`cargo build --release`. Run the game with

    target/release/smt4a <your rom>

which takes Zakuro's options (`--help`).

## Widescreen

The game draws a 400x240 picture, whatever shape Zakuro's "Top screen
shape" setting (in its in-game menu) stretches the top screen to.
`overrides/widescreen.c` widens the projections the game builds itself by
the same factor: the 3D gains view at the sides instead of being stretched
(Hor+), and the 2D (menus, HUD, text) is drawn at its own shape in the
middle. Full screen quads (fades, filters, the screen under a menu, the
dialogue backdrop) are spread so they still cover the picture, their
texture coordinates compressed to match. The aspect follows the setting
while the game runs, so that one menu entry does both.

`SMT4A_WIDE=16:9` (or `21:9`) turns it on for a replay, and
`SMT4A_DUMP_AT=frame,...` writes the top screen as `top-<frame>.ppm` into
`SMT4A_DUMP_DIR` (the current directory by default), with the perspective
and quad calls of each dumped frame printed. With the setting at console
the overrides must change nothing: `scripts/replay.sh` has to print
`MATCH`, and the `widescreen` line of its `replay.log` must count 0 for
every hook.

## Hints

`hints/000400000019A200.hints` lists the addresses Zakuro had to interpret
because static analysis didn't find them as functions (indirect calls,
jump tables). `port` turns each one into a recompiled function. The list
was gathered while playing; when the program reaches code that is still
missing, Zakuro interprets it and writes its address to
`~/.local/share/3dsrecomp/000400000019A200.hints`. Merge new ones into
`hints/` and rebuild.

## Checking a build

`scripts/replay.sh <rom> <session dir> <out dir>` plays a recorded session
back headless on the program and compares the saves it writes with the
ones the recorded run wrote, byte for byte. A session dir holds Zakuro's
`ZAKURO_RECORD` inputs as `inputs.txt`, the saves the run started from in
`data-start/`, and the saves it ended with in `data/`.

## Versions

| | |
|---|---|
| Zakuro | `781b4a8` (Cargo.toml) |
| 3dsrecomp, code generation | `73b8f9c` (scripts/build.sh) |
| 3dsrecomp's `recomp-abi`, what Zakuro reads the code with | `73b8f9c` (Cargo.lock), interface version 5 |
