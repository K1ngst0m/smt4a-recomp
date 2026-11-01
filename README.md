# smt4a-recomp

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
| Zakuro | `80d78be` (Cargo.toml) |
| 3dsrecomp, code generation | `9db0d8b` (scripts/build.sh) |
| 3dsrecomp's `recomp-abi`, what Zakuro reads the code with | `cec1270` (Cargo.lock), interface version 4 |

3dsrecomp's main is at interface version 5 now; the pinned Zakuro doesn't
build against it, so the lock keeps `recomp-abi` at `cec1270`.
