#!/bin/bash

cd "eval/baselines" || exit 1

for target in csv rest scriptsizec xml; do
  cargo clean
  echo "Building ${target}..."
  time cargo build --quiet -p "${target}" "$@"
done
