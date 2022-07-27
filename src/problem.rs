//! Provides a generic problem type that defines constraints and subsets.
//! 
//! Every complex exact cover problem (such as polyomino packing or Sudoku) first generates
//! this basic [`Problem`] instance before handing it to a solver.
//! To see examples of more complex problems, see [`problems`](crate::problems) module.

use std::hash::Hash;
use indexmap::{IndexMap};

/// Base trait for subset names and set elements.
pub trait Value: Clone + Hash + Eq {}
impl<T: Clone + Hash + Eq> Value for T {}

/// An exact cover problem instance.
/// 
/// The set elements are of type `E`.
/// They form constraints together with a multiplicity range.
/// The subsets are identified by names of type `N`.
/// 
/// # Ordering
/// 
/// The order of the subsets and the elements is determined by the insertion order.
/// It uses [`IndexMap`] internally to keep track of the order.
/// The subset order may affect the order of the solutions.
#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub struct Problem<N: Value, E: Value> {
    constraints: IndexMap<E, (usize, usize)>,
    subsets: IndexMap<N, Vec<E>>,
}

impl<N: Value, E: Value> Default for Problem<N, E> {
    fn default() -> Problem<N, E> {
        Problem { constraints: Default::default(), subsets: Default::default() }
    }
}

impl<N: Value, E: Value> Problem<N, E> {
    // TODO: hide IndexMap/IndexSet from API
    /// Returns a reference to the constraints of the problem.
    pub fn constraints(&self) -> &IndexMap<E, (usize, usize)> { &self.constraints }
    /// Returns a reference to the subsets of the problem.
    pub fn subsets(&self) -> &IndexMap<N, Vec<E>> { &self.subsets }

    /// Adds a subset to the problem.
    /// 
    /// If the subset name already exists, it replaces the corresponding subset.
    pub fn add_subset(&mut self, name: N, subset: Vec<E>) {
        self.subsets.insert(name, subset);
    }

    /// Adds a constraint with a multiplicity range.
    pub fn add_constraint(&mut self, elem: E, min: usize, max: usize) {
        self.constraints.insert(elem, (min, max));
    }

    /// Adds a constraint that has to be covered exactly once.
    pub fn add_exact_constraint(&mut self, elem: E) {
        self.add_constraint(elem, 1, 1);
    }
    
    /// Adds several exact constraints.
    pub fn add_exact_constraints<I: IntoIterator<Item = E>>(&mut self, constraints: I) {
        for constraint in constraints {
            self.add_exact_constraint(constraint);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn problem_can_be_created() {
        let mut prob = Problem::default();
        prob.add_exact_constraints(1..=7);
        prob.add_subset("A", vec![3, 5, 6]);
        prob.add_subset("B", vec![1, 4, 7]);
        prob.add_subset("C", vec![2, 3, 6]);
        prob.add_subset("D", vec![1, 4]);
        prob.add_subset("E", vec![2, 7]);
        prob.add_subset("F", vec![4, 5, 7]);
    }
}
