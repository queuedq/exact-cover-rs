use std::hash::Hash;
use indexmap::{IndexMap, IndexSet};

/// Base trait for subset names and constraints.
pub trait Value: Clone + Hash + Eq {}
impl<T: Clone + Hash + Eq> Value for T {}

/// An exact cover problem instance.
/// 
/// The subsets are identified with a name with a type `N`.
/// The elements of the set are called constraints and have a type `C`.
/// 
/// # Order
/// 
/// The order of subsets and constraints is determined by the insertion order.
/// It uses [`IndexMap`] and [`IndexSet`] internally to keep track of the order.
/// 
/// The subset order can affect the order of the solutions and
/// the algorithm performance, but it would not be a significant effect
/// as our algorithm uses the MRV heuristic.
#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub struct Problem<N: Value, C: Value> { // TOOD: Constraint will be more complex type
    constraints: IndexSet<C>,
    subsets: IndexMap<N, Vec<C>>,
}

impl<N: Value, C: Value> Default for Problem<N, C> {
    fn default() -> Self {
        Problem {
            constraints: Default::default(),
            subsets: Default::default(),
        }
    }
}

/// An error returned when a subset name parameter
/// given to [`Problem::add_subset`] is already existing.
#[derive(Debug)]
pub struct SubsetExistingError;

impl<N: Value, C: Value> Problem<N, C> {
    // TODO: hide IndexMap/IndexSet from API
    /// Returns a reference to the subsets of the problem.
    pub fn subsets(&self) -> &IndexMap<N, Vec<C>> { &self.subsets }
    /// Returns a reference to the constraints of the problem.
    pub fn constraints(&self) -> &IndexSet<C> { &self.constraints }

    /// Adds a subset to the problem.
    /// 
    /// If the subset name already exists,
    /// it updates the subset of that name with the given new subset.
    pub fn add_subset(&mut self, name: N, subset: Vec<C>) {
        self.subsets.insert(name, subset);
    }
    
    /// Adds a constraint (set element) to the problem.
    pub fn add_constraint(&mut self, constraint: C) {
        self.constraints.insert(constraint);
    }
    
    /// Adds several constraints to the problem.
    pub fn add_constraints<I: IntoIterator<Item = C>>(&mut self, constraints: I) {
        for constraint in constraints {
            self.constraints.insert(constraint);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn problem_can_be_created() {
        let mut prob = Problem::default();
        prob.add_constraints(1..=7);
        prob.add_subset("A", vec![3, 5, 6]);
        prob.add_subset("B", vec![1, 4, 7]);
        prob.add_subset("C", vec![2, 3, 6]);
        prob.add_subset("D", vec![1, 4]);
        prob.add_subset("E", vec![2, 7]);
        prob.add_subset("F", vec![4, 5, 7]);
    }
}
