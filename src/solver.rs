use std::thread;
use std::thread::{JoinHandle};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver, TryRecvError, RecvError};
use crate::dlx::{Matrix};
use crate::problem::{Problem, Value};
use crate::callback::Callback;

#[derive(Debug)]
pub enum SolverEvent<N: Value> {
    SolutionFound(Vec<N>),
    ProgressUpdated(f32),
    Paused,
    // TODO: see with_saved_problem
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
    Aborted(Matrix),
}


pub struct Solver<N: Value, C: Value> {
    problem: Problem<N, C>,
    solver_thread: Option<SolverThread>,
}

impl<N: Value, C: Value> Solver<N, C> {
    pub fn new(problem: Problem<N, C>) -> Solver<N, C> {
        Solver {
            problem,
            solver_thread: None,
        }
    }

    pub fn with_saved_problem(_problem: Problem<N, C>) -> Solver<N, C> {
        // `problem` will be of type PartiallySolvedProbem
        todo!()
    }

    fn generate_matrix(problem: &Problem<N, C>) -> Matrix {
        let names = problem.subsets().keys();

        let mut mat = Matrix::new(problem.constraints().len());
        for name in names {
            let row: Vec<_> = problem.subsets()[name].iter().map(|x| {
                // TODO: raise error when constraint is not existent
                problem.constraints().get_index_of(x).unwrap() + 1
            }).collect();
            mat.add_row(&row);
        }
        mat
    }

    fn send_signal(&self, signal: SolverThreadSignal) -> Result<(), ()> {
        let thread = self.solver_thread.as_ref().ok_or(())?;
        thread.send(signal)
    }

    pub fn run(&mut self) -> Result<(), ()> {
        if let Some(thread) = &self.solver_thread {
            thread.send(SolverThreadSignal::Run)
        } else {
            let mat = Solver::generate_matrix(&self.problem);
            self.solver_thread = Some(SolverThread::new(mat));
            Ok(())
        }
    }
    pub fn request_progress(&self) -> Result<(), ()> { self.send_signal(SolverThreadSignal::RequestProgress) }
    pub fn pause(&self) -> Result<(), ()> { self.send_signal(SolverThreadSignal::Pause) }
    pub fn abort(&self) -> Result<(), ()> { self.send_signal(SolverThreadSignal::Abort) }

    fn map_event(&self, event: SolverThreadEvent) -> SolverEvent<N> {
        match event {
            SolverThreadEvent::SolutionFound(sol) => SolverEvent::SolutionFound(
                sol.iter()
                    .map(|x| { self.problem.subsets().get_index(x-1).unwrap().0.clone() })
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
        match self.solver_thread.as_ref()?.recv() {
            Ok(e) => Some(self.map_event(e)),
            Err(_) => None,
        }
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

impl Callback for ThreadCallback {
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
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solver_can_solve_problem() {
        let mut prob = Problem::default();
        prob.add_constraints(1..=3);
        prob.add_subset("A", vec![1, 2, 3]);
        prob.add_subset("B", vec![1]);
        prob.add_subset("C", vec![2]);
        prob.add_subset("D", vec![3]);
        prob.add_subset("E", vec![1, 2]);
        prob.add_subset("F", vec![2, 3]);

        let mut solver = Solver::new(prob);
        solver.run().ok();

        let sol: Vec<_> = solver.filter_map(|e| match e {
            SolverEvent::SolutionFound(s) => Some(s),
            _ => None,
        }).collect();

        assert_eq!(sol.len(), 4);
    }
}
