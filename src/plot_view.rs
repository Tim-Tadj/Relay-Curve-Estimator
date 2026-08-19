use crate::estimator::{CurveFitResult, TestPoint};
use crate::theme::Theme;
use eframe::egui::{self, Button, Color32, CornerRadius, RichText, Stroke, Vec2b};
use egui_plot::{CoordinatesFormatter, Corner, Legend, Line, MarkerShape, Plot, PlotPoints, Points, VLine};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlotScaleMode {
    LogLog,
    Linear,
}

pub struct PlotConfig {
    pub scale_mode: PlotScaleMode,
    pub show_residuals: bool,
    pub show_pickup_line: bool,
    pub show_runner_up: bool,
    pub show_point_labels: bool,
}

impl Default for PlotConfig {
    fn default() -> Self {
        Self {
            scale_mode: PlotScaleMode::LogLog,
            show_residuals: true,
            show_pickup_line: true,
            show_runner_up: true,
            show_point_labels: true,
        }
    }
}

pub struct TccPlotView;

impl TccPlotView {
    pub fn show(
        ui: &mut egui::Ui,
        config: &mut PlotConfig,
        pickup_current: f64,
        points: &[TestPoint],
        best_match: Option<&CurveFitResult>,
        runner_up: Option<&CurveFitResult>,
    ) {
        ui.vertical(|ui| {
            // Plot Control Toolbar
            ui.horizontal(|ui| {
                ui.label(RichText::new("Time-Current Characteristic (TCC)").strong().size(14.0).color(Theme::TEXT_WHITE));
                ui.add_space(16.0);

                // High-contrast Scale Selector Buttons
                let is_log = config.scale_mode == PlotScaleMode::LogLog;
                let log_bg = if is_log { Theme::ACCENT_PRIMARY } else { Theme::BG_CARD };
                let log_text = if is_log { Theme::TEXT_WHITE } else { Theme::TEXT_SECONDARY };
                let log_stroke = if is_log { Stroke::new(1.0_f32, Theme::ACCENT_CYAN) } else { Stroke::new(1.0_f32, Theme::BORDER_CARD) };

                if ui.add(
                    Button::new(RichText::new("Log-Log (Standard)").strong().size(12.0).color(log_text))
                        .fill(log_bg)
                        .stroke(log_stroke)
                        .corner_radius(CornerRadius::same(4))
                ).clicked() {
                    config.scale_mode = PlotScaleMode::LogLog;
                }

                let is_lin = config.scale_mode == PlotScaleMode::Linear;
                let lin_bg = if is_lin { Theme::ACCENT_PRIMARY } else { Theme::BG_CARD };
                let lin_text = if is_lin { Theme::TEXT_WHITE } else { Theme::TEXT_SECONDARY };
                let lin_stroke = if is_lin { Stroke::new(1.0_f32, Theme::ACCENT_CYAN) } else { Stroke::new(1.0_f32, Theme::BORDER_CARD) };

                if ui.add(
                    Button::new(RichText::new("Linear").strong().size(12.0).color(lin_text))
                        .fill(lin_bg)
                        .stroke(lin_stroke)
                        .corner_radius(CornerRadius::same(4))
                ).clicked() {
                    config.scale_mode = PlotScaleMode::Linear;
                }

                ui.separator();

                // Layer toggles
                ui.checkbox(&mut config.show_residuals, RichText::new("Residuals").color(Theme::TEXT_WHITE));
                ui.checkbox(&mut config.show_runner_up, RichText::new("Compare 2nd Match").color(Theme::TEXT_WHITE));
                ui.checkbox(&mut config.show_pickup_line, RichText::new("Pickup Line").color(Theme::TEXT_WHITE));
            });

            ui.add_space(6.0);

            // Active valid test points
            let active_points: Vec<&TestPoint> = points.iter().filter(|p| p.active && p.current > pickup_current && p.time > 0.0).collect();

            let max_test_current = active_points
                .iter()
                .map(|p| p.current)
                .fold(pickup_current * 10.0, f64::max);

            let min_current = pickup_current * 1.02;
            let max_current = (max_test_current * 1.5).max(pickup_current * 12.0);

            let is_log = config.scale_mode == PlotScaleMode::LogLog;

            // Enclose in dedicated dark scope frame
            Theme::plot_frame().show(ui, |ui| {
                let plot = Plot::new("tcc_engineering_scope")
                    .show_background(false)
                    .legend(
                        Legend::default()
                            .position(Corner::RightTop)
                            .background_alpha(0.9)
                    )
                    .show_grid(true)
                    .show_x(true)
                    .show_y(true)
                    .auto_bounds(Vec2b::new(true, true))
                    .label_formatter(move |name, point| {
                        let (curr, time) = if is_log {
                            (10.0f64.powf(point.x), 10.0f64.powf(point.y))
                        } else {
                            (point.x, point.y)
                        };

                        let time_str = if time >= 60.0 {
                            format!("{:.2} s ({:.1} min)", time, time / 60.0)
                        } else if time >= 1.0 {
                            format!("{:.3} s", time)
                        } else {
                            format!("{:.1} ms", time * 1000.0)
                        };

                        if name.is_empty() {
                            format!("Current: {:.2} A\nTime: {}", curr, time_str)
                        } else {
                            format!("{}\nCurrent: {:.2} A\nTime: {}", name, curr, time_str)
                        }
                    })
                    .coordinates_formatter(
                        Corner::LeftBottom,
                        CoordinatesFormatter::new(move |point, _| {
                            let (curr, time) = if is_log {
                                (10.0f64.powf(point.x), 10.0f64.powf(point.y))
                            } else {
                                (point.x, point.y)
                            };

                            let time_str = if time >= 60.0 {
                                format!("{:.2} s ({:.1} min)", time, time / 60.0)
                            } else if time >= 1.0 {
                                format!("{:.3} s", time)
                            } else {
                                format!("{:.1} ms", time * 1000.0)
                            };

                            format!("Cursor -> Current: {:.2} A | Time: {}", curr, time_str)
                        }),
                    )
                    .x_axis_formatter(move |mark, _range| {
                        if is_log {
                            let actual_val = 10.0f64.powf(mark.value);
                            if actual_val >= 1000.0 {
                                format!("{:.1}k A", actual_val / 1000.0)
                            } else if actual_val >= 10.0 {
                                format!("{:.0} A", actual_val)
                            } else if actual_val >= 1.0 {
                                format!("{:.1} A", actual_val)
                            } else {
                                format!("{:.2} A", actual_val)
                            }
                        } else {
                            format!("{:.1} A", mark.value)
                        }
                    })
                    .y_axis_formatter(move |mark, _range| {
                        if is_log {
                            let actual_val = 10.0f64.powf(mark.value);
                            if actual_val >= 100.0 {
                                format!("{:.0} s", actual_val)
                            } else if actual_val >= 1.0 {
                                format!("{:.1} s", actual_val)
                            } else if actual_val >= 0.01 {
                                format!("{:.2} s", actual_val)
                            } else if actual_val >= 0.001 {
                                format!("{:.0} ms", actual_val * 1000.0)
                            } else {
                                format!("{:.4} s", actual_val)
                            }
                        } else {
                            format!("{:.2} s", mark.value)
                        }
                    });

                plot.show(ui, |plot_ui| {
                    // 1. Draw Pickup Current vertical marker line
                    if config.show_pickup_line && pickup_current > 0.0 {
                        let pickup_x = if is_log { pickup_current.log10() } else { pickup_current };
                        plot_ui.vline(
                            VLine::new(pickup_x)
                                .stroke(Stroke::new(1.2_f32, Color32::from_rgb(100, 116, 139)))
                                .name(format!("Pickup Is ({:.2} A)", pickup_current)),
                        );
                    }

                    // 2. Draw Primary Fitted Curve
                    if let Some(best) = best_match {
                        let num_steps = 250;
                        let log_min = min_current.ln();
                        let log_max = max_current.ln();
                        let log_step = (log_max - log_min) / (num_steps as f64);

                        let mut curve_pts = Vec::with_capacity(num_steps);
                        for i in 0..=num_steps {
                            let curr = (log_min + (i as f64) * log_step).exp();
                            if let Some(t) = best.curve.calculate_operating_time(curr, pickup_current, best.dial_setting) {
                                if t > 0.0 && t.is_finite() {
                                    let (plot_x, plot_y) = if is_log {
                                        (curr.log10(), t.log10())
                                    } else {
                                        (curr, t)
                                    };
                                    curve_pts.push([plot_x, plot_y]);
                                }
                            }
                        }

                        if !curve_pts.is_empty() {
                            let label = format!(
                                "[Fitted] {} ({}) - {} = {:.3}",
                                best.curve.name,
                                best.curve.standard.as_str(),
                                best.curve.standard.dial_short_name(),
                                best.dial_setting
                            );
                            plot_ui.line(
                                Line::new(PlotPoints::new(curve_pts))
                                    .color(Theme::ACCENT_CYAN)
                                    .width(2.8_f32)
                                    .name(label),
                            );
                        }
                    }

                    // 3. Draw Runner-Up Curve (for visual comparison)
                    if config.show_runner_up {
                        if let Some(runner) = runner_up {
                            let num_steps = 200;
                            let log_min = min_current.ln();
                            let log_max = max_current.ln();
                            let log_step = (log_max - log_min) / (num_steps as f64);

                            let mut runner_pts = Vec::with_capacity(num_steps);
                            for i in 0..=num_steps {
                                let curr = (log_min + (i as f64) * log_step).exp();
                                if let Some(t) = runner.curve.calculate_operating_time(curr, pickup_current, runner.dial_setting) {
                                    if t > 0.0 && t.is_finite() {
                                        let (plot_x, plot_y) = if is_log {
                                            (curr.log10(), t.log10())
                                        } else {
                                            (curr, t)
                                        };
                                        runner_pts.push([plot_x, plot_y]);
                                    }
                                }
                            }

                            if !runner_pts.is_empty() {
                                let label = format!(
                                    "[2nd Match] {} - {} = {:.3}",
                                    runner.curve.name,
                                    runner.curve.standard.dial_short_name(),
                                    runner.dial_setting
                                );
                                plot_ui.line(
                                    Line::new(PlotPoints::new(runner_pts))
                                        .color(Theme::ACCENT_AMBER)
                                        .width(1.8_f32)
                                        .name(label),
                                );
                            }
                        }
                    }

                    // 4. Draw Residual error vectors
                    if config.show_residuals {
                        if let Some(best) = best_match {
                            for p in &active_points {
                                if let Some(t_est) = best.curve.calculate_operating_time(p.current, pickup_current, best.dial_setting) {
                                    let (x1, y1) = if is_log {
                                        (p.current.log10(), p.time.log10())
                                    } else {
                                        (p.current, p.time)
                                    };
                                    let (x2, y2) = if is_log {
                                        (p.current.log10(), t_est.log10())
                                    } else {
                                        (p.current, t_est)
                                    };

                                    plot_ui.line(
                                        Line::new(PlotPoints::new(vec![[x1, y1], [x2, y2]]))
                                            .color(Theme::ACCENT_ROSE)
                                            .width(1.5_f32),
                                    );
                                }
                            }
                        }
                    }

                    // 5. Draw Measured Test Point markers
                    let mut pt_coords = Vec::new();
                    for p in &active_points {
                        let (px, py) = if is_log {
                            (p.current.log10(), p.time.log10())
                        } else {
                            (p.current, p.time)
                        };
                        pt_coords.push([px, py]);
                    }

                    if !pt_coords.is_empty() {
                        plot_ui.points(
                            Points::new(PlotPoints::new(pt_coords))
                                .color(Theme::ACCENT_EMERALD)
                                .radius(6.0_f32)
                                .shape(MarkerShape::Circle)
                                .filled(true)
                                .name("Measured Points"),
                        );
                    }
                });
            });
        });
    }
}
