## #2372 - Run release heavy tests before the debug suites

**Decided:** Keep the two build steps first. Put the unchanged heavy steps before the debug suites in the release plan. The executor first drains the heavy shared queue. It then runs the heavy isolated steps alone. It next drains the ordinary shared queue and runs the ordinary isolated steps alone. Each shared queue keeps the existing longest-first order. A private helper in `queue.rs` identifies heavy steps from `--release` and `--ignored`. The rules in `lanes.rs` stay unchanged.

**Why:** The five completed cut gate runs have two red steps. Both are `app heavy: pocket3d_hsm`. The runs took 66.4 and 91.1 minutes. Moving the heavy phase first makes that class of failure visible earlier. The issue comment records the artifact counts. The batch history contains no heavy tests and does not establish an overall failure rate. One asset lookup failure also accounts for 194 red post-merge steps. Thus raw red counts are not independent defects.

**Kept:** The step names, selectors, arguments, step count, job budget, isolation rules, and failure accumulation stay unchanged. A failed test does not prevent the later phases from running. A failed build still stops the run.

**Considered and dropped:** Moving plan entries alone does not control execution because the scheduler separates shared and isolated steps. Dynamic failure scores need a defined history window. They must distinguish repeated failures and shared causes. The small cut sample supports a fixed heavy phase instead. The optional cut workflow input is outside this patch and outside the allowed file scope.

**Cost to change later:** Change the phase classification and the order tests. A future failure score can sort within a phase without changing step names or isolation. No product data or G-code changes.
