use crate::params::MyFilterParams;
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use std::f32::consts::PI;
use std::sync::Arc;

// --- Graphic Theme Color Constants ---
const COLOR_BG_TOP: egui::Color32 = egui::Color32::from_rgb(8, 8, 8);
const COLOR_BG_BOTTOM: egui::Color32 = egui::Color32::from_rgb(26, 26, 26);
const COLOR_TEXT_MAIN: egui::Color32 = egui::Color32::from_rgb(220, 220, 220);
const COLOR_TEXT_TITLE: egui::Color32 = egui::Color32::from_rgb(200, 200, 200);
const COLOR_TEXT_SUBTITLE: egui::Color32 = egui::Color32::from_rgb(140, 140, 140);
const COLOR_TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(100, 100, 100);
const COLOR_TEXT_PRECISION: egui::Color32 = egui::Color32::from_rgb(100, 200, 255);
const COLOR_DIVIDER: egui::Color32 = egui::Color32::from_rgb(60, 60, 60);

const COLOR_KNOB_BODY: egui::Color32 = egui::Color32::from_rgb(35, 35, 38);
const COLOR_KNOB_BG: egui::Color32 = egui::Color32::from_rgb(20, 20, 20);
const COLOR_KNOB_BORDER: egui::Color32 = egui::Color32::from_rgb(55, 55, 55);
const COLOR_KNOB_STROKE: egui::Color32 = egui::Color32::from_rgb(80, 80, 85);
const COLOR_KNOB_NEEDLE: egui::Color32 = egui::Color32::from_rgb(200, 200, 200);

// --- Layout & Graphic Control Constants ---
/// Total width of the central GUI vertical content panel in pixels.
const UI_PANEL_WIDTH: f32 = 330.0;

/// Radius of primary frequency cutoff control knobs in pixels.
const KNOB_RADIUS_LARGE: f32 = 35.0;

/// Radius of secondary control knobs (such as Q factor) in pixels.
const KNOB_RADIUS_SMALL: f32 = 28.0;

/// Base sensitivity scaling factor for vertical mouse drag interaction.
const DRAG_SENSITIVITY: f32 = 0.005;

/// Base sensitivity scaling factor for mouse scroll wheel interaction.
const SCROLL_SENSITIVITY: f32 = 0.001;

/// Sensitivity reduction divisor applied when holding the Shift key for fine precision adjustment.
const PRECISION_SCALE_DIVISOR: f32 = 10.0;

/// Arc starting angle for rotary knob needle rendering (7 o'clock position = 135 degrees).
const KNOB_START_ANGLE: f32 = PI * 0.75;

/// Arc ending angle for rotary knob needle rendering (5 o'clock position = 405 degrees).
const KNOB_END_ANGLE: f32 = PI * 2.25;

/// Encapsulates parameter range bounds (`min`, `max`) and logarithmic skew factor.
///
/// **Why:** Grouping boundary limits into a single struct prevents passing excess positional parameters
/// to UI functions and satisfies clean code function signature standards.
#[derive(Clone, Copy, Debug)]
pub struct ParamBounds {
    pub min: f32,
    pub max: f32,
    pub skew_factor: f32,
}

/// Transient state container for GUI text edit buffers and modal window visibility flags.
pub struct InternalState {
    pub lp_freq_str: String,
    pub hp_freq_str: String,
    pub q_str: String,
    pub show_about: bool,
}

/// Constructs the plugin GUI editor using `nih_plug_egui` integration.
///
/// # Arguments
/// * `params` - Shared atomic parameter structure.
/// * `editor_state` - Persistent GUI state object for window dimensions.
///
/// # Returns
/// An `Option<Box<dyn Editor>>` containing the egui UI frame ready for DAW embedding.
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
            render_background(egui_ctx);

            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(egui_ctx, |ui| {
                    ui.style_mut().visuals.override_text_color = Some(COLOR_TEXT_MAIN);

                    ui.horizontal(|ui| {
                        ui.add_space(12.0);

                        ui.vertical(|ui| {
                            ui.set_width(UI_PANEL_WIDTH);
                            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 6.0);

                            render_header(ui);
                            ui.add_space(8.0);

                            render_main_grid(ui, setter, &params, state);

                            ui.add_space(10.0);
                            render_divider(ui);
                            ui.add_space(8.0);

                            render_footer(ui, state);
                        });
                    });
                });

            if state.show_about {
                render_about_dialog(egui_ctx, state);
            }
        },
    )
}

