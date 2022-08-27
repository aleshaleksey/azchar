use crate::styles;
use crate::AZCharFourth;

use libazdice::distribution::{Bonus, Dice, DiceBag, DiceGroup, RollResults};
use std::cmp::Ordering;

pub(crate) enum RollKind {
    Advantage,
    Disadvantage,
    Normal,
}

impl RollKind {
    pub(crate) fn d100(&self) -> Vec<DiceGroup> {
        match self {
            Self::Normal => vec![Dice::with_size_and_count(100, 1).into()],
            Self::Advantage => {
                let mut d = Dice::with_size_and_count(100, 2);
                d.with_drop_lowest(1).expect("This is ok.");
                vec![d.into()]
            }
            Self::Disadvantage => {
                let mut d = Dice::with_size_and_count(100, 2);
                d.with_drop_highest(1).expect("This is ok.");
                vec![d.into()]
            }
        }
    }

    pub(crate) fn d20(&self, bonus: i64) -> Vec<DiceGroup> {
        let mut dice = match self {
            Self::Normal => vec![Dice::with_size_and_count(20, 1).into()],
            Self::Advantage => {
                let mut d = Dice::with_size_and_count(20, 2);
                d.with_drop_lowest(1).expect("This is ok.");
                vec![d.into()]
            }
            Self::Disadvantage => {
                let mut d = Dice::with_size_and_count(20, 2);
                d.with_drop_highest(1).expect("This is ok.");
                vec![d.into()]
            }
        };
        match bonus.cmp(&0) {
            Ordering::Greater => dice.push(Bonus::plus(bonus as u32).into()),
            Ordering::Less => dice.push(Bonus::minus(bonus.abs() as u32).into()),
            _ => {}
        };
        dice
    }
}

impl AZCharFourth {
    pub(super) fn set_dice_dialog(&mut self, ctx: &egui::Context) {
        let mut hide = false;
        let pos = ctx.input().pointer.hover_pos();
        let pos = match pos {
            Some(pos) => pos,
            None => egui::pos2(100.0, 100.0),
        };
        if let Some(dice) = self.dice_dialog.as_ref() {
            egui::Area::new("roll-details")
                .default_pos(pos)
                .show(ctx, |ui| {
                    ui.set_style(styles::style());
                    self.frame.show(ui, |ui| {
                        ui.vertical(|ui| {
                            let _ = ui.selectable_label(false, dice.to_string()).clicked();
                            ui.horizontal(|ui| {
                                if ui.button("Ok").clicked() {
                                    hide = true;
                                }
                            })
                        })
                    });
                });
        }
        if hide {
            self.dice_dialog = None;
        }
    }
}

pub(super) fn fill(dice: Vec<DiceGroup>, dice_container: &mut Option<RollResults>) {
    let diceroll = DiceBag::from_dice(dice).roll();
    *dice_container = Some(diceroll);
}
