#!/bin/bash

cd "baselines" || exit 1

for target in csv rest scriptsizec xml; do
  cargo clean
  echo "Building ${target} (only dynamic, no optimizations)..."
  time cargo build --quiet -p "${target}" --profile bench-noopt
done


for target in csv rest scriptsizec xml; do
  cargo clean
  echo "Building ${target} (only dynamic)..."
  time cargo build --quiet -p "${target}" --profile bench
done

for target in csv rest scriptsizec xml; do
  cargo clean
  echo "Building ${target} (no optimizations)..."
  time cargo build --quiet -p "${target}" --features static_defs --profile bench-noopt
done

for target in csv rest scriptsizec xml; do
  cargo clean
  echo "Building ${target}..."
  time cargo build --quiet -p "${target}" --features static_defs --profile bench
done
