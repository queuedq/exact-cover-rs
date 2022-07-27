//! Provides a solver that solves a generic [`Problem`].

use std::thread;
use std::thread::{JoinHandle};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver, TryRecvError, RecvError};
use crate::dlx::callback::{Callback};
// use crate::dlx::dlx::{Matrix};
use crate::dlx::dlx_m::{Matrix};
use crate::problem::{Problem, Value};

/// Events that a solver emits.
pub enum SolverEvent<N: Value> {
    SolutionFound(Vec<N>),
    ProgressUpdated(f32),
    Paused,
    Aborted(Matrix), // Solver can resume from here later
    Finished,
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
    _Aborted(Matrix),
    Finished,
}

/// A solver for a [`Problem`] instance.
pub struct Solver<N: Value, E: Value> {
    problem: Problem<N, E>,
    solver_thread: Option<SolverThread>,
}

impl<N: Value, E: Value> Solver<N, E> {
    /// Creates a new solver that solves `problem`.
    pub fn new(problem: Problem<N, E>) -> Solver<N, E> {
        Solver {
            problem,
            solver_thread: None,
        }
    }
    
    pub fn generate_matrix(problem: &Problem<N, E>) -> Matrix {
        // TODO: validate problem
        Solver::generate_multi_matrix(problem)
    }

    // TODO: use original algorithm if applicable

    // fn generate_exact_matrix(problem: &Problem<N, E>) -> Matrix {
    //     let constraints = problem.constraints();
    //     let names = problem.subsets().keys();
    //     let mut mat = Matrix::new(constraints.len());

    //     for name in names {
    //         let row: Vec<_> = problem.subsets()[name].iter()
    //             .map(|e| { constraints.get_index_of(e).unwrap() + 1 })
    //             .collect();
    //         mat.add_row(&row);
    //     }
    //     mat
    // }

    fn generate_multi_matrix(problem: &Problem<N, E>) -> Matrix {
        let constraints = problem.constraints();
        let names = problem.subsets().keys();
        let mut mat = Matrix::new(constraints.len());

        for (e, &(min, max)) in constraints {
            mat.set_multiplicity(constraints.get_index_of(e).unwrap() + 1, min, max);
        }

        for name in names {
            let row: Vec<_> = problem.subsets()[name].iter()
                .map(|e| { constraints.get_index_of(e).unwrap() + 1 })
                .collect();
            mat.add_row(&row);
        }
        mat
    }

    fn send_signal(&self, signal: SolverThreadSignal) -> Result<(), ()> {
        let thread = self.solver_thread.as_ref().ok_or(())?;
        thread.send(signal)
    }

    /// Runs the solver thread.
    pub fn run(&mut self) {
        // TODO: where should I handle thread SendError?
        if let Some(thread) = &self.solver_thread {
            thread.send(SolverThreadSignal::Run).ok();
        } else {
            let mat = Solver::generate_matrix(&self.problem);
            self.solver_thread = Some(SolverThread::new(mat));
        }
    }
    pub fn request_progress(&self) { self.send_signal(SolverThreadSignal::RequestProgress).ok(); }
    pub fn pause(&self) { self.send_signal(SolverThreadSignal::Pause).ok(); }
    pub fn abort(&self) { self.send_signal(SolverThreadSignal::Abort).ok(); }

    fn map_event(&self, event: SolverThreadEvent) -> SolverEvent<N> {
        match event {
            SolverThreadEvent::SolutionFound(sol) => SolverEvent::SolutionFound(
                sol.iter()
                    .map(|x| { self.problem.subsets().get_index(x-1).unwrap().0.clone() })
                    .collect()
            ),
            SolverThreadEvent::ProgressUpdated(progress) => SolverEvent::ProgressUpdated(progress),
            SolverThreadEvent::Paused => SolverEvent::Paused,
            SolverThreadEvent::_Aborted(mat) => SolverEvent::Aborted(mat),
            SolverThreadEvent::Finished => SolverEvent::Finished,
        }
    }
}

/// An iterator of [`SolverEvent`]s that a solver emits.
pub struct SolverIter<N: Value, E: Value> {
    solver: Solver<N, E>,
}

impl<N: Value, E: Value> Iterator for SolverIter<N, E> {
    type Item = SolverEvent<N>;

    fn next(&mut self) -> Option<SolverEvent<N>> {
        if let Ok(e) = self.solver.solver_thread.as_ref()?.recv() {
            Some(self.solver.map_event(e))
        } else {
            None
        }
    }
}

// TODO: also provide stream
impl<N: Value, E: Value> IntoIterator for Solver<N, E> {
    type Item = SolverEvent<N>;
    type IntoIter = SolverIter<N, E>;

