use arrayvec::ArrayVec;
use eframe::egui::{self, Ui};

use crate::app::chance::Chance;

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

impl GameState {
    pub(in crate::app) fn show(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label("Current State");
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
            });

            let num_slots = usize::from(self.num_slots);
            for row in &mut self.rows {
                row.truncate(num_slots);
            }

            egui::Grid::new("main-state-grid").show(ui, |ui| {
                for (&label, row) in ROW_LABELS.iter().zip(&mut self.rows) {
                    show_slots_row(ui, label, num_slots, row, &mut self.chance)
                }
            });
        });
    }
}

fn show_slots_row(
    ui: &mut Ui,
    label: &'static str,
    num_slots: usize,
    row: &mut Row,
    chance: &mut Chance,
) {
    ui.label(label);
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