/// Renders a soft vertical dark gradient across the full GUI canvas background.
///
/// # Arguments
/// * `egui_ctx` - Active egui context.
fn render_background(egui_ctx: &egui::Context) {
    let screen = egui_ctx.screen_rect();
    let painter = egui_ctx.layer_painter(egui::LayerId::background());

    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(screen.left_top(), COLOR_BG_TOP);
    mesh.colored_vertex(screen.right_top(), COLOR_BG_TOP);
    mesh.colored_vertex(screen.left_bottom(), COLOR_BG_BOTTOM);
    mesh.colored_vertex(screen.right_bottom(), COLOR_BG_BOTTOM);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 2, 3);

    painter.add(egui::Shape::mesh(mesh));
}

/// Renders the main plugin title and subtitle header section.
///
/// # Arguments
/// * `ui` - Mutable reference to the parent egui UI context.
fn render_header(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        ui.heading(
            egui::RichText::new("lh_filter V1")
                .size(22.0)
                .strong()
                .color(COLOR_TEXT_TITLE),
        );
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new("Low & High Pass Filter")
                .size(12.0)
                .color(COLOR_TEXT_SUBTITLE),
        );
        ui.add_space(5.0);
        render_divider(ui);
        ui.add_space(5.0);
    });
}

/// Draws a subtle horizontal separation line.
///
/// # Arguments
/// * `ui` - Mutable reference to the parent egui UI context.
fn render_divider(ui: &mut egui::Ui) {
    ui.painter().line_segment(
        [
            egui::pos2(ui.available_rect_before_wrap().left(), ui.cursor().top()),
            egui::pos2(ui.available_rect_before_wrap().right(), ui.cursor().top()),
        ],
        egui::Stroke::new(1.0, COLOR_DIVIDER),
    );
}

/// Renders the 2x2 grid containing the low-pass knob, high-pass knob, Q factor knob, and bypass checkbox.
///
/// # Arguments
/// * `ui` - Mutable reference to the egui UI frame.
/// * `setter` - Parameter setter interface to send changes to the DAW host.
/// * `params` - Reference to plugin parameter definitions.
/// * `state` - Internal GUI state holding text buffers.
fn render_main_grid(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    params: &MyFilterParams,
    state: &mut InternalState,
) {
    egui::Grid::new("main_grid")
        .spacing(egui::vec2(40.0, 20.0))
        .min_col_width(150.0)
        .show(ui, |ui| {
            knob_widget(
                ui,
                setter,
                &params.lp_freq,
                "Low Pass",
                KNOB_RADIUS_LARGE,
                &mut state.lp_freq_str,
            );
            knob_widget(
                ui,
                setter,
                &params.hp_freq,
                "High Pass",
                KNOB_RADIUS_LARGE,
                &mut state.hp_freq_str,
            );
            ui.end_row();

            knob_widget(
                ui,
                setter,
                &params.q,
                "Q Factor",
                KNOB_RADIUS_SMALL,
                &mut state.q_str,
            );

            render_bypass_checkbox(ui, setter, &params.bypass);
            ui.end_row();
        });
}

/// Renders the master bypass checkbox toggle control.
///
/// # Arguments
/// * `ui` - Mutable reference to the egui UI frame.
/// * `setter` - Parameter setter handle.
/// * `bypass_param` - Reference to the boolean bypass parameter.
fn render_bypass_checkbox(ui: &mut egui::Ui, setter: &ParamSetter, bypass_param: &BoolParam) {
    ui.vertical_centered(|ui| {
        ui.add_space(15.0);
        ui.label(egui::RichText::new("Bypass").size(14.0).strong());
        ui.add_space(6.0);
        let mut bypass = bypass_param.value();
        if ui.checkbox(&mut bypass, "Active").changed() {
            setter.begin_set_parameter(bypass_param);
            setter.set_parameter(bypass_param, bypass);
            setter.end_set_parameter(bypass_param);
        }
    });
}

