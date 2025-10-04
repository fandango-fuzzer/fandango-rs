#!/bin/bash

set -e
set -m

mkdir profiling-results
RESULTS=$(realpath profiling-results)

git submodule update --init --recursive
cd fandango-prof

# dependencies
if ! [ -d ".venv" ]; then
  git apply ../prof.diff
  virtualenv .venv
  . .venv/bin/activate
  make install
  pip install docutils tccbox
else
  . .venv/bin/activate
fi

for trial in $(seq 1 5); do
  for target in {csv,rest,scriptsizec,xml}; do
    echo "Beginning trial ${trial} for subject ${target}"
    rm -f profile.json
    mkdir -p "${RESULTS}/${target}/${trial}"
    (time taskset -c 0 env PYTHONPATH=. python "evaluation/vs_isla/${target}_evaluation/${target}_evaluation.py") |& tee "${RESULTS}/${target}/${trial}/experiment_output.txt"
    mv profile.json "${RESULTS}/${target}/${trial}/profile.json"
  done
done
