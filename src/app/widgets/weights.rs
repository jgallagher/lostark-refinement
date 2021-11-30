use crate::app::solution::Scoring;
use eframe::egui::{self, epaint, Ui, Vec2};
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub(in crate::app) struct Preset {
    pub(in crate::app) name: &'static str,
    success: [f64; 3],
    fail: [f64; 3],
}

impl Preset {
    pub(in crate::app) fn equals(&self, scoring: &Scoring) -> bool {
        self.success == scoring.success && self.fail == scoring.fail
    }
}

pub(in crate::app) const PRESETS: [Preset; 2] = [
    Preset {
        name: "Balanced; slightly prefer skill 1",
        success: [1.1, 1.0, -1.0],
        fail: [0.0, 0.0, 0.0],
    },
    Preset {
        name: "Balanced; slightly prefer skill 2",
        success: [1.0, 1.1, -1.0],
        fail: [0.0, 0.0, 0.0],
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

impl Weights {
    pub(in crate::app) fn show(&mut self, ui: &mut Ui) -> Option<Scoring> {
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

        Some(Scoring {
            success: [success[0]?, success[1]?, success[2]?],
            fail: [fail[0]?, fail[1]?, fail[2]?],
        })
    }

    pub(in crate::app) fn assign_to_preset(&mut self, preset: &Preset) {
        for i in 0..3 {
            self.success[i] = format!("{:.1}", preset.success[i]);
            self.fail[i] = format!("{:.1}", preset.fail[i]);
        }
    }
}
