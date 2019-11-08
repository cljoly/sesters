#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

OUTDIR="target/cov"

cargo build
cargo test --no-run

if [ -d "$OUTDIR" ]; then
    rm -r "$OUTDIR"/*
fi
if [ -f "$OUTDIR" ]; then
    rmdir "$OUTDIR"
fi
mkdir -p "$OUTDIR"

for file in target/debug/sesters-*
do
    [ -x "${file}" ] || continue;
    mkdir -p "$OUTDIR"/"$(basename "$file")"
    kcov --exclude-pattern=/.cargo,/usr/lib --verify "$OUTDIR"/"$(basename "$file")" "$file"
done

