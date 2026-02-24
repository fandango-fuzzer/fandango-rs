#!/bin/bash

export RUSTFLAGS="-Znext-solver"

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
cargo build --release --example c_lang --features clang,static_defs

# For unconstrained
# Technically, this will be the same result regardless of whether we use multi- or single-objective
taskset -c 0 cargo run --release --example c_lang --features clang,static_defs -- single unconstrained
mv ./c_lang.txt "${RESULTS}/c_lang_unconstrained_results.txt"

# For multi validity
taskset -c 0 cargo run --release --example c_lang --features clang,static_defs -- multi validity
mv ./c_lang.txt "${RESULTS}/c_lang_multi_valid_results.txt"

# For single validity
taskset -c 0 cargo run --release --example c_lang --features clang,static_defs -- single validity
mv ./c_lang.txt "${RESULTS}/c_lang_single_valid_results.txt"

# For multi validity and size
taskset -c 0 cargo run --release --example c_lang --features clang,static_defs -- multi both
mv ./c_lang.txt "${RESULTS}/c_lang_multi_both_results.txt"

# For single validity and size
taskset -c 0 cargo run --release --example c_lang --features clang,static_defs -- single both
mv ./c_lang.txt "${RESULTS}/c_lang_single_both_results.txt"
