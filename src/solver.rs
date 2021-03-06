use std::thread;
use std::thread::{JoinHandle, Result as ThreadResult};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver, TryRecvError, RecvError};
use indexmap::{IndexSet};
use crate::dlx::{Matrix};
use crate::problem::{Problem, Value};


#[derive(Debug)]
pub enum SolverEvent<N: Value> {
    SolutionFound(Vec<N>),
    ProgressUpdated(f32),
    Paused,
    // TODO: see with_saved_problem
    Aborted(Matrix), // Solver can resume from here later
}

enum SolverThreadSignal {
    Run,
    RequestProgress,
    Pause,
    Abort,
}

enum SolverThreadEvent {
    SolutionFound(Vec<usize>),
    ProgressUpdated(f32),
    Paused,
    Aborted(Matrix),
}


pub struct Solver<N: Value, C: Value> {
    problem: Problem<N, C>,
    names: IndexSet<N>,
    solver_thread: SolverThread,
}

impl<N: Value, C: Value> Solver<N, C> {
    pub fn new(problem: Problem<N, C>) -> Solver<N, C> {
        let names: IndexSet<_> = problem.subsets().keys().cloned().collect();
        let constraints: IndexSet<_> = problem.constraints().clone();
        let mat = Solver::generate_matrix(&problem, &names, &constraints);
        let solver_thread = SolverThread::new(mat);

        Solver {
            problem,
            names,
            solver_thread,
        }
    }

    pub fn with_saved_problem(_problem: Problem<N, C>) -> Solver<N, C> {
        // `problem` will be of type PartiallySolvedProbem
        todo!()
    }

    fn generate_matrix(
        problem: &Problem<N, C>, names: &IndexSet<N>, constraints: &IndexSet<C>
    ) -> Matrix {
        let mut mat = Matrix::new(constraints.len());
        for name in names {
            let row: Vec<_> = problem.subsets()[name].iter().map(|x| {
                // TODO: raise error when constraint is not existent
                constraints.get_index_of(x).unwrap() + 1
            }).collect();
            mat.add_row(&row);
        }
        mat
    }

    pub fn run(&self) -> Result<(), ()> {
        self.solver_thread.send(SolverThreadSignal::Run)
    }
    
    pub fn request_progress(&self) -> Result<(), ()> {
        self.solver_thread.send(SolverThreadSignal::RequestProgress)
    }

    pub fn pause(&self) -> Result<(), ()> {
        self.solver_thread.send(SolverThreadSignal::Pause)
    }
    
    pub fn abort(&self) -> Result<(), ()> {
        self.solver_thread.send(SolverThreadSignal::Abort)
    }

    fn map_event(&self, event: SolverThreadEvent) -> SolverEvent<N> {
        match event {
            SolverThreadEvent::SolutionFound(sol) => SolverEvent::SolutionFound(
                sol.iter()
                    .map(|x| { self.names.get_index(x-1).unwrap().clone() })
                    .collect()
            ),
            SolverThreadEvent::ProgressUpdated(progress) => SolverEvent::ProgressUpdated(progress),
            SolverThreadEvent::Paused => SolverEvent::Paused,
            SolverThreadEvent::Aborted(mat) => SolverEvent::Aborted(mat),
        }
    }
}

// TODO: use stream instead of iterator
impl<N: Value, C: Value> Iterator for Solver<N, C> {
    type Item = SolverEvent<N>;

    fn next(&mut self) -> Option<SolverEvent<N>> {
        match self.solver_thread.recv() {
            Ok(e) => Some(self.map_event(e)),
            Err(_) => None,
        }
    }
}


struct SolverThread {
    tx_signal: Sender<SolverThreadSignal>,
    rx_event: Receiver<SolverThreadEvent>,
    thread: JoinHandle<()>,
}

impl SolverThread {
    pub fn new(mut mat: Matrix) -> SolverThread {
        let (tx_signal, rx_signal) = mpsc::channel();
        let (tx_event, rx_event) = mpsc::channel();

        // TODO: use proper return type instead of bool (in dlx)
        let callback = move |_mat: &Matrix, sol: Option<Vec<usize>>| -> bool {
            // Send events
            if let Some(s) = sol {
                tx_event.send(SolverThreadEvent::SolutionFound(s)).ok();
            }

            let update_progress = || {
                // TODO: implement progress update (in dlx)
                tx_event.send(SolverThreadEvent::ProgressUpdated(0.0)).ok();
                todo!()
            };

            let pause = || -> SolverThreadSignal {
                tx_event.send(SolverThreadEvent::Paused).ok();
                loop {
                    match rx_signal.recv() {
                        Ok(SolverThreadSignal::Run) => break SolverThreadSignal::Run,
                        Ok(SolverThreadSignal::Abort) => break SolverThreadSignal::Abort,
                        _ => (),
                    }
                }
            };

            // Handle pending signals
            loop {
                match rx_signal.try_recv() {
                    Ok(SolverThreadSignal::Run) => (),
                    Ok(SolverThreadSignal::RequestProgress) => update_progress(),
                    Ok(SolverThreadSignal::Pause) => {
                        if let SolverThreadSignal::Abort = pause() { break false }
                    },
                    Ok(SolverThreadSignal::Abort) => break false,
                    Err(_) => break true,
                }
            }
        };

        // TODO: what happens when it gets RequestProgress after thread is finished?
        let thread = thread::spawn(move || { mat.solve(callback); });
        
        SolverThread {
            tx_signal,
            rx_event,
            thread,
        }
    }

    pub fn send(&self, signal: SolverThreadSignal) -> Result<(), ()> {
        self.tx_signal.send(signal).map_err(|_| {()})
    }

    pub fn try_recv(&self) -> Result<SolverThreadEvent, TryRecvError> {
        self.rx_event.try_recv()
    }

    pub fn recv(&self) -> Result<SolverThreadEvent, RecvError> {
        self.rx_event.recv()
    }

    pub fn join(self) -> ThreadResult<()> {
        self.thread.join()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solver_can_solve_problem() {
        let mut prob = Problem::default();
        prob.add_constraints(1..=3);
        prob.add_subset("A", &[1, 2, 3]);
        prob.add_subset("B", &[1]);
        prob.add_subset("C", &[2]);
        prob.add_subset("D", &[3]);
        prob.add_subset("E", &[1, 2]);
        prob.add_subset("F", &[2, 3]);

        let solver = Solver::new(prob);
        solver.run().ok();

        let sol: Vec<_> = solver.filter_map(|e| match e {
            SolverEvent::SolutionFound(s) => Some(s),
            _ => None,
        }).collect();

        assert_eq!(sol.len(), 4);
    }
}
