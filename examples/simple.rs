use exact_cover::{Problem, Solver, SolverEvent};

fn main() {
    let mut prob = Problem::default();
    prob.add_constraints(1..=3);
    prob.add_subset("A", vec![1, 2, 3]);
    prob.add_subset("B", vec![1]);
    prob.add_subset("C", vec![2]);
    prob.add_subset("D", vec![3]);
    prob.add_subset("E", vec![1, 2]);
    prob.add_subset("F", vec![2, 3]);

    let mut solver = Solver::new(prob);
    let mut solutions = vec![];
    solver.run();

    for event in solver {
        if let SolverEvent::SolutionFound(sol) = event {
            solutions.push(sol);
        }
    }

    println!("{:?}", solutions);
}
