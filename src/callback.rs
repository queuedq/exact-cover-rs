//! Basic low-level callback objects to pass to the DLX algorithm.

use crate::dlx::{Matrix, Callback};

#[derive(Default)]
pub struct SolutionCallback {
    pub solutions: Vec<Vec<usize>>,
}

impl Callback for SolutionCallback {
    fn on_solution(&mut self, sol: Vec<usize>, _mat: &mut Matrix) {
        self.solutions.push(sol);
    }
}
