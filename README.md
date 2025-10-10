# fandango-rs

This commit represents the artifact for the OOPSLA submission.
Thanks for checking it out!
We're quite proud of it, and happy that you're working with our code.

## Evaluation

Sections 4 and 5 of the paper include a number of experiments.
All section 4 tables may be reproduced by entering the `baselines/` folder and executing `profile.sh`.
You'll need to have Rust and Python 3 already installed for this to work.

The table presented in section 5 is as follows:
```tex
TODO
```

You may execute TODO in TODO to retrieve the corresponding variables to fill this table.

## Navigating this repository

There are a number of individual software packages provided here, summarized below:

- *core*: This is the core typing, parsing, generation, and visitor logic for fandango-rs.
  If you want to know how we prep the type system for generated types, look here.
- *generator* and *derive*: These implement the code generation logic and integration with Rust, respectively.
  The code in generator is a little messy, sorry; it needs a good refactor yet.
- *runtime*: This contains the implementations of (an approximate version of) the original FANDANGO algorithm and NSGA-II.
  There are quite a few type system shenanigans going on to make this work that aren't documented in high detail, sorry again.
- *targets*: These are target-specific visitor implementations which enable the evaluations in sections 4 and 5.
  You may inspect how we implement grammars specifically here.
- *baselines*: This is the benchmarking code for fandango-rs.
  In addition to crates which are specific to each target, we offer two benchmarks, `criterion` and `models`, which benchmark in different ways.
  `criterion` benchmarks were for iterative development, to make sure that we did not worsen performance between versions.
  `models` is what is used in the paper to compare against FANDANGO.
  The script `profile.sh` in this directory runs the Section 4 evaluation.
- *lm3s6965-demo*: Not all the cool things made it into the paper.
  This directory contains an example of running fandango-rs within a baremetal firmware.

## Licensing

This crate is licensed under EUPL v1.2.

Some sections of the code are adaptations of [py_literal](https://github.com/jturner314/py_literal/releases/tag/0.4.0),
which is licensed under Apache and MIT, for which license files are provided
in [core/src/py_literal](fandango/src/py_literal).
