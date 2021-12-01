use crate::app::solution::Scoring;
use eframe::egui::{self, epaint, Ui, Vec2};

#[derive(Debug, PartialEq)]
struct Preset {
    name: &'static str,
    scoring: Scoring,
}

const PRESETS: [Preset; 6] = [
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
    Preset {
        name: "Maximize skill 1",
        scoring: Scoring {
            success: [100.0, 1.0, -1.0],
            fail: [0.0, 0.0, 0.0],
        },
    },
    Preset {
        name: "Maximize skill 2",
        scoring: Scoring {
            success: [1.0, 100.0, -1.0],
            fail: [0.0, 0.0, 0.0],
        },
    },
    Preset {
        name: "Minimize negative; slightly prefer skill 1",
        scoring: Scoring {
            success: [1.1, 1.0, -100.0],
            fail: [0.0, 0.0, 100.0],
        },
    },
    Preset {
        name: "Minimize negative; slightly prefer skill 2",
        scoring: Scoring {
            success: [1.0, 1.1, -100.0],
            fail: [0.0, 0.0, 100.0],
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
        let mut this = Self {
            success: Default::default(),
            fail: Default::default(),
        };
        this.assign_to_preset(&PRESETS[0]);
        this
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
        let mut scoring = None;
        ui.vertical(|ui| {
            let mut success = [None; 3];
            let mut fail = [None; 3];
            ui.heading("Weights");
            egui::Grid::new("weights-grid")
                .min_col_width(72.0)
                .show(ui, |ui| {
                    ui.label("");
                    ui.label("Success");
                    ui.label("Fail");
                    ui.end_row();

                    for (i, &s) in ["Skill 1", "Skill 2", "Negative"].iter().enumerate() {
                        ui.label(s);
                        success[i] = show_textedit(ui, &mut self.success[i]);
                        fail[i] = show_textedit(ui, &mut self.fail[i]);
                        ui.end_row();
                    }
                });

            scoring = parsed_fields_to_scoring(success, fail);
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
                let resp = egui::ComboBox::from_id_source("presets-combo")
                    .width(300.0)
                    .show_index(ui, selected_preset, PRESETS.len() + 1, |i| {
                        PRESETS
                            .get(i)
                            .map(|p| p.name.to_string())
                            .unwrap_or_else(|| "Custom".to_string())
                    });
                if resp.changed() {
                    println!("changed to {}", selected_preset);
                    if let Some(preset) = PRESETS.get(*selected_preset) {
                        self.assign_to_preset(preset);
                        scoring = Some(preset.scoring);
                    }
                }
            });
        });

        scoring
    }

    fn assign_to_preset(&mut self, preset: &Preset) {
        for i in 0..3 {
            self.success[i] = format!("{:.1}", preset.scoring.success[i]);
            self.fail[i] = format!("{:.1}", preset.scoring.fail[i]);
        }
    }

    pub(in crate::app) fn parse(&self) -> Option<Scoring> {
        parsed_fields_to_scoring(
            [
                self.success[0].parse().ok(),
                self.success[1].parse().ok(),
                self.success[2].parse().ok(),
            ],
            [
                self.fail[0].parse().ok(),
                self.fail[1].parse().ok(),
                self.fail[2].parse().ok(),
            ],
        )
    }
}
