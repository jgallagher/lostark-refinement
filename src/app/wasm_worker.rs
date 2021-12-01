use super::{
    solution::{Answer, Scoring, Solution},
    widgets::GameState,
    SimResult,
};
use arrayvec::ArrayVec;
use eframe::epi::RepaintSignal;
use std::{cell::RefCell, sync::Arc};

pub(super) struct ThreadHandle {
    inner: RefCell<Inner>,
}

impl ThreadHandle {
    pub(super) fn spawn(
        scoring: Option<Scoring>,
        game_state: GameState,
        sim_tries: Option<u32>,
        _repaint_signal: Arc<dyn RepaintSignal>,
    ) -> Self {
        let mut inner = Inner {
            solution: None,
            most_likely: None,
            scoring,
            sim_tries,
            game_state,
        };
        inner.rebuild_solution();
        Self {
            inner: RefCell::new(inner),
        }
    }

    pub(super) fn status(&self) -> String {
        let inner = self.inner.borrow();
        match inner.solution.as_ref() {
            Some(solution) => {
                if inner.most_likely.is_some() {
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
        let mut inner = self.inner.borrow_mut();
        inner.scoring = Some(scoring);
        inner.rebuild_solution();
    }

    pub(super) fn update_sim_tries(&self, sim_tries: u32) {
        let mut inner = self.inner.borrow_mut();
        inner.sim_tries = Some(sim_tries);
        inner.rerun_simulation();
    }

    pub(super) fn update_game_state(&self, game_state: GameState) {
        let mut inner = self.inner.borrow_mut();
        let prev_num_slots = inner.game_state.num_slots();
        inner.game_state = game_state;
        if prev_num_slots != inner.game_state.num_slots() {
            inner.rebuild_solution();
        } else {
            inner.rerun_simulation();
        }
    }

    pub(super) fn sim_results(&self) -> Option<Vec<SimResult>> {
        self.inner.borrow().most_likely.clone()
    }

    pub(super) fn sorted_choices(&self, state: &GameState) -> Option<ArrayVec<Answer, 3>> {
        self.inner
            .borrow()
            .solution
            .as_ref()
            .and_then(|solution| solution.sorted_choices(state))
    }
}

struct Inner {
    solution: Option<Solution>,
    most_likely: Option<Vec<SimResult>>,
    scoring: Option<Scoring>,
    sim_tries: Option<u32>,
    game_state: GameState,
}

impl Inner {
    fn rebuild_solution(&mut self) {
        let scoring = match self.scoring {
            Some(scoring) => scoring,
            None => return,
        };

        self.solution = Some(Solution::build(scoring, self.game_state.num_slots()));
        self.rerun_simulation();
    }

    fn rerun_simulation(&mut self) {
        let sim_tries = match self.sim_tries {
            Some(n) => n,
            None => return,
        };

        let solution = match self.solution.as_ref() {
            Some(s) => s,
            None => return,
        };

        let most_likely = solution.simulate_top_10(sim_tries, &self.game_state);

        self.most_likely = Some(most_likely);
    }
}
