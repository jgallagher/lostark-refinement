use super::solution::{Scoring, Solution};
use crossbeam_channel::{Receiver, Sender};
use eframe::epi::RepaintSignal;
use fnv::FnvHashMap;
use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use std::{sync::Arc, thread};

#[derive(Debug, Clone, Copy)]
pub(super) struct SimResult {
    pub(super) counts: [u8; 3],
    pub(super) probability: f64,
    pub(super) score: f64,
}

pub(super) struct ThreadHandle {
    state: Arc<RwLock<State>>,
    update_weights: Sender<Scoring>,
    update_sim_tries: Sender<u32>,
}

impl ThreadHandle {
    pub(super) fn spawn(repaint_signal: Arc<dyn RepaintSignal>) -> Self {
        let state = Arc::default();
        let (update_weights, update_weights_rx) = crossbeam_channel::unbounded();
        let (update_sim_tries, update_sim_tries_rx) = crossbeam_channel::unbounded();

        let inner = Inner {
            state: Arc::clone(&state),
            update_weights: update_weights_rx,
            update_sim_tries: update_sim_tries_rx,
            sim_tries: None,
            repaint_signal,
        };
        thread::spawn(move || inner.run());
        Self {
            state,
            update_weights,
            update_sim_tries,
        }
    }

    pub(super) fn status(&self) -> String {
        let state = self.state.read();
        match state.solution.as_ref() {
            Some(solution) => {
                if state.most_likely.is_some() {
                    format!("Solved ({} states)", solution.num_states())
                } else {
                    format!(
                        "Solved ({} states); running simulations...",
                        solution.num_states()
                    )
                }
            }
            None => "Finding solution...".to_string(),
        }
    }

    pub(super) fn update_weights(&self, scoring: Scoring) {
        self.update_weights.send(scoring).unwrap();
    }

    pub(super) fn update_sim_tries(&self, sim_tries: u32) {
        self.update_sim_tries.send(sim_tries).unwrap();
    }

    pub(super) fn sim_results(&self) -> Option<Vec<SimResult>> {
        self.state.read().most_likely.clone()
    }
}

#[derive(Debug, Default)]
struct State {
    solution: Option<Solution>,
    most_likely: Option<Vec<SimResult>>,
}

impl State {
    fn reset_solution(&mut self) {
        self.solution = None;
        self.most_likely = None;
    }

    fn reset_simulation(&mut self) {
        self.most_likely = None;
    }
}

struct Inner {
    state: Arc<RwLock<State>>,
    update_weights: Receiver<Scoring>,
    update_sim_tries: Receiver<u32>,
    sim_tries: Option<u32>,
    repaint_signal: Arc<dyn RepaintSignal>,
}

impl Inner {
    fn run(mut self) -> Result<(), crossbeam_channel::TryRecvError> {
        loop {
            crossbeam_channel::select! {
                recv(self.update_weights) -> scoring => {
                    self.rebuild_solution(scoring?);
                }
                recv(self.update_sim_tries) -> sim_tries => {
                    self.sim_tries = Some(sim_tries?);
                    println!("background thread received sim_tries {}", sim_tries.unwrap());
                    self.state.write().reset_simulation();
                    self.repaint_signal.request_repaint();
                    self.rerun_simulation();
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
        self.state.write().reset_solution();

        let new_solution = Solution::build(scoring, 24); // TODO count from UI

        self.state.write().solution = Some(new_solution);

        self.repaint_signal.request_repaint();
        self.rerun_simulation();
    }

    fn rerun_simulation(&self) {
        const SIM_RESULTS_TO_DISPLAY: usize = 10;

        let sim_tries = match self.sim_tries {
            Some(n) => n,
            None => return,
        };

        let state = self.state.upgradable_read();
        let solution = match state.solution.as_ref() {
            Some(s) => s,
            None => return,
        };

        let mut counts: FnvHashMap<[u8; 3], u32> = FnvHashMap::default();
        let mut rng = rand::thread_rng();
        for _ in 0..sim_tries {
            *counts.entry(solution.simulate_once(&mut rng)).or_default() += 1;
        }
        let mut counts = counts.into_iter().collect::<Vec<_>>();
        counts.sort_unstable_by_key(|(_result, count)| *count);

        let mut most_likely = Vec::with_capacity(SIM_RESULTS_TO_DISPLAY);
        for (result, count) in counts.into_iter().rev().take(SIM_RESULTS_TO_DISPLAY) {
            let score = solution.eval_result(result);
            most_likely.push(SimResult {
                counts: result,
                probability: f64::from(count) / f64::from(sim_tries),
                score,
            });
        }

        {
            let mut state = RwLockUpgradableReadGuard::upgrade(state);
            state.most_likely = Some(most_likely);
        }
        self.repaint_signal.request_repaint();
    }
}
