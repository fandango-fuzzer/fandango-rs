#!/bin/bash

# Remove compiler-testing-results if it exists
rm -rf compiler-testing-results

set -e
set -m

mkdir compiler-testing-results
RESULTS=$(realpath compiler-testing-results)

#
# fandango (Python) side
#

cd compiler-testing-fandango-py

# Setup
python -m venv .venv
. .venv/bin/activate
pip install fandango-fuzzer==1.0.0

# Run experiments

taskset -c 0 python lang_unconstrained.py
mv ./c_lang_unconstrained_results_python.txt "${RESULTS}/c_lang_unconstrained_results_python.txt"

taskset -c 0 python lang_validity_only.py
mv ./c_lang_validity_only_results_python.txt "${RESULTS}/c_lang_validity_only_results_python.txt"

# This experiment dramatically times out.
# taskset -c 0 python lang_validity_and_size.py
# mv ./c_lang_validity_and_size_results_python.txt "${RESULTS}/c_lang_validity_and_size_results_python.txt"

# Undo venv
deactivate

#
# fandango-rs side
#

cd ../..
cargo build --release --example c_lang_unconstrained --example c_lang_validity_only --example c_lang_validity_and_size --features clang,static_defs

# For unconstrained
taskset -c 0 cargo run --release --example c_lang_unconstrained --features clang,static_defs
mv ./c_lang_unconstrained_results.txt "${RESULTS}/c_lang_unconstrained_results.txt"

# For validity only
taskset -c 0 cargo run --release --example c_lang_validity_only --features clang,static_defs
mv ./c_lang_validity_only_results.txt "${RESULTS}/c_lang_validity_only_results.txt"

# For validity + size
taskset -c 0 cargo run --release --example c_lang_validity_and_size --features clang,static_defs
mv ./c_lang_validity_and_size_results.txt "${RESULTS}/c_lang_validity_and_size_results.txt"
