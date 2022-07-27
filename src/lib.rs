//! Asynchronous [exact cover] solver library using Knuth's [dancing links (DLX)] algorithm.
//! 
//! [exact cover]: https://en.wikipedia.org/wiki/Exact_cover
//! [dancing links (DLX)]: https://en.wikipedia.org/wiki/Dancing_Links
//! 
//! ⚠️ This library is working in progress and the API is highly likely to be changed.
//! 
//! # Concept
//! 
//! Many puzzle-like problems, such as polyomino packing, Sudoku, N-queens problem, etc.
//! can be modeled as exact cover problems. This library provides an efficient solver to
//! the generic exact cover problem and its generalizations, so that you can model your own problem
//! as an exact cover problem, solve it, and further anaylize the solutions by code.
//! 
//! # Basic example
//! 
//! ```
//! use exact_cover::{Problem, Solver, SolverEvent};
//! 
//! fn main() {
//!     let mut prob = Problem::default();
//!     prob.add_exact_constraints(1..=3);
//!     prob.add_subset("A", vec![1, 2, 3]);
//!     prob.add_subset("B", vec![1]);
//!     prob.add_subset("C", vec![2]);
//!     prob.add_subset("D", vec![3]);
//!     prob.add_subset("E", vec![1, 2]);
//!     prob.add_subset("F", vec![2, 3]);
//! 
//!     let mut solver = Solver::new(prob);
//!     let mut solutions = vec![];
//!     solver.run();
//! 
//!     for event in solver {
//!         if let SolverEvent::SolutionFound(sol) = event {
//!             solutions.push(sol);
//!         }
//!     }
//! 
//!     println!("{:?}", solutions); // [["A"], ["B", "C", "D"], ["B", "F"], ["E", "D"]]
//! }
//! ```
//! 
//! # Asynchronous API
//! 
//! ⚠️ The feature is not available yet.
//! 
//! Solving a complex exact cover problem takes a long time.
//! Users don't want to wait for the solving process to end without knowing
//! how far it has progressed or how much time is left.
//! This library provides an asynchronous API and various features to help with this issue.
//! 
//! - Thanks to the asynchronous API, your program doesn't have to wait for the solver
//!   until it finds the next solution.
//! - You can fetch the estimated progress of the solving process, anytime you want.
//! - When the search space is too large and the solving process is not going to end in centuries,
//!   you can abort the solver.
//! - You can pause the solving process and save the solver state to resume later.

pub mod vector;

pub mod dlx;
pub mod problem;
pub mod solver;

pub mod problems;

pub use problem::Problem;
pub use solver::{Solver, SolverEvent};
