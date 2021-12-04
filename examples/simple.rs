use exact_cover::{Problem, Solver, SolverEvent};

fn main() {
    let mut prob = Problem::default();
    prob.add_constraints(1..=3);
    prob.add_subset("A", &[1, 2, 3]);
    prob.add_subset("B", &[1]);
    prob.add_subset("C", &[2]);
    prob.add_subset("D", &[3]);
    prob.add_subset("E", &[1, 2]);
    prob.add_subset("F", &[2, 3]);

    let mut solver = Solver::new(prob);
    solver.run().ok();

    let sol: Vec<_> = solver.filter_map(|e| match e {
        SolverEvent::SolutionFound(s) => Some(s),
        _ => None,
    }).collect();

    println!("{:?}", sol);
}
