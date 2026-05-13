use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use std::sync::Arc;
use crate::params::MyFilterParams;

pub struct InternalState {
    pub lp_freq_str: String,
    pub hp_freq_str: String,
    pub q_str: String,
}

pub fn create_editor(
    params: Arc<MyFilterParams>,
    editor_state: Arc<EguiState>,
) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        editor_state,
        InternalState {
            lp_freq_str: String::new(),
            hp_freq_str: String::new(),
            q_str: String::new(),
        },
        |_, _| {},
        move |egui_ctx, setter, state| {
            egui::CentralPanel::default().show(egui_ctx, |ui| {
                // Add outer margin/padding
                ui.horizontal(|ui| {
                    ui.add_space(15.0); // 15px left margin
                    
                    ui.vertical(|ui| {
                        ui.set_width(450.0); // Allow 20px on right too
                        
                        ui.style_mut().spacing.item_spacing = egui::vec2(10.0, 10.0);
                        
                        ui.vertical_centered(|ui| {
                            ui.add_space(15.0);
                            ui.heading(egui::RichText::new("LHFilter").size(26.0).strong());
                            ui.add_space(5.0);
                            ui.separator();
                        });

                        ui.add_space(20.0);

                        // 2x2 Grid Layout
                        egui::Grid::new("main_grid")
                            .spacing(egui::vec2(60.0, 40.0))
                            .min_col_width(180.0)
                            .show(ui, |ui| {
                                // LP Knob
                                knob_widget(ui, setter, &params.lp_freq, "Low Pass", 50.0, &mut state.lp_freq_str);
                                
                                // HP Knob
                                knob_widget(ui, setter, &params.hp_freq, "High Pass", 50.0, &mut state.hp_freq_str);
                                ui.end_row();

                                // Q Factor (under LP)
                                knob_widget(ui, setter, &params.q, "Q Factor", 40.0, &mut state.q_str);
                                
                                // Bypass (under HP)
                                ui.vertical_centered(|ui| {
                                    ui.add_space(25.0);
                                    ui.label(egui::RichText::new("Bypass").size(16.0).strong());
                                    ui.add_space(10.0);
                                    let mut bypass = params.bypass.value();
                                    if ui.checkbox(&mut bypass, "Active").changed() {
                                        setter.begin_set_parameter(&params.bypass);
                                        setter.set_parameter(&params.bypass, bypass);
                                        setter.end_set_parameter(&params.bypass);
                                    }
                                });
                                ui.end_row();
                            });
                    });
                });
            });
        },
    )
}

