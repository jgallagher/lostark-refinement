use eframe::{egui, epi};

mod chance;
mod solution;
mod widgets;
mod worker_thread;

use self::solution::Scoring;
use self::widgets::{Weights, PRESET_WEIGHTS};
use self::worker_thread::ThreadHandle;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    weights: Weights,
    selected_preset: usize,

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

        // sanity check selected preset
        if self.selected_preset > PRESET_WEIGHTS.len() + 1 {
            self.selected_preset = 0;
        }

        // spawn worker thread
        let worker_thread = ThreadHandle::spawn(frame.repaint_signal());
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

        /*
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                });
            });
        });
        */

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            let scoring = weights.show(ui);
            if let Some(scoring) = scoring {
                // Update presets combo box to match current weights
                let mut found_preset = false;
                for (i, preset) in PRESET_WEIGHTS.iter().enumerate() {
                    if preset.equals(&scoring) {
                        *selected_preset = i;
                        found_preset = true;
                        break;
                    }
                }
                if !found_preset {
                    *selected_preset = PRESET_WEIGHTS.len();
                }

                // Update our & worker thread's scoring
                if Some(scoring) != *current_scoring {
                    *current_scoring = Some(scoring);
                    worker_thread.update_weights(scoring);
                }
            }

            ui.horizontal(|ui| {
                ui.label("Presets");
                let resp = egui::ComboBox::from_id_source("presets-combo").show_index(
                    ui,
                    selected_preset,
                    PRESET_WEIGHTS.len() + 1,
                    |i| {
                        PRESET_WEIGHTS
                            .get(i)
                            .map(|p| p.name.to_string())
                            .unwrap_or_else(|| "Custom".to_string())
                    },
                );
                if resp.changed() {
                    println!("changed to {}", selected_preset);
                    if let Some(preset) = PRESET_WEIGHTS.get(*selected_preset) {
                        weights.assign_to_preset(preset);
                    }
                }
            });

            /*
            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
            if ui
                .add_enabled(
                    worker_thread_state.is_some(),
                    egui::Button::new("Update state!"),
                )
                .clicked()
            {
                println!("requesting state update");
                worker_thread.request_update_state();
            }
            */
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