/// Renders the bottom footer bar containing version tag and About dialog button.
///
/// # Arguments
/// * `ui` - Mutable reference to the egui UI frame.
/// * `state` - Internal state to toggle the About modal window.
fn render_footer(ui: &mut egui::Ui, state: &mut InternalState) {
    ui.horizontal(|ui| {
        ui.add_space(5.0);
        ui.label(
            egui::RichText::new("v0.1.0")
                .size(11.0)
                .color(COLOR_TEXT_MUTED),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(5.0);
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("ℹ About")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(160, 160, 160)),
                    )
                    .frame(false),
                )
                .clicked()
            {
                state.show_about = !state.show_about;
            }
        });
    });
}

/// Renders the modal overlay dialog displaying author contact and plugin description.
///
/// # Arguments
/// * `egui_ctx` - Active egui context.
/// * `state` - Internal state managing modal visibility.
fn render_about_dialog(egui_ctx: &egui::Context, state: &mut InternalState) {
    egui::Window::new("About")
        .collapsible(false)
        .resizable(false)
        .default_width(340.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(egui::Frame::window(&egui_ctx.style()).fill(egui::Color32::from_rgb(25, 25, 25)))
        .show(egui_ctx, |ui| {
            ui.style_mut().visuals.override_text_color = Some(COLOR_TEXT_TITLE);

            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("lh_filter V1").size(20.0).strong());
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(
                        "Dual-mode Low-Pass & High-Pass frequency filter\nwith resonance control and soft-limiting output stage.\nDesigned for precise frequency shaping in any mix chain.",
                    )
                    .size(12.0)
                    .color(egui::Color32::from_rgb(170, 170, 170)),
                );
                ui.add_space(12.0);

                ui.painter().line_segment(
                    [
                        egui::pos2(ui.available_rect_before_wrap().left() + 20.0, ui.cursor().top()),
                        egui::pos2(ui.available_rect_before_wrap().right() - 20.0, ui.cursor().top()),
                    ],
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 50)),
                );
                ui.add_space(12.0);

                ui.label(egui::RichText::new("Created by Oleg Chizhov").size(13.0).strong());
                ui.add_space(6.0);
                ui.label(egui::RichText::new("Contact: jaqueole@gmail.com").size(11.5).color(COLOR_TEXT_SUBTITLE));
                ui.add_space(3.0);
                ui.label(egui::RichText::new("PayPal donation: jaqueole@gmail.com").size(11.5).color(COLOR_TEXT_SUBTITLE));
                ui.add_space(12.0);

                if ui.button("Close").clicked() {
                    state.show_about = false;
                }
            });

            if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                state.show_about = false;
            }
        });
}

/// Converts a real parameter value into a normalized [0, 1] plain value, accounting for logarithmic skew factor.
///
/// # Arguments
/// * `real_val` - Parameter value in native units (e.g., Hz).
/// * `bounds` - Parameter boundary limits and skew factor.
///
/// # Returns
/// Normalized value in range `[0.0, 1.0]`.
pub fn real_to_plain(real_val: f32, bounds: ParamBounds) -> f32 {
    let normalized = ((real_val - bounds.min) / (bounds.max - bounds.min)).clamp(0.0, 1.0);
    if (bounds.skew_factor - 1.0).abs() < f32::EPSILON {
        normalized
    } else {
        normalized.powf(1.0 / bounds.skew_factor)
    }
}

/// Converts a normalized [0, 1] plain value into a real parameter value in native units, accounting for logarithmic skew factor.
///
/// # Arguments
/// * `plain_val` - Normalized plain slider value in range `[0.0, 1.0]`.
/// * `bounds` - Parameter boundary limits and skew factor.
///
/// # Returns
/// Real parameter value in native units (e.g., Hz).
pub fn plain_to_real(plain_val: f32, bounds: ParamBounds) -> f32 {
    let clamped_plain = plain_val.clamp(0.0, 1.0);
    if (bounds.skew_factor - 1.0).abs() < f32::EPSILON {
        bounds.min + clamped_plain * (bounds.max - bounds.min)
    } else {
        bounds.min + (bounds.max - bounds.min) * clamped_plain.powf(bounds.skew_factor)
    }
}

/// Extracts boundary limits (`min`, `max`, `skew_factor`) from a `FloatParam` into a `ParamBounds` struct.
///
/// # Arguments
/// * `param` - FloatParam instance to extract bounds from.
///
/// # Returns
/// A `ParamBounds` struct containing `min`, `max`, and `skew_factor`.
fn extract_range_bounds(param: &FloatParam) -> ParamBounds {
    match param.range() {
        FloatRange::Linear { min, max } => ParamBounds {
            min,
            max,
            skew_factor: 1.0,
        },
        FloatRange::Skewed { min, max, factor } => ParamBounds {
            min,
            max,
            skew_factor: factor,
        },
        _ => ParamBounds {
            min: 20.0,
            max: 20000.0,
            skew_factor: 1.0,
        },
    }
}

