//! Phase 35 — Alarm Rules management UI panel.
//!
//! Toggle with Ctrl+R. Shows existing rules in a table and a form to add new ones.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use uuid::Uuid;

use crate::alarm::rules::{AlarmRule, AlarmRuleSet, RuleCondition};
use crate::ui::LayoutMode;

// ── State ─────────────────────────────────────────────────────────────────────

/// UI state for the alarm rules panel.
#[derive(Resource, Default)]
pub struct AlarmRulesState {
    pub visible:      bool,
    pub new_name:     String,
    pub new_key:      String,
    pub new_op:       OpChoice,
    pub new_value:    String,
    pub new_severity: String,
    pub new_message:  String,
}

/// Combo-box choices for rule condition operator.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum OpChoice {
    #[default]
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    OutsideRange,
}

impl OpChoice {
    pub fn label(&self) -> &'static str {
        match self {
            Self::GreaterThan    => "> (greater than)",
            Self::LessThan       => "< (less than)",
            Self::GreaterOrEqual => ">= (greater or equal)",
            Self::LessOrEqual    => "<= (less or equal)",
            Self::OutsideRange   => "outside center ± margin",
        }
    }

    /// Convert form values into a `RuleCondition`. Returns `None` if the
    /// threshold string cannot be parsed as `f64`.
    pub fn to_condition(&self, value_str: &str) -> Option<RuleCondition> {
        let v: f64 = value_str.trim().parse().ok()?;
        Some(match self {
            Self::GreaterThan    => RuleCondition::GreaterThan    { value: v },
            Self::LessThan       => RuleCondition::LessThan       { value: v },
            Self::GreaterOrEqual => RuleCondition::GreaterOrEqual { value: v },
            Self::LessOrEqual    => RuleCondition::LessOrEqual    { value: v },
            // For OutsideRange the form value is the half-width margin; center = 0.
            Self::OutsideRange   => RuleCondition::OutsideRange   { center: 0.0, margin: v },
        })
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Toggle the alarm rules panel with Ctrl+R.
pub fn toggle_alarm_rules(
    keyboard:  Res<ButtonInput<KeyCode>>,
    mut state: ResMut<AlarmRulesState>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft)
             || keyboard.pressed(KeyCode::ControlRight);
    if ctrl && keyboard.just_pressed(KeyCode::KeyR) {
        state.visible = !state.visible;
    }
}

