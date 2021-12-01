use arrayvec::ArrayVec;
use eframe::egui::{self, epaint, Ui, Vec2};

use crate::app::{chance::Chance, solution::Answer};

type Row = ArrayVec<bool, { ALL_NUM_SLOTS[ALL_NUM_SLOTS.len() - 1].0 as usize }>;

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) struct GameState {
    chance: Chance,
    num_slots: u8,
    rows: [Row; 3],
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            chance: Chance::SeventyFive,
            num_slots: 8,
            rows: [Row::new(), Row::new(), Row::new()],
        }
    }
}

impl GameState {
    pub(in crate::app) fn chance(&self) -> Chance {
        self.chance
    }

    pub(in crate::app) fn num_slots(&self) -> u8 {
        self.num_slots
    }

    pub(in crate::app) fn row(&self, i: usize) -> &[bool] {
        &self.rows[i]
    }
}

const ALL_CHANCES: [Chance; 6] = [
    Chance::SeventyFive,
    Chance::SixtyFive,
    Chance::FiftyFive,
    Chance::FourtyFive,
    Chance::ThirtyFive,
    Chance::TwentyFive,
];

const ALL_NUM_SLOTS: [(u8, &str); 15] = [
    (2, "2"),
    (3, "3"),
    (4, "4"),
    (5, "5"),
    (6, "6"),
    (7, "7"),
    (8, "8"),
    (9, "9"),
    (10, "10"),
    (11, "11"),
    (12, "12"),
    (13, "13"),
    (14, "14"),
    (15, "15"),
    (16, "16"),
];

const ROW_LABELS: [&str; 3] = ["Skill 1", "Skill 2", "Negative"];

const TRANSPARENT_FRAME: egui::Frame = egui::Frame {
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

const HIGHLIGHT_FRAME: egui::Frame = egui::Frame {
    margin: Vec2::new(2.0, 2.0),
    corner_radius: 0.0,
    shadow: epaint::Shadow {
        extrusion: 0.0,
        color: egui::Color32::TRANSPARENT,
    },
    fill: egui::Color32::TRANSPARENT,
    stroke: egui::Stroke {
        width: 2.0,
        color: egui::Color32::GREEN,
    },
};

impl GameState {
    pub(in crate::app) fn show(&mut self, ui: &mut Ui, choices: Option<ArrayVec<Answer, 3>>) {
        ui.vertical(|ui| {
            ui.heading("Lost Ark Ability Stone Refinement Optimizer");
            ui.horizontal(|ui| {
                ui.label("Success Chance:");
                egui::ComboBox::from_id_source("success-chance-combo")
                    .selected_text(self.chance.as_str())
                    .show_ui(ui, |ui| {
                        for c in ALL_CHANCES {
                            let text = c.as_str();
                            ui.selectable_value(&mut self.chance, c, text);
                        }
                    });

                ui.label("Total Slots:");
                egui::ComboBox::from_id_source("total-slots-combo")
                    .selected_text(format!("{}", self.num_slots))
                    .show_ui(ui, |ui| {
                        for (n, text) in ALL_NUM_SLOTS {
                            ui.selectable_value(&mut self.num_slots, n, text);
                        }
                    });

                if ui.button("RESET").clicked() {
                    self.chance = Chance::SeventyFive;
                    for r in &mut self.rows {
                        r.clear();
                    }
                }
            });

            let num_slots = usize::from(self.num_slots);
            for row in &mut self.rows {
                row.truncate(num_slots);
            }

            egui::Grid::new("main-state-grid")
                .min_row_height(45.0)
                .show(ui, |ui| {
                    for (i, (&label, row)) in ROW_LABELS.iter().zip(&mut self.rows).enumerate() {
                        show_slots_row(ui, label, num_slots, row, &mut self.chance, i, &choices)
                    }
                });

            if let Some(mut choices) = choices {
                ui.separator();
                ui.label("Average Final Score for Each Choice");
                let best = choices[0].index;
                choices.sort_unstable_by_key(|a| a.index);

                egui::Grid::new("each-choice-final-score-grid").show(ui, |ui| {
                    for choice in choices {
                        ui.label(ROW_LABELS[choice.index]);
                        ui.label(format!("{:.3}", choice.score));
                        if choice.index == best {
                            ui.label("*** BEST ***");
                        }
                        ui.end_row();
                    }
                });
            }
        });
    }
}

fn show_slots_row(
    ui: &mut Ui,
    label: &'static str,
    num_slots: usize,
    row: &mut Row,
    chance: &mut Chance,
    row_index: usize,
    optimal: &Option<ArrayVec<Answer, 3>>,
) {
    let label_frame = if optimal.as_ref().map(|a| a[0].index) == Some(row_index) {
        &HIGHLIGHT_FRAME
    } else {
        &TRANSPARENT_FRAME
    };
    label_frame.show(ui, |ui| {
        ui.label(label);
    });

    for i in 0..num_slots {
        if let Some(&succeeded) = row.get(i) {
            ui.label(if succeeded { "+1" } else { "fail" });
        } else if i == row.len() {
            ui.vertical(|ui| {
                let mut selected = -1;
                if ui.radio_value(&mut selected, 1, "+1").clicked() {
                    row.push(true);
                    chance.down();
                }
                if ui.radio_value(&mut selected, 0, "fail").clicked() {
                    row.push(false);
                    chance.up();
                }
            });
        } else {
            ui.label("--");
        }
    }
    if ui
        .add_enabled(!row.is_empty(), egui::Button::new("X"))
        .clicked()
    {
        if let Some(prev_success) = row.pop() {
            if prev_success {
                chance.up();
            } else {
                chance.down();
            }
        }
    }
    ui.end_row();
}