/// Custom interactive rotary knob UI control with support for mouse dragging, mouse wheel, double-click reset, and direct text entry.
///
/// # Arguments
/// * `ui` - Mutable egui UI context reference.
/// * `setter` - Parameter setter handle.
/// * `param` - Associated FloatParam reference.
/// * `label` - Text title label displayed above the knob.
/// * `radius` - Radius of the rotary knob graphics in pixels.
/// * `text_buf` - Mutable string buffer backing the text edit field.
fn knob_widget(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    param: &FloatParam,
    label: &str,
    radius: f32,
    text_buf: &mut String,
) {
    ui.vertical_centered(|ui| {
        let is_shift = ui.input(|i| i.modifiers.shift);

        let label_text = if is_shift {
            egui::RichText::new(label)
                .strong()
                .color(COLOR_TEXT_PRECISION)
        } else {
            egui::RichText::new(label).strong()
        };
        ui.label(label_text);
        ui.add_space(5.0);

        let size = radius * 2.2;
        let (rect, response) =
            ui.allocate_at_least(egui::vec2(size, size), egui::Sense::click_and_drag());
        let text_edit_id = ui.make_persistent_id(label);

        let bounds = extract_range_bounds(param);
        let current_real = param.value();
        let current_plain = real_to_plain(current_real, bounds);

        let (value_changed, new_real_value) = handle_knob_interaction(
            ui,
            &response,
            text_edit_id,
            param,
            current_plain,
            bounds,
            is_shift,
        );

        if value_changed {
            setter.begin_set_parameter(param);
            setter.set_parameter(param, new_real_value);
            setter.end_set_parameter(param);
            ui.ctx().request_repaint();
        }

        if ui.is_rect_visible(rect) {
            let updated_real = param.value();
            let updated_plain = real_to_plain(updated_real, bounds);
            draw_knob_visuals(ui, rect, &response, radius, updated_plain, is_shift);
        }

        ui.add_space(6.0);
        render_param_text_entry(ui, setter, param, text_edit_id, radius, text_buf, bounds);
    });
}

/// Evaluates mouse gestures (double-click, vertical drag, scroll wheel) for rotary knob control.
///
/// # Arguments
/// * `ui` - Mutable egui UI context reference.
/// * `response` - Response object for the knob's allocated rect.
/// * `text_edit_id` - Persistent ID of the associated text field.
/// * `param` - Associated FloatParam reference.
/// * `current_plain` - Current normalized plain value in range `[0.0, 1.0]`.
/// * `bounds` - Parameter boundaries and skew factor.
/// * `is_shift` - When true, enables precision fine-tuning mode (reducing drag/scroll sensitivity by 10x).
///
/// # Returns
/// A tuple `(value_changed: bool, new_real_value: f32)`.
fn handle_knob_interaction(
    ui: &mut egui::Ui,
    response: &egui::Response,
    text_edit_id: egui::Id,
    param: &FloatParam,
    current_plain: f32,
    bounds: ParamBounds,
    is_shift: bool,
) -> (bool, f32) {
    if response.double_clicked() {
        return (true, param.default_plain_value());
    }

    if response.dragged() {
        ui.memory_mut(|m| m.surrender_focus(text_edit_id));
        let delta = response.drag_delta().y;
        let mut sensitivity = DRAG_SENSITIVITY;
        if is_shift {
            sensitivity /= PRECISION_SCALE_DIVISOR;
        }

        let new_plain = (current_plain - delta * sensitivity).clamp(0.0, 1.0);
        let new_real = plain_to_real(new_plain, bounds);
        return (true, new_real);
    }

    if response.hovered() {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 {
            ui.memory_mut(|m| m.surrender_focus(text_edit_id));
            let mut sensitivity = SCROLL_SENSITIVITY;
            if is_shift {
                sensitivity /= PRECISION_SCALE_DIVISOR;
            }

            let new_plain = (current_plain + scroll * sensitivity).clamp(0.0, 1.0);
            let new_real = plain_to_real(new_plain, bounds);
            return (true, new_real);
        }
    }

    (false, param.value())
}