/// Render the alarm rules management window.
pub fn render_alarm_rules(
    mut contexts: EguiContexts,
    layout_mode:  Res<LayoutMode>,
    mut state:    ResMut<AlarmRulesState>,
    mut rule_set: ResMut<AlarmRuleSet>,
) {
    if !state.visible { return; }
    if *layout_mode == LayoutMode::FullscreenScene { return; }

    // Staged mutations
    let mut to_remove: Option<Uuid>   = None;
    let mut to_toggle: Option<Uuid>   = None;
    let mut add_rule:  Option<AlarmRule> = None;
    let mut do_save                   = false;

    let ctx = contexts.ctx_mut().expect("egui context");

    egui::Window::new("⚡ Alarm Rules")
        .default_width(520.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Client-side rules evaluate every incoming telemetry reading.");
            ui.label("Shortcut: Ctrl+R to open/close.");
            ui.separator();

            // ── Existing rules table ────────────────────────────────────────
            if rule_set.rules.is_empty() {
                ui.weak("No rules defined yet. Add one below ↓");
            } else {
                egui::Grid::new("rules_table")
                    .striped(true)
                    .min_col_width(70.0)
                    .show(ui, |ui| {
                        ui.strong("Name");
                        ui.strong("Key");
                        ui.strong("Condition");
                        ui.strong("Severity");
                        ui.strong("On");
                        ui.strong("Del");
                        ui.end_row();

                        for rule in &rule_set.rules {
                            let dim = if rule.enabled {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::DARK_GRAY
                            };
                            ui.colored_label(dim, &rule.name);
                            ui.colored_label(dim, &rule.key);
                            ui.colored_label(dim, rule.condition.description());
                            let sev_color = severity_color(&rule.severity);
                            ui.colored_label(sev_color, &rule.severity);

                            let toggle_icon = if rule.enabled { "✅" } else { "⬜" };
                            if ui.small_button(toggle_icon).clicked() {
                                to_toggle = Some(rule.id);
                            }
                            if ui.small_button("🗑").clicked() {
                                to_remove = Some(rule.id);
                            }
                            ui.end_row();
                        }
                    });
            }

            ui.separator();
            ui.strong("Add New Rule");
            ui.add_space(4.0);

            egui::Grid::new("new_rule_form")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut state.new_name);
                    ui.end_row();

                    ui.label("Telemetry key:");
                    ui.text_edit_singleline(&mut state.new_key);
                    ui.end_row();

                    ui.label("Operator:");
                    egui::ComboBox::from_id_salt("rule_op")
                        .selected_text(state.new_op.label())
                        .show_ui(ui, |ui| {
                            for op in [
                                OpChoice::GreaterThan,
                                OpChoice::LessThan,
                                OpChoice::GreaterOrEqual,
                                OpChoice::LessOrEqual,
                                OpChoice::OutsideRange,
                            ] {
                                let lbl = op.label();
                                ui.selectable_value(&mut state.new_op, op, lbl);
                            }
                        });
                    ui.end_row();

                    ui.label("Threshold:");
                    ui.text_edit_singleline(&mut state.new_value);
                    ui.end_row();

                    ui.label("Severity:");
                    let sev_display = if state.new_severity.is_empty() {
                        "MAJOR"
                    } else {
                        &state.new_severity
                    };
                    egui::ComboBox::from_id_salt("rule_sev")
                        .selected_text(sev_display)
                        .show_ui(ui, |ui| {
                            for sev in ["WARNING", "MINOR", "MAJOR", "CRITICAL"] {
                                ui.selectable_value(
                                    &mut state.new_severity,
                                    sev.to_string(),
                                    sev,
                                );
                            }
                        });
                    ui.end_row();

                    ui.label("Message (opt):");
                    ui.text_edit_singleline(&mut state.new_message);
                    ui.end_row();
                });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let can_add = !state.new_name.is_empty()
                    && !state.new_key.is_empty()
                    && state.new_op.to_condition(&state.new_value).is_some();

                if ui
                    .add_enabled(can_add, egui::Button::new("➕ Add Rule"))
                    .clicked()
                {
                    if let Some(cond) = state.new_op.to_condition(&state.new_value) {
                        add_rule = Some(AlarmRule {
                            id:        Uuid::new_v4(),
                            name:      std::mem::take(&mut state.new_name),
                            device_id: None,
                            key:       std::mem::take(&mut state.new_key),
                            condition: cond,
                            severity:  if state.new_severity.is_empty() {
                                "MAJOR".into()
                            } else {
                                state.new_severity.clone()
                            },
                            message:   std::mem::take(&mut state.new_message),
                            enabled:   true,
                        });
                        state.new_value.clear();
                    }
                }

                if ui.button("💾 Save to disk").clicked() {
                    do_save = true;
                }

                if ui.button("✕ Close").clicked() {
                    state.visible = false;
                }
            });
        });

    // Apply staged mutations
    if let Some(id) = to_remove {
        rule_set.remove(id);
    }
    if let Some(id) = to_toggle {
        rule_set.toggle(id);
    }
    if let Some(rule) = add_rule {
        rule_set.add(rule);
    }
    if do_save {
        rule_set.save();
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alarm::rules::RuleCondition;

    #[test]
    fn op_choice_greater_than_parses_float() {
        let cond = OpChoice::GreaterThan.to_condition("80.5").expect("valid float");
        assert_eq!(cond, RuleCondition::GreaterThan { value: 80.5 });
    }

    #[test]
    fn op_choice_less_than_parses_integer_string() {
        let cond = OpChoice::LessThan.to_condition("5").expect("valid integer string");
        assert_eq!(cond, RuleCondition::LessThan { value: 5.0 });
    }

    #[test]
    fn op_choice_invalid_string_returns_none() {
        assert!(OpChoice::GreaterThan.to_condition("not_a_number").is_none());
        assert!(OpChoice::LessThan.to_condition("").is_none());
        assert!(OpChoice::LessOrEqual.to_condition("abc").is_none());
    }

    #[test]
    fn op_choice_greater_or_equal_parses() {
        let cond = OpChoice::GreaterOrEqual.to_condition("100.0").expect("valid");
        assert_eq!(cond, RuleCondition::GreaterOrEqual { value: 100.0 });
    }

    #[test]
    fn op_choice_less_or_equal_parses() {
        let cond = OpChoice::LessOrEqual.to_condition("-5.5").expect("negative float");
        assert_eq!(cond, RuleCondition::LessOrEqual { value: -5.5 });
    }

    #[test]
    fn op_choice_outside_range_uses_center_zero_with_margin() {
        let cond = OpChoice::OutsideRange.to_condition("10.0").expect("valid");
        // center = 0, margin = value
        assert_eq!(cond, RuleCondition::OutsideRange { center: 0.0, margin: 10.0 });
    }

    #[test]
    fn op_choice_all_variants_have_labels() {
        for op in [
            OpChoice::GreaterThan, OpChoice::LessThan,
            OpChoice::GreaterOrEqual, OpChoice::LessOrEqual,
            OpChoice::OutsideRange,
        ] {
            assert!(!op.label().is_empty(), "every operator should have a non-empty label");
        }
    }

    #[test]
    fn alarm_rules_state_default_not_visible() {
        let state = AlarmRulesState::default();
        assert!(!state.visible);
        assert!(state.new_name.is_empty());
        assert!(state.new_key.is_empty());
        assert!(state.new_value.is_empty());
    }
}

fn severity_color(sev: &str) -> egui::Color32 {
    match sev {
        "WARNING"  => egui::Color32::YELLOW,
        "MINOR"    => egui::Color32::from_rgb(255, 153, 0),
        "MAJOR"    => egui::Color32::RED,
        "CRITICAL" => egui::Color32::from_rgb(204, 0, 204),
        _          => egui::Color32::LIGHT_GRAY,
    }
}
