use exact_cover::{Problem, Solver};

fn main() {
    let mut prob = Problem::default();
    prob.add_constraints(1..=7);
    prob.add_subset("A", &[3, 5, 6]);
    prob.add_subset("B", &[1, 4, 7]);
    prob.add_subset("C", &[2, 3, 6]);
    prob.add_subset("D", &[1, 4]);
    prob.add_subset("E", &[2, 7]);
    prob.add_subset("F", &[4, 5, 7]);
    println!("{:?}", prob);
    
    let mut solver = Solver::new(prob);
    println!("{:?}", solver);
    println!("{:?}", solver.solve());
}
