# exact-cover-rs

(WIP) Asynchronous exact cover solver library using Knuth's dancing links algorithm.

⚠️ This library is working in progress and the API is highly likely to be changed. Please use other libraries such as [dlx](https://crates.io/crates/dlx) or [dancing-links](https://crates.io/crates/dancing-links) for the time being.

## Goals

⚠️ Most of the features mentioned here are not implemented yet, but they are the primary goals of this project.

- **Asynchronous API.** Solving a complex exact cover problem takes a long time. Users don't want to wait for the solving process to end without knowing how far it has progressed or how much time is left. This library provides an asynchronous API and various features to help with this issue.
    - Thanks to the asynchronous API, you can do other things while the solver process is running on the other thread.
    - You can fetch the estimated progress of the solving process.
    - When the problem is too complex and the solving process is not going to end in centuries, you can abort the solver.
    - You can pause the solving process and save the solver state to resume later.
    - All the above can be done in code, not just by a user interruption.
- **Solving generalized exact cover problems (color/multiplicity)**
