use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use std::sync::Arc;
use crate::params::MyFilterParams;

pub struct InternalState {
    pub lp_freq_str: String,
    pub hp_freq_str: String,
    pub q_str: String,
    pub show_about: bool,
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
            show_about: false,
        },
        |_, _| {},
        move |egui_ctx, setter, state| {
            // -------------------------------------------------------
            // Background: Black-to-grey soft vertical gradient
            // -------------------------------------------------------
            let screen = egui_ctx.screen_rect();
            let painter = egui_ctx.layer_painter(egui::LayerId::background());
            let top_color = egui::Color32::from_rgb(8, 8, 8);         // near-black
            let bottom_color = egui::Color32::from_rgb(26, 26, 26);    // very dark grey — subtle
            let mesh = {
                let mut mesh = egui::Mesh::default();
                let tl = screen.left_top();
                let tr = screen.right_top();
                let bl = screen.left_bottom();
                let br = screen.right_bottom();
                mesh.colored_vertex(tl, top_color);
                mesh.colored_vertex(tr, top_color);
                mesh.colored_vertex(bl, bottom_color);
                mesh.colored_vertex(br, bottom_color);
                mesh.add_triangle(0, 1, 2);
                mesh.add_triangle(1, 2, 3);
                mesh
            };
            painter.add(egui::Shape::mesh(mesh));

            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(egui_ctx, |ui| {
                    // Style overrides for dark theme text visibility
                    ui.style_mut().visuals.override_text_color = Some(egui::Color32::from_rgb(220, 220, 220));

                    // Add outer margin/padding
                    ui.horizontal(|ui| {
                        ui.add_space(12.0); // 12px left margin
                        
                        ui.vertical(|ui| {
                            ui.set_width(330.0);
                            
                            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 6.0);
                            
                            ui.vertical_centered(|ui| {
                                ui.add_space(10.0);
                                ui.heading(egui::RichText::new("lf_filter V1").size(22.0).strong().color(egui::Color32::from_rgb(200, 200, 200)));
                                ui.add_space(2.0);
                                ui.label(egui::RichText::new("Low & High Pass Filter").size(12.0).color(egui::Color32::from_rgb(140, 140, 140)));
                                ui.add_space(5.0);
                                ui.painter().line_segment(
                                    [
                                        egui::pos2(ui.available_rect_before_wrap().left(), ui.cursor().top()),
                                        egui::pos2(ui.available_rect_before_wrap().right(), ui.cursor().top()),
                                    ],
                                    egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)),
                                );
                                ui.add_space(5.0);
                            });

                            ui.add_space(8.0);

                            // 2x2 Grid Layout
                            egui::Grid::new("main_grid")
                                .spacing(egui::vec2(40.0, 20.0))
                                .min_col_width(150.0)
                                .show(ui, |ui| {
                                    // LP Knob
                                    knob_widget(ui, setter, &params.lp_freq, "Low Pass", 35.0, &mut state.lp_freq_str);
                                    
                                    // HP Knob
                                    knob_widget(ui, setter, &params.hp_freq, "High Pass", 35.0, &mut state.hp_freq_str);
                                    ui.end_row();

                                    // Q Factor (under LP)
                                    knob_widget(ui, setter, &params.q, "Q Factor", 28.0, &mut state.q_str);
                                    
                                    // Bypass (under HP)
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(15.0);
                                        ui.label(egui::RichText::new("Bypass").size(14.0).strong());
                                        ui.add_space(6.0);
                                        let mut bypass = params.bypass.value();
                                        if ui.checkbox(&mut bypass, "Active").changed() {
                                            setter.begin_set_parameter(&params.bypass);
                                            setter.set_parameter(&params.bypass, bypass);
                                            setter.end_set_parameter(&params.bypass);
                                        }
                                    });
                                    ui.end_row();
                                });

                            // Footer with About button
                            ui.add_space(10.0);
                            ui.painter().line_segment(
                                [
                                    egui::pos2(ui.available_rect_before_wrap().left(), ui.cursor().top()),
                                    egui::pos2(ui.available_rect_before_wrap().right(), ui.cursor().top()),
                                ],
                                egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)),
                            );
                            ui.add_space(8.0);

                            ui.horizontal(|ui| {
                                ui.add_space(5.0);
                                ui.label(egui::RichText::new("v0.1.0").size(11.0).color(egui::Color32::from_rgb(100, 100, 100)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add_space(5.0);
                                    if ui.add(egui::Button::new(
                                        egui::RichText::new("ℹ About").size(11.0).color(egui::Color32::from_rgb(160, 160, 160))
                                    ).frame(false)).clicked() {
                                        state.show_about = !state.show_about;
                                    }
                                });
                            });
                        });
                    });
                });

            // About window
            if state.show_about {
                egui::Window::new("About")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(340.0)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .frame(egui::Frame::window(&egui_ctx.style()).fill(egui::Color32::from_rgb(25, 25, 25)))
                    .show(egui_ctx, |ui| {
                        ui.style_mut().visuals.override_text_color = Some(egui::Color32::from_rgb(200, 200, 200));

                        ui.vertical_centered(|ui| {
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("lf_filter V1").size(20.0).strong());
                            ui.add_space(6.0);
                            ui.label(egui::RichText::new("Dual-mode Low-Pass & High-Pass frequency filter\nwith resonance control and soft-limiting output stage.\nDesigned for precise frequency shaping in any mix chain.")
                                .size(12.0).color(egui::Color32::from_rgb(170, 170, 170)));
                            ui.add_space(12.0);

                            ui.painter().line_segment(
                                [
                                    egui::pos2(ui.available_rect_before_wrap().left() + 20.0, ui.cursor().top()),
                                    egui::pos2(ui.available_rect_before_wrap().right() - 20.0, ui.cursor().top()),
                                ],
                                egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 50)),
                            );
                            ui.add_space(12.0);

                            // GDPR-compliant: labels only, no hyperlinks
                            ui.label(egui::RichText::new("Created by Oleg Chizhov aka Чеширьsky")
                                .size(13.0).strong());
                            ui.add_space(6.0);
                            ui.label(egui::RichText::new("Contact: jaqueole@gmail.com")
                                .size(11.5).color(egui::Color32::from_rgb(150, 150, 150)));
                            ui.add_space(3.0);
                            ui.label(egui::RichText::new("PayPal donation: jaqueole@gmail.com")
                                .size(11.5).color(egui::Color32::from_rgb(150, 150, 150)));
                            ui.add_space(12.0);
                        });

                        // Close on button or Escape
                        ui.vertical_centered(|ui| {
                            if ui.button("Close").clicked() {
                                state.show_about = false;
                            }
                        });
                        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            state.show_about = false;
                        }
                    });
            }
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
        // Use Shift for precision — Ctrl/Cmd conflicts with DAW shortcuts
        let is_shift = ui.input(|i| i.modifiers.shift);
        
        // Visual feedback for precision mode
        if is_shift {
            ui.label(egui::RichText::new(label).strong().color(egui::Color32::from_rgb(100, 200, 255)));
        } else {
            ui.label(egui::RichText::new(label).strong());
        }
        ui.add_space(5.0);

        let size = radius * 2.2;
        let (rect, response) = ui.allocate_at_least(egui::vec2(size, size), egui::Sense::click_and_drag());
        
        let text_edit_id = ui.make_persistent_id(label);

        // Extract the actual skew factor from the param range instead of hardcoding
        let (range_min, range_max, skew_factor) = match param.range() {
            FloatRange::Linear { min, max } => (min, max, 1.0_f32),
            FloatRange::Skewed { min, max, factor } => (min, max, factor),
            _ => (20.0, 20000.0, 1.0),
        };

        // Derive plain value (0.0 to 1.0) strictly from the current REAL value
        let current_real = param.value();
        let current_plain = if skew_factor != 1.0 {
            ((current_real - range_min) / (range_max - range_min)).clamp(0.0, 1.0).powf(1.0 / skew_factor)
        } else {
            ((current_real - range_min) / (range_max - range_min)).clamp(0.0, 1.0)
        };

        let mut value_changed = false;
        let mut new_real_value = current_real;

        // Handle interactions
        if response.double_clicked() {
            // Use param.default_plain_value() — returns the REAL default value
            new_real_value = param.default_plain_value();
            value_changed = true;
        }
        else if response.dragged() {
            ui.memory_mut(|m| m.surrender_focus(text_edit_id));
            
            let delta = response.drag_delta().y;
            let mut sensitivity = 0.005;
            if is_shift {
                sensitivity /= 10.0;
            }
            
            let new_plain = (current_plain - delta * sensitivity).clamp(0.0, 1.0);
            
            new_real_value = if skew_factor != 1.0 {
                range_min + (range_max - range_min) * new_plain.powf(skew_factor)
            } else {
                range_min + new_plain * (range_max - range_min)
            };
            value_changed = true;
        }

        // Mouse Wheel support
        if response.hovered() {
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 {
                ui.memory_mut(|m| m.surrender_focus(text_edit_id));
                
                let mut sensitivity = 0.001;
                if is_shift {
                    sensitivity /= 10.0;
                }
                
                let new_plain = (current_plain + scroll * sensitivity).clamp(0.0, 1.0);
                
                new_real_value = if skew_factor != 1.0 {
                    range_min + (range_max - range_min) * new_plain.powf(skew_factor)
                } else {
                    range_min + new_plain * (range_max - range_min)
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
            let _visuals = ui.style().interact(&response);

            // Background ring (subtle dark circle)
            painter.circle(
                center,
                radius + 1.5,
                egui::Color32::from_rgb(20, 20, 20),
                egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 55, 55)),
            );

            // Main knob body
            painter.circle(
                center,
                radius,
                egui::Color32::from_rgb(35, 35, 38),
                egui::Stroke::NONE,
            );

            // Precision mode highlight ring
            let stroke = if is_shift && response.hovered() {
                egui::Stroke::new(2.5, egui::Color32::from_rgb(100, 200, 255))
            } else {
                egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 80, 85))
            };

            painter.circle(
                center,
                radius,
                egui::Color32::TRANSPARENT,
                stroke,
            );

            let start_angle = std::f32::consts::PI * 0.75;
            let end_angle = std::f32::consts::PI * 2.25;
            
            // Re-calculate angle based on the very latest real value
            let updated_real = param.value();
            let updated_plain = if skew_factor != 1.0 {
                ((updated_real - range_min) / (range_max - range_min)).clamp(0.0, 1.0).powf(1.0 / skew_factor)
            } else {
                ((updated_real - range_min) / (range_max - range_min)).clamp(0.0, 1.0)
            };
            
            let current_angle = start_angle + updated_plain * (end_angle - start_angle);

            let needle_end = center + egui::vec2(current_angle.cos(), current_angle.sin()) * (radius * 0.85);
            painter.line_segment(
                [center, needle_end],
                egui::Stroke::new(3.0, egui::Color32::from_rgb(200, 200, 200)),
            );
            
            painter.circle_filled(center, radius * 0.12, egui::Color32::from_rgb(160, 160, 160));
        }

        ui.add_space(6.0);

        // Text Field
        let val = param.value();
        let has_focus = ui.memory(|m| m.has_focus(text_edit_id));
        
        if !has_focus {
            *text_buf = format!("{:.2}", val);
        }
        
        let text_edit = egui::TextEdit::singleline(text_buf)
            .id(text_edit_id)
            .horizontal_align(egui::Align::Center)
            .font(egui::FontId::proportional(14.0))
            .text_color(egui::Color32::from_rgb(200, 200, 200));
            
        let edit_response = ui.add_sized([radius * 2.8, 24.0], text_edit);

        if edit_response.lost_focus() || (edit_response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
            if let Ok(parsed) = text_buf.parse::<f32>() {
                // Host expects REAL value, so we just clamp the input manually to be safe
                let clamped_real = parsed.clamp(range_min, range_max);

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
