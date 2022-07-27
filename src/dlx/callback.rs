//! Basic low-level callback objects to pass to the DLX algorithm.

/// An interface of callback objects to pass to the DLX algorithm.
pub trait Callback<M> {
    fn on_solution(&mut self, _sol: Vec<usize>, _mat: &mut M) {}
    fn on_iteration(&mut self, _mat: &mut M) {}
    fn on_abort(&mut self, _mat: &mut M) {}
    fn on_finish(&mut self) {}
}

/// A simple callback that just collects solutions into a vector.
pub struct SolutionCallback {
    pub solutions: Vec<Vec<usize>>,
}

impl Default for SolutionCallback {
    fn default() -> SolutionCallback {
        SolutionCallback { solutions: vec![] }
    }
}

impl<M> Callback<M> for SolutionCallback {
    fn on_solution(&mut self, sol: Vec<usize>, _mat: &mut M) {
        self.solutions.push(sol);
    }
}