/// Paints the graphical elements of the rotary knob (background ring, body circle, precision stroke, and indicator needle).
///
/// # Arguments
/// * `ui` - Mutable egui UI context reference.
/// * `rect` - Allocated screen bounding box for the knob.
/// * `response` - egui Response object for hover state.
/// * `radius` - Knob radius in pixels.
/// * `normalized_plain` - Current knob position normalized in range `[0.0, 1.0]`.
/// * `is_shift` - Whether precision shift mode is active.
fn draw_knob_visuals(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    response: &egui::Response,
    radius: f32,
    normalized_plain: f32,
    is_shift: bool,
) {
    let painter = ui.painter();
    let center = rect.center();

    painter.circle(
        center,
        radius + 1.5,
        COLOR_KNOB_BG,
        egui::Stroke::new(1.0, COLOR_KNOB_BORDER),
    );
    painter.circle(center, radius, COLOR_KNOB_BODY, egui::Stroke::NONE);

    let stroke = if is_shift && response.hovered() {
        egui::Stroke::new(2.5, COLOR_TEXT_PRECISION)
    } else {
        egui::Stroke::new(2.0, COLOR_KNOB_STROKE)
    };
    painter.circle(center, radius, egui::Color32::TRANSPARENT, stroke);

    let current_angle = KNOB_START_ANGLE + normalized_plain * (KNOB_END_ANGLE - KNOB_START_ANGLE);
    let needle_end =
        center + egui::vec2(current_angle.cos(), current_angle.sin()) * (radius * 0.85);

    painter.line_segment(
        [center, needle_end],
        egui::Stroke::new(3.0, COLOR_KNOB_NEEDLE),
    );
    painter.circle_filled(center, radius * 0.12, COLOR_KNOB_NEEDLE);
}

/// Renders a single-line text input field below the rotary knob for direct numeric value editing.
///
/// # Arguments
/// * `ui` - Mutable egui UI context reference.
/// * `setter` - Parameter setter handle.
/// * `param` - Associated FloatParam reference.
/// * `text_edit_id` - Persistent ID of the text edit field.
/// * `radius` - Knob radius to scale text field width.
/// * `text_buf` - Mutable string buffer for text editing.
/// * `bounds` - Parameter boundary limits.
fn render_param_text_entry(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    param: &FloatParam,
    text_edit_id: egui::Id,
    radius: f32,
    text_buf: &mut String,
    bounds: ParamBounds,
) {
    let val = param.value();
    let has_focus = ui.memory(|m| m.has_focus(text_edit_id));

    if !has_focus {
        *text_buf = format!("{:.2}", val);
    }

    let text_edit = egui::TextEdit::singleline(text_buf)
        .id(text_edit_id)
        .horizontal_align(egui::Align::Center)
        .font(egui::FontId::proportional(14.0))
        .text_color(COLOR_TEXT_TITLE);

    let edit_response = ui.add_sized([radius * 2.8, 24.0], text_edit);

    if edit_response.lost_focus()
        || (edit_response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
    {
        if let Ok(parsed) = text_buf.parse::<f32>() {
            let clamped_real = parsed.clamp(bounds.min, bounds.max);

            setter.begin_set_parameter(param);
            setter.set_parameter(param, clamped_real);
            setter.end_set_parameter(param);
            ui.ctx().request_repaint();

            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                ui.memory_mut(|m| m.surrender_focus(text_edit_id));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_real_plain_transformations() {
        let bounds = ParamBounds {
            min: 0.0,
            max: 10.0,
            skew_factor: 1.0,
        };

        assert_eq!(real_to_plain(5.0, bounds), 0.5);
        assert_eq!(plain_to_real(0.5, bounds), 5.0);
    }

    #[test]
    fn test_skewed_real_plain_transformations() {
        let bounds = ParamBounds {
            min: 20.0,
            max: 20000.0,
            skew_factor: 0.25,
        };

        let plain = real_to_plain(bounds.min, bounds);
        assert_eq!(plain, 0.0);

        let real_restored = plain_to_real(plain, bounds);
        assert_eq!(real_restored, bounds.min);

        let plain_max = real_to_plain(bounds.max, bounds);
        assert_eq!(plain_max, 1.0);
        let real_max_restored = plain_to_real(plain_max, bounds);
        assert_eq!(real_max_restored, bounds.max);
    }
}