    /// Returns an iterator of [`SolverEvent`]s that a solver emits.
    fn into_iter(self) -> Self::IntoIter {
        SolverIter { solver: self }
    }
}


/// Represents a running thread.
struct SolverThread {
    tx_signal: Sender<SolverThreadSignal>,
    rx_event: Receiver<SolverThreadEvent>,
    _thread: JoinHandle<()>, // TODO: do I need it?
}

impl SolverThread {
    // TODO: terminate thread on drop 
    fn new(mut mat: Matrix) -> SolverThread {
        let (tx_signal, rx_signal) = mpsc::channel();
        let (tx_event, rx_event) = mpsc::channel();
        
        let mut callback = ThreadCallback::new(rx_signal, tx_event);
        let thread = thread::spawn(move || { mat.solve(&mut callback); });
        
        SolverThread {
            tx_signal,
            rx_event,
            _thread: thread,
        }
    }

    fn send(&self, signal: SolverThreadSignal) -> Result<(), ()> {
        // TODO: Handle signals after the thread is terminated
        // e.g. what happens when it gets RequestProgress after thread is finished?
        self.tx_signal.send(signal).map_err(|_| {()})
    }

    fn recv(&self) -> Result<SolverThreadEvent, RecvError> {
        // TODO: Emit "Finished" event when the DLX algorithm has terminated successfully
        self.rx_event.recv()
    }
}

struct ThreadCallback {
    signal: Receiver<SolverThreadSignal>,
    event: Sender<SolverThreadEvent>,
}

impl ThreadCallback {
    fn new(
        signal: Receiver<SolverThreadSignal>,
        event: Sender<SolverThreadEvent>,
    ) -> ThreadCallback {
        ThreadCallback { signal, event }
    }

    fn update_progress(&self) {
        // TODO: implement progress update (in dlx)
        self.event.send(SolverThreadEvent::ProgressUpdated(0.0)).ok();
        todo!()
    }

    // Returns a signal received while paused.
    fn pause(&self) -> SolverThreadSignal {
        self.event.send(SolverThreadEvent::Paused).ok();
        loop {
            match self.signal.recv() {
                Ok(SolverThreadSignal::Run) => break SolverThreadSignal::Run,
                Ok(SolverThreadSignal::RequestProgress) => (),
                Ok(SolverThreadSignal::Pause) => (),
                Ok(SolverThreadSignal::Abort) => break SolverThreadSignal::Abort,
                Err(RecvError) => break SolverThreadSignal::Abort,
            }
        }
    }
}

impl Callback<Matrix> for ThreadCallback {
    fn on_solution(&mut self, sol: Vec<usize>, _mat: &mut Matrix) {
        self.event.send(SolverThreadEvent::SolutionFound(sol)).ok();
    }
    
    fn on_iteration(&mut self, mat: &mut Matrix) {
        let mut pause_signal = None; // signal received while paused

        let abort = loop {
            let signal = match pause_signal {
                Some(s) => Ok(s),
                None => self.signal.try_recv(),
            };
            pause_signal = None;

            match signal {
                Ok(SolverThreadSignal::Run) => (),
                Ok(SolverThreadSignal::RequestProgress) => self.update_progress(),
                Ok(SolverThreadSignal::Pause) => pause_signal = Some(self.pause()),
                Ok(SolverThreadSignal::Abort) => break true,
                Err(TryRecvError::Disconnected) => break true,
                Err(TryRecvError::Empty) => break false,
            }
        };

        if abort { mat.abort(); }
    }

    fn on_abort(&mut self, _mat: &mut Matrix) {
        // TODO: write matrix serialization code
        // self.event.send(SolverThreadEvent::Aborted(mat.serialize()));
    }

    fn on_finish(&mut self) {
        self.event.send(SolverThreadEvent::Finished).ok();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solver_can_solve_problem() {
        let mut prob = Problem::default();
        prob.add_exact_constraints(1..=3);
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

        assert_eq!(solutions.len(), 4);
    }

    #[test]
    fn solver_can_solve_problem_with_multiplicity() {
        let mut prob = Problem::default();
        prob.add_constraint(1, 0, 1);
        prob.add_constraint(2, 0, 1);
        prob.add_subset("B", vec![1]);
        prob.add_subset("C", vec![2]);

        let mut solver = Solver::new(prob);
        let mut solutions = vec![];
        solver.run();
        
        for event in solver {
            if let SolverEvent::SolutionFound(sol) = event {
                solutions.push(sol);
            }
        }
        
        assert_eq!(solutions.len(), 4);
    }
}
