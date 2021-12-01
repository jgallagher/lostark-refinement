use eframe::{egui, epi};

mod chance;
mod solution;
mod widgets;
mod worker_thread;

use self::solution::Scoring;
use self::widgets::{GameState, Simulation, Weights};
use self::worker_thread::ThreadHandle;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    weights: Weights,
    selected_preset: usize,
    simulation: Simulation,
    sim_tries: Option<u32>,
    game_state: GameState,

    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    current_scoring: Option<Scoring>,

    #[cfg_attr(feature = "persistence", serde(skip))]
    worker_thread: Option<worker_thread::ThreadHandle>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            weights: Weights::default(),
            selected_preset: 0,
            simulation: Simulation::default(),
            sim_tries: None,
            game_state: GameState::default(),
            current_scoring: None,
            worker_thread: None,
        }
    }
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "eframe template"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        // spawn worker thread
        let worker_thread = ThreadHandle::spawn(
            self.weights.parse(),
            self.game_state.clone(),
            self.sim_tries,
            frame.repaint_signal(),
        );
        self.worker_thread = Some(worker_thread);
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let Self {
            weights,
            selected_preset,
            simulation,
            sim_tries,
            game_state,
            current_scoring,
            worker_thread,
        } = self;

        let worker_thread = worker_thread.as_ref().unwrap();

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.group(|ui| {
                        let scoring = weights.show(ui, selected_preset);
                        if let Some(scoring) = scoring {
                            // Update our & worker thread's scoring
                            if Some(scoring) != *current_scoring {
                                *current_scoring = Some(scoring);
                                worker_thread.update_weights(scoring);
                            }
                        }
                    });

                    ui.group(|ui| {
                        let tries = simulation.show(ui, worker_thread.sim_results());
                        if Some(tries) != *sim_tries {
                            *sim_tries = Some(tries);
                            worker_thread.update_sim_tries(tries);
                        }
                    });
                });

                ui.group(|ui| {
                    let prev_state = game_state.clone();
                    game_state.show(ui, worker_thread.optimal_choice(&prev_state));
                    if prev_state != *game_state {
                        worker_thread.update_game_state(game_state.clone());
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom-panel").show(ctx, |ui| {
            ui.label(worker_thread.status());
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}
