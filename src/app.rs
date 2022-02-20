use eframe::{egui, epi};

mod chance;
mod solution;
mod widgets;

#[cfg(not(target_arch = "wasm32"))]
mod worker_thread;

#[cfg(target_arch = "wasm32")]
#[path = "app/wasm_worker.rs"]
mod worker_thread;

use self::solution::Scoring;
use self::widgets::{GameState, Simulation, Weights};
use self::worker_thread::ThreadHandle;

#[derive(Debug, Clone, Copy)]
struct SimResult {
    counts: [u8; 3],
    probability: f64,
    score: f64,
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(PartialEq)]
enum LightOrDarkMode {
    Light,
    Dark,
}

impl Default for LightOrDarkMode {
    fn default() -> Self {
        Self::Light
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
#[derive(Default)]
pub struct TemplateApp {
    weights: Weights,
    selected_preset: usize,
    simulation: Simulation,
    sim_tries: Option<u32>,
    game_state: GameState,
    light_or_dark: LightOrDarkMode,

    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    current_scoring: Option<Scoring>,

    #[cfg_attr(feature = "persistence", serde(skip))]
    worker_thread: Option<worker_thread::ThreadHandle>,
}

fn set_light_mode(ctx: &egui::CtxRef) {
    let mut visuals = egui::Visuals::light();
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
    ctx.set_visuals(visuals);
}

fn set_dark_mode(ctx: &egui::CtxRef) {
    let visuals = egui::Visuals::dark();
    ctx.set_visuals(visuals);
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "Lost Ark Ability Stone Refinement Optimizer"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        let mut fonts = egui::FontDefinitions::default();
        fonts.family_and_size.insert(
            egui::TextStyle::Body,
            (egui::FontFamily::Proportional, 16.0),
        );
        ctx.set_fonts(fonts);

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
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let Self {
            weights,
            selected_preset,
            simulation,
            sim_tries,
            game_state,
            light_or_dark,
            current_scoring,
            worker_thread,
        } = self;

        match light_or_dark {
            LightOrDarkMode::Light => set_light_mode(ctx),
            LightOrDarkMode::Dark => set_dark_mode(ctx),
        }

        let worker_thread = worker_thread.as_ref().unwrap();

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                #[cfg(not(target_arch = "wasm32"))]
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.quit();
                    }
                });

                ui.selectable_value(light_or_dark, LightOrDarkMode::Light, "Light Mode");
                ui.selectable_value(light_or_dark, LightOrDarkMode::Dark, "Dark Mode");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.vertical(|ui| {
                ui.group(|ui| {
                    let prev_state = game_state.clone();
                    game_state.show(ui, worker_thread.sorted_choices(&prev_state));
                    if prev_state != *game_state {
                        worker_thread.update_game_state(game_state.clone());
                    }
                });

                //ui.horizontal(|ui| {
                //ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.with_layout(
                    egui::Layout::left_to_right()
                        .with_main_justify(true)
                        .with_cross_align(egui::Align::Min),
                    |ui| {
                        egui::Grid::new("weights-help").show(ui, |ui| {
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
                            ui.end_row();

                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.heading("Help / About");
                                    ui.label("\u{2022} Set point values for success/failure for each row in `Weights` above");
                                    ui.label("\u{2022} Reported scores are the expected values based on current progress and chosen weights");
                                    ui.label("\u{2022} Follow the suggested selections (green highlit skill)");
                                    ui.label("\u{2022} Update the top section with the in-game result (+1 or failure)");
                                    ui.label("\u{2022} The right section shows the 10 most probable final outcomes");
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 0.0;
                                        ui.label("\u{2022} Problems/suggestions/question? Open an ");
                                        ui.hyperlink_to("issue", "https://github.com/jgallagher/lostark-refinement/issues");
                                    });
                                });
                            });
                            ui.end_row();
                        });

                        ui.group(|ui| {
                            let tries = simulation.show(ui, worker_thread.sim_results());
                            if Some(tries) != *sim_tries {
                                *sim_tries = Some(tries);
                                worker_thread.update_sim_tries(tries);
                            }
                        });
                    },
                );
            });
        });

        egui::TopBottomPanel::bottom("bottom-panel").show(ctx, |ui| {
            ui.label(worker_thread.status());

            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    //ui.spacing_mut().item_spacing.x = 0.0;
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                    ui.label("and");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label("powered by");
                })
            });
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
