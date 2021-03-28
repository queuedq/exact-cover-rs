use indexmap::{IndexSet};
use crate::dlx::{Matrix};
use crate::problem::{Problem, Value};

#[derive(Debug)]
pub struct Solver<N: Value, C: Value> {
    problem: Problem<N, C>,
    matrix: Matrix,

    names: IndexSet<N>, // TODO: abstract it to be able to use more efficient alternative
    constraints: IndexSet<C>,
}

impl<N: Value, C: Value> Solver<N, C> {
    pub fn new(problem: Problem<N, C>) -> Solver<N, C> {
        let names: IndexSet<_> = problem.subsets().keys().cloned().collect();
        let constraints: IndexSet<_> = problem.constraints().iter().cloned().collect();

        let mut matrix = Matrix::new(problem.constraints().len());
        for name in &names {
            let row: Vec<_> = problem.subsets()[name].iter().map(|x| {
                // TODO: raise error when constraint is not existent
                constraints.get_index_of(x).unwrap() + 1
            }).collect();
            matrix.add_row(&row);
        }
        
        Solver { problem, matrix, names, constraints }
    }

    pub fn solve(&mut self) -> Vec<Vec<N>> { // TODO: Use iterator
        self.matrix.search().iter().map(|ans| {
            ans.iter().map(|i| *self.names.get_index(i - 1).unwrap()).collect()
        }).collect()
    }
}

impl<N: Value, C: Value> Solver<N, C> {
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solver_can_solve_problem() {
        let mut prob = Problem::default();
        prob.add_constraints(1..=7);
        prob.add_subset("A", &[3, 5, 6]);
        prob.add_subset("B", &[1, 4, 7]);
        prob.add_subset("C", &[2, 3, 6]);
        prob.add_subset("D", &[1, 4]);
        prob.add_subset("E", &[2, 7]);
        prob.add_subset("F", &[4, 5, 7]);
        
        let mut solver = Solver::new(prob);
        assert_eq!(solver.solve().len(), 1);
    }
}
