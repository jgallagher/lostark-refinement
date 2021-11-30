use super::solution::{Scoring, Solution};
use crossbeam_channel::{Receiver, Sender};
use eframe::epi::RepaintSignal;
use parking_lot::Mutex;
use std::{sync::Arc, thread};

pub(super) struct ThreadHandle {
    state: Arc<Mutex<Option<Solution>>>,
    update_weights: Sender<Scoring>,
}

impl ThreadHandle {
    pub(super) fn spawn(repaint_signal: Arc<dyn RepaintSignal>) -> Self {
        let state = Arc::new(Mutex::new(None));
        let (update_weights, update_weights_rx) = crossbeam_channel::unbounded();

        let inner = Inner {
            state: Arc::clone(&state),
            update_weights: update_weights_rx,
            repaint_signal,
        };
        thread::spawn(move || inner.run());
        Self {
            state,
            update_weights,
        }
    }

    pub(super) fn status(&self) -> String {
        let state = self.state.lock();
        match state.as_ref() {
            Some(solution) => format!("Solved ({} states)", solution.num_states()),
            None => format!("Finding solution..."),
        }
    }

    pub(super) fn update_weights(&self, scoring: Scoring) {
        self.update_weights.send(scoring).unwrap();
    }
}

struct Inner {
    state: Arc<Mutex<Option<Solution>>>,
    update_weights: Receiver<Scoring>,
    repaint_signal: Arc<dyn RepaintSignal>,
}

impl Inner {
    fn run(self) -> Result<(), crossbeam_channel::TryRecvError> {
        loop {
            crossbeam_channel::select! {
                recv(self.update_weights) -> scoring => {
                    self.rebuild_solution(scoring?);
                }
            }
        }
    }

    fn rebuild_solution(&self, mut scoring: Scoring) {
        // drain any queued up changes, only keeping the latest
        while let Ok(s) = self.update_weights.try_recv() {
            scoring = s;
        }
        println!("background thread received new scoring {:?}", scoring);
        *self.state.lock() = None;

        let new_solution = Solution::build(&scoring, 24); // TODO count from UI

        *self.state.lock() = Some(new_solution);
        self.repaint_signal.request_repaint();
    }
}
