use exact_cover::{Problem, Solver, SolverEvent};

fn main() {
    let mut prob = Problem::default();
    prob.add_constraint(1, 1, 1);
    prob.add_constraint(2, 1, 2);
    prob.add_constraint(3, 0, 1);
    prob.add_subset("A", vec![1, 2, 3]);
    prob.add_subset("B", vec![2]);
    prob.add_subset("C", vec![1, 2]);
    prob.add_subset("D", vec![2, 3]);

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
