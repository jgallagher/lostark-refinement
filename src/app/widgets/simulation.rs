use crate::app::SimResult;
use eframe::egui::{self, Ui};

const PRESET_TRIES: [u32; 4] = [100, 1_000, 10_000, 100_000];

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub(in crate::app) struct Simulation {
    index: usize,
}

impl Default for Simulation {
    fn default() -> Self {
        Self {
            index: PRESET_TRIES.iter().position(|&x| x == 10_000).unwrap_or(0),
        }
    }
}

impl Simulation {
    pub(in crate::app) fn show(&mut self, ui: &mut Ui, most_likely: Option<Vec<SimResult>>) -> u32 {
        ui.vertical(|ui| {
            ui.heading("Most Likely Outcomes");
            ui.horizontal(|ui| {
                ui.label("Simulation runs");
                egui::ComboBox::from_id_source("simulation-runs").show_index(
                    ui,
                    &mut self.index,
                    PRESET_TRIES.len(),
                    |i| PRESET_TRIES[i].to_string(),
                );
            });

            if let Some(most_likely) = most_likely {
                ui.vertical(|ui| {
                    egui::Grid::new("sim-results-grid").show(ui, |ui| {
                        ui.label("Skill 1");
                        ui.label("Skill 2");
                        ui.label("Negative");
                        ui.label("Probability");
                        ui.label("Final Score");
                        ui.end_row();

                        for result in most_likely {
                            ui.label(format!("+{}", result.counts[0]));
                            ui.label(format!("+{}", result.counts[1]));
                            ui.label(format!("+{}", result.counts[2]));
                            ui.label(format!("{:.2}%", 100.0 * result.probability));
                            ui.label(format!("{:.3}", result.score));
                            ui.end_row();
                        }
                    });
                });
            }
        });

        PRESET_TRIES[self.index]
    }
}
