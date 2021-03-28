use std::hash::Hash;
use std::collections::{HashMap, HashSet};

pub trait Value: Copy + Hash + Eq {}
impl<T: Copy + Hash + Eq> Value for T {}

#[derive(Debug, Default)]
pub struct Problem<N: Value, C: Value> { // TOOD: Constraint will be more complex type
    constraints: HashSet<C>,
    subsets: HashMap<N, Vec<C>>,
}

impl<N: Value, C: Value> Problem<N, C> {
    pub fn add_subset(&mut self, name: N, elements: &[C]) {
        if self.subsets.contains_key(&name) { return } // TODO: Raise error
        self.subsets.insert(name, elements.to_vec());
    }

    pub fn add_constraints<I: IntoIterator<Item = C>>(&mut self, constraints: I) {
        for constraint in constraints {
            self.constraints.insert(constraint);
        }
    }

    pub fn add_constraint(&mut self, constraint: C) {
        self.constraints.insert(constraint);
    }

    pub fn subsets(&self) -> &HashMap<N, Vec<C>> { &self.subsets }
    pub fn constraints(&self) -> &HashSet<C> { &self.constraints }
    pub fn constraints_mut(&mut self) -> &mut HashSet<C> { &mut self.constraints }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn problem_can_be_created() {
        let mut prob = Problem::default();
        prob.add_subset("A", &[3, 5, 6]);
        prob.add_subset("B", &[1, 4, 7]);
        prob.add_subset("C", &[2, 3, 6]);
        prob.add_subset("D", &[1, 4]);
        prob.add_subset("E", &[2, 7]);
        prob.add_subset("F", &[4, 5, 7]);
    }
}
