use crate::app::solution::Scoring;
use eframe::egui::{self, epaint, Ui, Vec2};

#[derive(Debug, PartialEq)]
pub(in crate::app) struct Preset {
    pub(in crate::app) name: &'static str,
    scoring: Scoring,
}

/*
impl Preset {
    pub(in crate::app) fn equals(&self, scoring: &Scoring) -> bool {
        self.scoring == scoring.success && self.fail == scoring.fail
    }
}
*/

pub(in crate::app) const PRESETS: [Preset; 2] = [
    Preset {
        name: "Balanced; slightly prefer skill 1",
        scoring: Scoring {
            success: [1.1, 1.0, -1.0],
            fail: [0.0, 0.0, 0.0],
        },
    },
    Preset {
        name: "Balanced; slightly prefer skill 2",
        scoring: Scoring {
            success: [1.0, 1.1, -1.0],
            fail: [0.0, 0.0, 0.0],
        },
    },
];

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub(in crate::app) struct Weights {
    success: [String; 3],
    fail: [String; 3],
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            success: ["1.0".to_string(), "1.5".to_string(), "-1.0".to_string()],
            fail: ["-1.0".to_string(), "-1.0".to_string(), "0.0".to_string()],
        }
    }
}

const VALID_FRAME: egui::Frame = egui::Frame {
    margin: Vec2::new(2.0, 2.0),
    corner_radius: 0.0,
    shadow: epaint::Shadow {
        extrusion: 0.0,
        color: egui::Color32::TRANSPARENT,
    },
    fill: egui::Color32::TRANSPARENT,
    stroke: egui::Stroke {
        width: 2.0,
        color: egui::Color32::TRANSPARENT,
    },
};

const INVALID_FRAME: egui::Frame = egui::Frame {
    margin: Vec2::new(2.0, 2.0),
    corner_radius: 0.0,
    shadow: epaint::Shadow {
        extrusion: 0.0,
        color: egui::Color32::TRANSPARENT,
    },
    fill: egui::Color32::TRANSPARENT,
    stroke: egui::Stroke {
        width: 2.0,
        color: egui::Color32::RED,
    },
};

fn show_textedit(ui: &mut Ui, s: &mut String) -> Option<f64> {
    match s.trim().parse::<f64>() {
        Ok(x) if x.is_finite() => {
            VALID_FRAME.show(ui, |ui| {
                ui.text_edit_singleline(s);
            });
            Some(x)
        }
        _ => {
            INVALID_FRAME.show(ui, |ui| {
                ui.text_edit_singleline(s);
            });
            None
        }
    }
}

fn parsed_fields_to_scoring(success: [Option<f64>; 3], fail: [Option<f64>; 3]) -> Option<Scoring> {
    Some(Scoring {
        success: [success[0]?, success[1]?, success[2]?],
        fail: [fail[0]?, fail[1]?, fail[2]?],
    })
}

impl Weights {
    pub(in crate::app) fn show(
        &mut self,
        ui: &mut Ui,
        selected_preset: &mut usize,
    ) -> Option<Scoring> {
        let mut success = [None; 3];
        let mut fail = [None; 3];
        ui.heading("Weights");
        egui::Grid::new("weights-grid").show(ui, |ui| {
            ui.label("");
            ui.label("Success");
            ui.label("Fail");
            ui.end_row();

            for (i, &s) in ["Buff 1", "Buff 2", "Debuff"].iter().enumerate() {
                ui.label(s);
                success[i] = show_textedit(ui, &mut self.success[i]);
                fail[i] = show_textedit(ui, &mut self.fail[i]);
                ui.end_row();
            }
        });

        let mut scoring = parsed_fields_to_scoring(success, fail);
        if let Some(scoring) = scoring.as_ref() {
            // Update presets combo box to match current weights
            let mut found_preset = false;
            for (i, preset) in PRESETS.iter().enumerate() {
                if preset.scoring == *scoring {
                    *selected_preset = i;
                    found_preset = true;
                    break;
                }
            }
            if !found_preset {
                *selected_preset = PRESETS.len();
            }
        }

        ui.horizontal(|ui| {
            ui.label("Presets");
            let resp = egui::ComboBox::from_id_source("presets-combo").show_index(
                ui,
                selected_preset,
                PRESETS.len() + 1,
                |i| {
                    PRESETS
                        .get(i)
                        .map(|p| p.name.to_string())
                        .unwrap_or_else(|| "Custom".to_string())
                },
            );
            if resp.changed() {
                println!("changed to {}", selected_preset);
                if let Some(preset) = PRESETS.get(*selected_preset) {
                    self.assign_to_preset(preset);
                    scoring = Some(preset.scoring);
                }
            }
        });

        scoring
    }

    pub(in crate::app) fn assign_to_preset(&mut self, preset: &Preset) {
        for i in 0..3 {
            self.success[i] = format!("{:.1}", preset.scoring.success[i]);
            self.fail[i] = format!("{:.1}", preset.scoring.fail[i]);
        }
    }
}
