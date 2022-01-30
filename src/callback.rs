use crate::dlx::{Matrix};

pub trait Callback {
    fn on_solution(&mut self, _sol: Vec<usize>, _mat: &mut Matrix) {}
    fn on_iteration(&mut self, _mat: &mut Matrix) {}
    fn on_abort(&mut self, _mat: &mut Matrix) {}
}

#[derive(Default)]
pub struct SolutionCallback {
    pub solutions: Vec<Vec<usize>>,
}

impl Callback for SolutionCallback {
    fn on_solution(&mut self, sol: Vec<usize>, _mat: &mut Matrix) {
        self.solutions.push(sol);
    }
}
