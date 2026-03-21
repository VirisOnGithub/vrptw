use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use crate::optimizing_algorithm::OptimizationAlgorithm;
use crate::problem::{Problem, Solution};

enum RunnerCommand {
    Step { iterations: usize },
    Stop,
}

#[derive(Clone)]
pub struct RunnerUpdate {
    pub solution: Solution,
    pub iterations_done: usize,
    pub total_iterations: usize,
    pub is_finished: bool,
}

pub struct AlgorithmRunner {
    command_tx: Sender<RunnerCommand>,
    update_rx: Receiver<RunnerUpdate>,
    worker_handle: Option<JoinHandle<()>>,
    awaiting_update: bool,
    is_finished: bool,
}

impl AlgorithmRunner {
    /// creates two channels for communication between threads :
    /// RunnerCommand sends data from gui to worker
    /// RunnerUpdate sends data from worker to gui
    pub fn new(mut algo: Box<dyn OptimizationAlgorithm + Send + Sync>, problem: Problem) -> Self {
        let (command_tx, command_rx) = mpsc::channel::<RunnerCommand>();
        let (update_tx, update_rx) = mpsc::channel::<RunnerUpdate>();

        let worker_handle = thread::spawn(move || {
            while let Ok(command) = command_rx.recv() {
                match command {
                    RunnerCommand::Step { iterations } => {
                        let before = algo.total_iterations();
                        if !algo.is_finished() {
                            algo.step(&problem, iterations);
                        }
                        let after = algo.total_iterations();
                        let update = RunnerUpdate {
                            solution: algo.current_solution().clone(),
                            iterations_done: after.saturating_sub(before),
                            total_iterations: after,
                            is_finished: algo.is_finished(),
                        };
                        if update_tx.send(update).is_err() {
                            break;
                        }
                        if algo.is_finished() {
                            break;
                        }
                    }
                    RunnerCommand::Stop => break,
                }
            }
        });

        Self {
            command_tx,
            update_rx,
            worker_handle: Some(worker_handle),
            awaiting_update: false,
            is_finished: false,
        }
    }

    pub fn request_step(&mut self, iterations: usize) {
        if self.is_finished || self.awaiting_update {
            return;
        }
        if self
            .command_tx
            .send(RunnerCommand::Step { iterations })
            .is_ok()
        {
            self.awaiting_update = true;
        } else {
            self.is_finished = true;
        }
    }

    pub fn poll_latest_update(&mut self) -> Option<RunnerUpdate> {
        let mut latest = None;
        while let Ok(update) = self.update_rx.try_recv() {
            self.awaiting_update = false;
            self.is_finished = update.is_finished;
            latest = Some(update);
        }
        latest
    }

    pub fn is_finished(&self) -> bool {
        self.is_finished
    }

    fn stop_worker(&mut self) {
        let _ = self.command_tx.send(RunnerCommand::Stop);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for AlgorithmRunner {
    fn drop(&mut self) {
        self.stop_worker();
    }
}
