use crossbeam_channel::{Receiver, Sender};
use eframe::epi::RepaintSignal;
use parking_lot::Mutex;
use std::{sync::Arc, thread};

pub(super) struct ThreadHandle {
    state: Arc<Mutex<Option<u64>>>,
    command_queue: Sender<Command>,
}

impl ThreadHandle {
    pub(super) fn spawn(repaint_signal: Arc<dyn RepaintSignal>) -> Self {
        let state = Arc::new(Mutex::new(None));
        let (tx, rx) = crossbeam_channel::unbounded();
        {
            let state = Arc::clone(&state);
            thread::spawn(move || run(state, rx, repaint_signal));
        }
        Self {
            state,
            command_queue: tx,
        }
    }

    pub(super) fn request_update_state(&self) {
        self.command_queue.send(Command::UpdateState).unwrap();
    }

    pub(super) fn current_state(&self) -> Option<u64> {
        *self.state.lock()
    }
}

enum Command {
    UpdateState,
}

fn run(
    state: Arc<Mutex<Option<u64>>>,
    commands: Receiver<Command>,
    repaint_signal: Arc<dyn RepaintSignal>,
) {
    let mut i = 0;
    for command in commands {
        match command {
            Command::UpdateState => {
                *state.lock() = None;

                thread::sleep(std::time::Duration::from_secs(5));

                *state.lock() = Some(i);
                repaint_signal.request_repaint();
                i += 1;
            }
        }
    }
}
