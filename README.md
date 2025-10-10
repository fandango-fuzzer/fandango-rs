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
\begin{table}
    \centering
    \caption{Results of \tool{} and \fandango{} generating C programs until one minute is surpassed.
    }
    \label{tab:compiler-test-summary}
    \begin{threeparttable}
    \begin{tabular}{crrrrr}
        \toprule
         & \multicolumn{5}{c}{\tool{}} \\
          \cmidrule(l{0.25em}r{0.25em}){2-6}
        Objectives & \# Prog. Gen. & $k$-path ($k=5$) & Gen. Time & {\tt gcc} Time & Total Time \\
        \midrule
        $\varnothing$ & \unconstrainedTotalAvgRs & \unconstrainedKPathCoverageAvgRs & \unconstrainedGenTimeAvgRs & \unconstrainedCompileTimeAvgRs & \unconstrainedTotalTimeAvgRs \\
        Validity & \validConstrainedCompileAvgRs & \validConstrainedKPathCoverageAvgRs & \validConstrainedGenTimeAvgRs & \validConstrainedCompileTimeAvgRs & \validConstrainedTotalTimeAvgRs \\
        Validity $\cup$ Generation & \validAndSizedConstrainedCompileAvgRs & \validAndSizedConstrainedKPathCoverageAvgRs & \validAndSizedConstrainedGenTimeAvgRs & \validAndSizedConstrainedCompileTimeAvgRs & \validAndSizedConstrainedTotalTimeAvgRs \\
        \midrule
         & \multicolumn{5}{c}{\fandango{}} \\
          \cmidrule(l{0.25em}r{0.25em}){2-6}
        Objectives & \# Prog. Gen. & $k$-path ($k=5$) & Gen. Time & {\tt gcc} Time & Total Time \\
        \midrule
        $\varnothing$ & \unconstrainedTotalAvgPy & \unconstrainedKPathCoverageAvgPy & \unconstrainedGenTimeAvgPy & \unconstrainedCompileTimeAvgPy & \unconstrainedTotalTimeAvgPy \\
        Validity & \validConstrainedCompileAvgPy & \validConstrainedKPathCoverageAvgPy & \validConstrainedGenTimeAvgPy & \validConstrainedCompileTimeAvgPy & \validConstrainedTotalTimeAvgPy \\
        Validity $\cup$ Generation & \textit{n.d.}\tnote{3} & \textit{n.d.}\tnote{3} & \textit{n.d.}\tnote{3} & \textit{n.d.}\tnote{3} & \textit{n.d.}\tnote{4} \\
        \bottomrule
    \end{tabular}
    \begin{tablenotes}
        \item[3] Unable to produce any inputs.
        \item[4] Process self-aborted before producing any inputs.
    \end{tablenotes}
    \end{threeparttable}
\end{table}
```

These macros are generated automatically by running our compiler evaluation script from the root directory of the project.
To see the macros, inspect the generated files in `compiler-testing-results` (the folder is re-generated each time the script is run).

```
./compiler_experiment.sh
```

To see the results and see the results to fill the table:

- `cat compiler-testing-results/c_lang_unconstrained_results.txt` for the unconstrained fandango-rs experiment;
- `cat compiler-testing-results/c_lang_validity_only_results.txt` for the fandango-rs experiment with validity constraints;
- `cat compiler-testing-results/c_lang_validity_and_size_results.txt` for the fandango-rs experiment with generation goals and validity constraints;
- `cat compiler-testing-results/c_lang_unconstrained_results_python.txt` for the unconstrained experiment with original fandango;
- `cat compiler-testing-results/c_lang_validity_only_results_python.txt` for the validity constrained experiment with original fandango.

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

## Notice on use of generative AI

This repository was written nearly entirely without AI.
Generative code assistance was used as part of the Section 5 evaluation, but ultimately did not have any specific interesting findings to include in the paper about this.
Files which were in any part produced with generative code assistance are clearly marked at the start of the file.

## Licensing

This crate is licensed under EUPL v1.2.

Some sections of the code are adaptations of [py_literal](https://github.com/jturner314/py_literal/releases/tag/0.4.0),
which is licensed under Apache and MIT, for which license files are provided
in [core/src/py_literal](fandango/src/py_literal).