fn knob_widget(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    param: &FloatParam,
    label: &str,
    radius: f32,
    text_buf: &mut String,
) {
    ui.vertical_centered(|ui| {
        let is_ctrl = ui.input(|i| i.modifiers.command);
        
        // Visual feedback for precision mode
        if is_ctrl {
            ui.label(egui::RichText::new(label).strong().color(egui::Color32::from_rgb(100, 200, 255)));
        } else {
            ui.label(egui::RichText::new(label).strong());
        }
        ui.add_space(5.0);

        let size = radius * 2.2;
        let (rect, response) = ui.allocate_at_least(egui::vec2(size, size), egui::Sense::click_and_drag());
        
        let text_edit_id = ui.make_persistent_id(label);

        // Derive plain value (0.0 to 1.0) strictly from the current REAL value
        let current_real = param.value();
        let current_plain = match param.range() {
            FloatRange::Linear { min, max } => (current_real - min) / (max - min),
            FloatRange::Skewed { min, max, .. } => {
                let skew = 1.0 / 4.35;
                ((current_real - min) / (max - min)).clamp(0.0, 1.0).powf(skew)
            }
            _ => (current_real - 20.0) / 19980.0,
        }.clamp(0.0, 1.0);

        let mut value_changed = false;
        let mut new_real_value = current_real;

        // Handle interactions
        if response.double_clicked() {
            // Bulletproof defaults
            if label.contains("Q") {
                new_real_value = 10.0;
            } else if label.contains("Low") {
                new_real_value = 20000.0;
            } else {
                new_real_value = 20.0;
            }
            value_changed = true;
        }
        else if response.dragged() {
            ui.memory_mut(|m| m.surrender_focus(text_edit_id));
            
            let delta = response.drag_delta().y;
            let mut sensitivity = 0.005;
            if is_ctrl {
                sensitivity /= 10.0;
            }
            
            let new_plain = (current_plain - delta * sensitivity).clamp(0.0, 1.0);
            
            new_real_value = match param.range() {
                FloatRange::Linear { min, max } => min + new_plain * (max - min),
                FloatRange::Skewed { min, max, .. } => {
                    let skew = 4.35;
                    min + (max - min) * new_plain.powf(skew)
                }
                _ => 20.0 + new_plain * 19980.0,
            };
            value_changed = true;
        }

        // Mouse Wheel support
        if response.hovered() {
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 {
                ui.memory_mut(|m| m.surrender_focus(text_edit_id));
                
                let mut sensitivity = 0.001;
                if is_ctrl {
                    sensitivity /= 10.0;
                }
                
                let new_plain = (current_plain + scroll * sensitivity).clamp(0.0, 1.0);
                
                new_real_value = match param.range() {
                    FloatRange::Linear { min, max } => min + new_plain * (max - min),
                    FloatRange::Skewed { min, max, .. } => {
                        let skew = 4.35;
                        min + (max - min) * new_plain.powf(skew)
                    }
                    _ => 20.0 + new_plain * 19980.0,
                };
                value_changed = true;
            }
        }

        // Update parameter
        if value_changed {
            setter.begin_set_parameter(param);
            setter.set_parameter(param, new_real_value);
            setter.end_set_parameter(param);
            ui.ctx().request_repaint();
        }

        // Draw the Knob
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = rect.center();
            let visuals = ui.style().interact(&response);

            // Precision mode outline
            let stroke = if is_ctrl && response.hovered() {
                egui::Stroke::new(2.5, egui::Color32::from_rgb(100, 200, 255))
            } else {
                egui::Stroke::new(2.5, visuals.fg_stroke.color)
            };

            painter.circle(
                center,
                radius,
                visuals.bg_fill,
                stroke,
            );

            let start_angle = std::f32::consts::PI * 0.75;
            let end_angle = std::f32::consts::PI * 2.25;
            
            // Re-calculate angle based on the very latest real value
            let updated_real = param.value();
            let updated_plain = match param.range() {
                FloatRange::Linear { min, max } => (updated_real - min) / (max - min),
                FloatRange::Skewed { min, max, .. } => {
                    let skew = 1.0 / 4.35;
                    ((updated_real - min) / (max - min)).clamp(0.0, 1.0).powf(skew)
                }
                _ => (updated_real - 20.0) / 19980.0,
            }.clamp(0.0, 1.0);
            
            let current_angle = start_angle + updated_plain * (end_angle - start_angle);

            let needle_end = center + egui::vec2(current_angle.cos(), current_angle.sin()) * (radius * 0.85);
            painter.line_segment(
                [center, needle_end],
                egui::Stroke::new(4.0, visuals.fg_stroke.color),
            );
            
            painter.circle_filled(center, radius * 0.15, visuals.fg_stroke.color);
        }

        ui.add_space(10.0);

        // Text Field
        let val = param.value();
        let has_focus = ui.memory(|m| m.has_focus(text_edit_id));
        
        if !has_focus {
            *text_buf = format!("{:.2}", val);
        }
        
        let text_edit = egui::TextEdit::singleline(text_buf)
            .id(text_edit_id)
            .horizontal_align(egui::Align::Center)
            .font(egui::FontId::proportional(14.0));
            
        let edit_response = ui.add_sized([radius * 2.8, 24.0], text_edit);

        if edit_response.lost_focus() || (edit_response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
            if let Ok(parsed) = text_buf.parse::<f32>() {
                // Host expects REAL value, so we just clamp the input manually to be safe
                let clamped_real = match param.range() {
                    FloatRange::Linear { min, max } => parsed.clamp(min, max),
                    FloatRange::Skewed { min, max, .. } => parsed.clamp(min, max),
                    _ => parsed,
                };

                setter.begin_set_parameter(param);
                setter.set_parameter(param, clamped_real);
                setter.end_set_parameter(param);
                ui.ctx().request_repaint();
                
                // Return focus to knob if Enter was pressed
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    ui.memory_mut(|m| m.surrender_focus(text_edit_id));
                }
            }
        }
    });
}
