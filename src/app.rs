use crate::curves::CurveDefinition;
use crate::estimator::{EstimationReport, RelayEstimator, TestPoint};
use crate::plot_view::{PlotConfig, TccPlotView};
use crate::presets::PresetTestCase;
use crate::spreadsheet::SpreadsheetGrid;
use crate::theme::Theme;
use eframe::egui::{self, Align, Align2, Button, Color32, CornerRadius, FontId, Layout, RichText, ScrollArea, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    EstimationAndGraph,
    CandidateRankings,
    ForwardSimulator,
    FormulaReference,
}

pub struct RelayCurveApp {
    // Inputs
    pub pickup_current: f64,
    pub pickup_input_str: String,
    pub test_points: Vec<TestPoint>,
    pub spreadsheet: SpreadsheetGrid,

    // Solver state
    pub last_report: Option<EstimationReport>,
    pub error_message: Option<String>,
    pub selected_preset_idx: usize,

    // UI state
    pub active_tab: AppTab,
    pub plot_config: PlotConfig,

    // Forward simulator state
    pub sim_current: f64,
    pub sim_current_str: String,
    pub sim_dial_override: f64,
    pub sim_selected_curve_idx: usize,

    // Status / feedback toast
    pub toast_message: Option<(String, std::time::Instant)>,
}

impl Default for RelayCurveApp {
    fn default() -> Self {
        let presets = PresetTestCase::all();
        let default_preset = &presets[0];

        let mut app = Self {
            pickup_current: default_preset.pickup_current,
            pickup_input_str: format!("{:.2}", default_preset.pickup_current),
            test_points: default_preset.points.clone(),
            spreadsheet: SpreadsheetGrid::default(),
            last_report: None,
            error_message: None,
            selected_preset_idx: 0,
            active_tab: AppTab::EstimationAndGraph,
            plot_config: PlotConfig::default(),
            sim_current: 3.0,
            sim_current_str: "3.00".to_string(),
            sim_dial_override: 1.0,
            sim_selected_curve_idx: 0,
            toast_message: None,
        };

        app.run_estimation();
        app
    }
}

/// Helper to render high-contrast dark text edit fields with bright white text
fn dark_text_edit(ui: &mut egui::Ui, text: &mut String, width: f32) -> egui::Response {
    ui.scope(|ui| {
        ui.visuals_mut().extreme_bg_color = Theme::BG_INPUT;
        ui.visuals_mut().widgets.inactive.bg_fill = Theme::BG_INPUT;
        ui.visuals_mut().widgets.inactive.weak_bg_fill = Theme::BG_INPUT;
        ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Theme::BORDER_INPUT);
        ui.visuals_mut().widgets.hovered.bg_fill = Theme::BG_CARD_ALT;
        ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Theme::ACCENT_CYAN);
        ui.visuals_mut().widgets.active.bg_fill = Theme::BG_INPUT;
        ui.visuals_mut().widgets.active.bg_stroke = Stroke::new(1.5_f32, Theme::ACCENT_CYAN);
        ui.add(
            egui::TextEdit::singleline(text)
                .desired_width(width)
                .text_color(Theme::TEXT_WHITE)
                .font(egui::TextStyle::Monospace),
        )
    }).inner
}

/// Custom centered button helper
fn centered_action_button(ui: &mut egui::Ui, text: &str, height: f32, bg_color: Color32, hover_color: Color32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let fill = if response.is_pointer_button_down_on() {
            Color32::from_rgb(2, 90, 140)
        } else if response.hovered() {
            hover_color
        } else {
            bg_color
        };

        ui.painter().rect_filled(rect, CornerRadius::same(6), fill);
        ui.painter().rect_stroke(rect, CornerRadius::same(6), Stroke::new(1.0_f32, Theme::BORDER_ACTIVE), egui::StrokeKind::Inside);
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            text,
            FontId::proportional(13.5),
            Theme::TEXT_WHITE,
        );
    }
    response
}

/// Dedicated vector math equation painter for IEC 60255
fn render_iec_math_card(ui: &mut egui::Ui) {
    Theme::card_frame_subtle().show(ui, |ui| {
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(220.0, 52.0), egui::Sense::hover());
            if ui.is_rect_visible(rect) {
                let cy = rect.center().y;
                let lx = rect.left() + 8.0;

                // "t  ="
                ui.painter().text(
                    egui::pos2(lx, cy),
                    Align2::LEFT_CENTER,
                    "t  =",
                    FontId::proportional(17.0),
                    Theme::TEXT_WHITE,
                );

                let frac_x = lx + 105.0;

                // Numerator: k · TMS
                ui.painter().text(
                    egui::pos2(frac_x, cy - 12.0),
                    Align2::CENTER_CENTER,
                    "k · TMS",
                    FontId::proportional(14.0),
                    Theme::ACCENT_CYAN,
                );

                // Fraction divider bar
                ui.painter().line_segment(
                    [egui::pos2(frac_x - 55.0, cy), egui::pos2(frac_x + 55.0, cy)],
                    Stroke::new(1.5_f32, Theme::TEXT_SECONDARY),
                );

                // Denominator: (I / Is)^α − 1
                ui.painter().text(
                    egui::pos2(frac_x, cy + 12.0),
                    Align2::CENTER_CENTER,
                    "(I / Is)^α − 1",
                    FontId::proportional(14.0),
                    Theme::TEXT_WHITE,
                );
            }

            ui.add_space(20.0);
            ui.vertical(|ui| {
                ui.label(RichText::new("Standard IEC 60255 Overcurrent Characteristic").strong().size(12.0).color(Theme::TEXT_SECONDARY));
                ui.label(RichText::new("LaTeX:  t = \\frac{k \\cdot \\text{TMS}}{(I/I_s)^\\alpha - 1}").monospace().size(11.0).color(Theme::TEXT_MUTED));
            });
        });
    });
}

/// Dedicated vector math equation painter for IEEE C37.112
fn render_ieee_math_card(ui: &mut egui::Ui) {
    Theme::card_frame_subtle().show(ui, |ui| {
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(310.0, 52.0), egui::Sense::hover());
            if ui.is_rect_visible(rect) {
                let cy = rect.center().y;
                let lx = rect.left() + 8.0;

                // "t  =  TD · ["
                ui.painter().text(
                    egui::pos2(lx, cy),
                    Align2::LEFT_CENTER,
                    "t  =  TD · [",
                    FontId::proportional(17.0),
                    Theme::TEXT_WHITE,
                );

                let frac_x = lx + 140.0;

                // Numerator: A
                ui.painter().text(
                    egui::pos2(frac_x, cy - 12.0),
                    Align2::CENTER_CENTER,
                    "A",
                    FontId::proportional(14.0),
                    Theme::ACCENT_CYAN,
                );

                // Fraction divider bar
                ui.painter().line_segment(
                    [egui::pos2(frac_x - 50.0, cy), egui::pos2(frac_x + 50.0, cy)],
                    Stroke::new(1.5_f32, Theme::TEXT_SECONDARY),
                );

                // Denominator: (I / Is)^p − 1
                ui.painter().text(
                    egui::pos2(frac_x, cy + 12.0),
                    Align2::CENTER_CENTER,
                    "(I / Is)^p − 1",
                    FontId::proportional(14.0),
                    Theme::TEXT_WHITE,
                );

                // "+  B ]"
                ui.painter().text(
                    egui::pos2(frac_x + 58.0, cy),
                    Align2::LEFT_CENTER,
                    "+  B ]",
                    FontId::proportional(17.0),
                    Theme::TEXT_WHITE,
                );
            }

            ui.add_space(20.0);
            ui.vertical(|ui| {
                ui.label(RichText::new("Standard IEEE C37.112 Overcurrent Characteristic").strong().size(12.0).color(Theme::TEXT_SECONDARY));
                ui.label(RichText::new("LaTeX:  t = \\text{TD} \\cdot \\left[ \\frac{A}{(I/I_s)^p - 1} + B \\right]").monospace().size(11.0).color(Theme::TEXT_MUTED));
            });
        });
    });
}

impl RelayCurveApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Theme::apply_custom_dark_visuals(&cc.egui_ctx);
        Self::default()
    }

    pub fn set_toast(&mut self, msg: impl Into<String>) {
        self.toast_message = Some((msg.into(), std::time::Instant::now()));
    }

    pub fn run_estimation(&mut self) {
        if let Ok(val) = self.pickup_input_str.trim().parse::<f64>() {
            self.pickup_current = val;
        }

        match RelayEstimator::estimate(self.pickup_current, &self.test_points) {
            Ok(report) => {
                if let Some(best) = report.all_ranked.first() {
                    self.sim_dial_override = best.dial_setting;
                }
                self.last_report = Some(report);
                self.error_message = None;
            }
            Err(err) => {
                self.error_message = Some(err);
                self.last_report = None;
            }
        }
    }

    pub fn load_preset(&mut self, index: usize) {
        let presets = PresetTestCase::all();
        if let Some(p) = presets.get(index) {
            self.selected_preset_idx = index;
            self.pickup_current = p.pickup_current;
            self.pickup_input_str = format!("{:.2}", p.pickup_current);
            self.test_points = p.points.clone();
            for pt in &mut self.test_points {
                pt.sync_strings();
            }
            self.spreadsheet.select_cell(0, 0);
            self.run_estimation();
            self.set_toast(format!("Loaded preset: {}", p.name));
        }
    }

    pub fn copy_summary_to_clipboard(&mut self, ctx: &egui::Context) {
        if let Some(report) = &self.last_report {
            let best = &report.best_match;
            let mut summary = format!(
                "=== RELAY CURVE ESTIMATION REPORT ===\n\
                Standard: {}\n\
                Estimated Curve: {} ({})\n\
                Pickup Current (Is): {:.3} A\n\
                Estimated {}: {:.4}\n\
                RMSE Error: {:.6} s\n\
                Mean Square Error (MSE): {:.6}\n\
                Max Relative Error: {:.2}%\n\
                Fit Quality Score: {:.1}%\n\
                Valid Points Evaluated: {}\n\n\
                --- Verification Points ---\n\
                Current(A) | Actual(s) | Estimated(s) | Error(s) | Error(%)\n",
                best.curve.standard.as_str(),
                best.curve.name,
                best.curve.code,
                self.pickup_current,
                best.curve.standard.dial_name(),
                best.dial_setting,
                best.rmse,
                best.mse,
                best.max_rel_error_percent,
                best.fit_quality_score,
                report.valid_points_used
            );

            for v in &best.verifications {
                summary.push_str(&format!(
                    "{:10.3} | {:9.3} | {:12.3} | {:8.3} | {:7.2}%\n",
                    v.current, v.actual_time, v.estimated_time, v.abs_error, v.rel_error_percent
                ));
            }

            ctx.copy_text(summary);
            self.set_toast("Estimation summary copied to clipboard.");
        }
    }

    pub fn export_csv(&mut self) {
        if let Some(report) = &self.last_report {
            let best = &report.best_match;
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("CSV File", &["csv"])
                .set_file_name("relay_curve_results.csv")
                .save_file()
            {
                let mut csv_content = format!(
                    "# Relay Curve Estimation Export\n\
                    # Standard: {}\n\
                    # Curve: {}\n\
                    # Pickup Current (A): {:.4}\n\
                    # {}: {:.4}\n\
                    # RMSE: {:.6}\n\
                    Current_A,Actual_Time_s,Estimated_Time_s,Abs_Error_s,Rel_Error_Percent\n",
                    best.curve.standard.as_str(),
                    best.curve.name,
                    self.pickup_current,
                    best.curve.standard.dial_short_name(),
                    best.dial_setting,
                    best.rmse
                );

                for v in &best.verifications {
                    csv_content.push_str(&format!(
                        "{:.4},{:.4},{:.4},{:.4},{:.2}\n",
                        v.current, v.actual_time, v.estimated_time, v.abs_error, v.rel_error_percent
                    ));
                }

                if let Err(e) = std::fs::write(&path, csv_content) {
                    self.set_toast(format!("Failed to save CSV: {}", e));
                } else {
                    self.set_toast(format!("Exported CSV to {}", path.display()));
                }
            }
        }
    }

    fn render_nav_tab(&mut self, ui: &mut egui::Ui, tab: AppTab, label: &str) {
        let is_selected = self.active_tab == tab;
        let bg_color = if is_selected {
            Theme::ACCENT_PRIMARY
        } else {
            Theme::BG_CARD
        };
        let text_color = if is_selected {
            Theme::TEXT_WHITE
        } else {
            Theme::TEXT_SECONDARY
        };
        let stroke = if is_selected {
            Stroke::new(1.5_f32, Theme::ACCENT_CYAN)
        } else {
            Stroke::new(1.0_f32, Theme::BORDER_CARD)
        };

        let btn = Button::new(RichText::new(label).strong().size(13.0).color(text_color))
            .fill(bg_color)
            .stroke(stroke)
            .corner_radius(CornerRadius::same(5))
            .min_size(Vec2::new(150.0, 30.0));

        if ui.add(btn).clicked() {
            self.active_tab = tab;
        }
    }
}

impl eframe::App for RelayCurveApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Theme::apply_custom_dark_visuals(ctx);

        if let Some((_, time)) = self.toast_message {
            if time.elapsed().as_secs() > 4 {
                self.toast_message = None;
            }
        }

        // Top Navigation Bar
        egui::TopBottomPanel::top("top_panel").frame(Theme::card_frame()).show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Application Title
                ui.label(RichText::new("RELAY CURVE ESTIMATOR").strong().size(16.0).color(Theme::ACCENT_CYAN));
                ui.label(RichText::new("v0.2.0").size(11.0).color(Theme::TEXT_MUTED));

                ui.add_space(20.0);

                // High-Contrast Navigation Tabs
                self.render_nav_tab(ui, AppTab::EstimationAndGraph, "Estimation & Graph");
                self.render_nav_tab(ui, AppTab::CandidateRankings, "Curve Rankings");
                self.render_nav_tab(ui, AppTab::ForwardSimulator, "Trip Simulator");
                self.render_nav_tab(ui, AppTab::FormulaReference, "Reference & Formulas");

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.add(
                        Button::new(RichText::new("Copy Summary").color(Theme::TEXT_WHITE).size(12.0))
                            .fill(Theme::BG_CARD_ALT)
                            .stroke(Stroke::new(1.0_f32, Theme::BORDER_CARD))
                    ).clicked() {
                        self.copy_summary_to_clipboard(ctx);
                    }

                    if ui.add(
                        Button::new(RichText::new("Export CSV").color(Theme::TEXT_WHITE).size(12.0))
                            .fill(Theme::BG_CARD_ALT)
                            .stroke(Stroke::new(1.0_f32, Theme::BORDER_CARD))
                    ).clicked() {
                        self.export_csv();
                    }

                    if let Some((toast, _)) = &self.toast_message {
                        ui.label(RichText::new(toast).color(Theme::ACCENT_EMERALD).strong().size(12.0));
                    }
                });
            });
        });

        // Left Configuration Sidebar
        egui::SidePanel::left("left_sidebar")
            .resizable(true)
            .default_width(340.0)
            .width_range(300.0..=480.0)
            .frame(Theme::card_frame())
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.render_sidebar(ui, ctx);
                });
            });

        // Central Main Workspace
        egui::CentralPanel::default().frame(Theme::card_frame_subtle()).show(ctx, |ui| {
            match self.active_tab {
                AppTab::EstimationAndGraph => self.render_estimation_tab(ui),
                AppTab::CandidateRankings => self.render_rankings_tab(ui),
                AppTab::ForwardSimulator => self.render_simulator_tab(ui),
                AppTab::FormulaReference => self.render_reference_tab(ui),
            }
        });
    }
}

impl RelayCurveApp {
    fn render_sidebar(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.label(RichText::new("CONFIGURATION & TEST DATA").strong().size(14.0).color(Theme::TEXT_WHITE));
        ui.add_space(8.0);

        // Preset selector card (Dark styled dropdown)
        Theme::card_frame_subtle().show(ui, |ui| {
            ui.label(RichText::new("Standard Relay Presets:").size(12.0).color(Theme::TEXT_SECONDARY));
            let presets = PresetTestCase::all();
            let current_preset_name = presets.get(self.selected_preset_idx).map(|p| p.name).unwrap_or("Select Preset");

            ui.scope(|ui| {
                ui.visuals_mut().widgets.inactive.bg_fill = Theme::BG_INPUT;
                ui.visuals_mut().widgets.inactive.weak_bg_fill = Theme::BG_INPUT;
                ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Theme::BORDER_INPUT);
                ui.visuals_mut().widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Theme::TEXT_WHITE);
                ui.visuals_mut().widgets.hovered.bg_fill = Theme::BG_CARD_ALT;
                ui.visuals_mut().widgets.active.bg_fill = Theme::BG_INPUT;

                egui::ComboBox::from_id_salt("preset_select")
                    .selected_text(RichText::new(current_preset_name).color(Theme::TEXT_WHITE).size(12.0))
                    .width(ui.available_width() - 8.0)
                    .show_ui(ui, |ui| {
                        for (i, p) in presets.iter().enumerate() {
                            if ui.selectable_label(self.selected_preset_idx == i, RichText::new(p.name).color(Theme::TEXT_WHITE)).clicked() {
                                self.load_preset(i);
                            }
                        }
                    });
            });
        });

        ui.add_space(10.0);

        let mut needs_recalc = false;

        // Pickup Current Input Card
        Theme::card_frame().show(ui, |ui| {
            ui.label(RichText::new("Pickup Current (Is):").strong().color(Theme::ACCENT_CYAN));
            ui.horizontal(|ui| {
                let response = dark_text_edit(ui, &mut self.pickup_input_str, 110.0);
                ui.label(RichText::new("Amperes (A)").color(Theme::TEXT_MUTED));

                if response.changed() {
                    if let Ok(val) = self.pickup_input_str.trim().parse::<f64>() {
                        if val > 0.0 {
                            self.pickup_current = val;
                            needs_recalc = true;
                        }
                    }
                }
            });

            ui.add_space(6.0);

            // Quick set pills
            ui.horizontal(|ui| {
                ui.label(RichText::new("Quick set:").size(11.0).color(Theme::TEXT_MUTED));
                for val in [0.5, 1.0, 2.0, 5.0, 10.0] {
                    if ui.add(
                        Button::new(RichText::new(format!("{val:.1}A")).size(11.0).color(Theme::TEXT_SECONDARY))
                            .fill(Theme::BG_CARD_ALT)
                            .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                    ).clicked() {
                        self.pickup_input_str = format!("{val:.2}");
                        self.pickup_current = val;
                        needs_recalc = true;
                    }
                }
            });
        });

        ui.add_space(12.0);

        // Measured Test Points - Authentic Excel Spreadsheet Grid
        Theme::card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("TEST POINTS").strong().size(12.0).color(Theme::TEXT_WHITE));
                ui.label(RichText::new(format!("({} points)", self.test_points.len())).size(11.0).color(Theme::TEXT_MUTED));
            });

            ui.add_space(6.0);

            self.spreadsheet.show(
                ui,
                &mut self.test_points,
                self.pickup_current,
                &mut needs_recalc,
                &mut self.toast_message,
            );
        });

        if needs_recalc {
            self.run_estimation();
        }

        ui.add_space(14.0);

        // Centered Action Button
        if centered_action_button(ui, "ESTIMATE BEST CURVE", 38.0, Theme::ACCENT_PRIMARY, Theme::ACCENT_CYAN).clicked() {
            self.run_estimation();
            self.set_toast("Recalculated all candidate curve fits.");
        }

        if let Some(report) = &self.last_report {
            ui.add_space(6.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(format!(
                    "Solved in {} µs | Valid Points: {}/{}",
                    report.elapsed_micros,
                    report.valid_points_used,
                    report.total_points_provided
                )).size(11.0).color(Theme::TEXT_MUTED));
            });
        }

        if let Some(err) = &self.error_message {
            ui.add_space(8.0);
            Theme::card_frame_subtle().show(ui, |ui| {
                ui.label(RichText::new(format!("[Warning] {err}")).color(Theme::ACCENT_ROSE).size(12.0));
            });
        }
    }

    fn render_estimation_tab(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            // 1. Structured Engineering Best Match Card
            if let Some(report) = &self.last_report {
                let best = &report.best_match;
                Theme::card_frame().show(ui, |ui| {
                    ui.vertical(|ui| {
                        // Title bar
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(" OPTIMAL FIT ")
                                    .strong()
                                    .size(11.0)
                                    .color(Color32::BLACK)
                                    .background_color(Theme::ACCENT_EMERALD)
                            );
                            ui.label(RichText::new(&best.curve.name).strong().size(16.0).color(Theme::TEXT_WHITE));
                            ui.label(RichText::new(format!("({})", best.curve.standard.as_str())).size(12.0).color(Theme::TEXT_MUTED));
                        });

                        ui.add_space(6.0);
                        ui.label(RichText::new(&best.curve.description).size(12.0).color(Theme::TEXT_SECONDARY));
                        ui.add_space(10.0);

                        // 4-Column Stat Metrics Grid (consistent layout and sizing)
                        ui.columns(4, |cols| {
                            cols[0].group(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(RichText::new(best.curve.standard.dial_short_name()).size(10.0).color(Theme::TEXT_MUTED));
                                    ui.label(RichText::new(format!("{:.4}", best.dial_setting)).strong().size(15.0).color(Theme::ACCENT_CYAN));
                                });
                            });

                            cols[1].group(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(RichText::new("FIT QUALITY").size(10.0).color(Theme::TEXT_MUTED));
                                    ui.label(RichText::new(format!("{:.1}%", best.fit_quality_score)).strong().size(15.0).color(Theme::ACCENT_EMERALD));
                                });
                            });

                            cols[2].group(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(RichText::new("RMSE ERROR").size(10.0).color(Theme::TEXT_MUTED));
                                    ui.label(RichText::new(format!("{:.4} s", best.rmse)).strong().size(15.0).color(Theme::TEXT_WHITE));
                                });
                            });

                            cols[3].group(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(RichText::new("MAX ERROR").size(10.0).color(Theme::TEXT_MUTED));
                                    ui.label(RichText::new(format!("{:.2}%", best.max_rel_error_percent)).strong().size(15.0).color(Theme::TEXT_WHITE));
                                });
                            });
                        });
                    });
                });
            }

            ui.add_space(8.0);

            // 2. Interactive TCC Graph
            let runner_up = self.last_report.as_ref().and_then(|r| r.all_ranked.get(1));
            let best_match = self.last_report.as_ref().map(|r| &r.best_match);

            ui.group(|ui| {
                ui.set_min_height(390.0);
                TccPlotView::show(
                    ui,
                    &mut self.plot_config,
                    self.pickup_current,
                    &self.test_points,
                    best_match,
                    runner_up,
                );
            });

            ui.add_space(10.0);

            // 3. Verification Data Table
            if let Some(report) = &self.last_report {
                let best = &report.best_match;
                ui.label(RichText::new("Verification & Residual Error Analysis").strong().size(14.0).color(Theme::TEXT_WHITE));
                ui.add_space(4.0);

                Theme::card_frame().show(ui, |ui| {
                    egui::Grid::new("verification_grid")
                        .striped(true)
                        .min_col_width(100.0)
                        .spacing([16.0, 6.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("Current (A)").strong().color(Theme::TEXT_SECONDARY));
                            ui.label(RichText::new("Multiple (I/Is)").strong().color(Theme::TEXT_SECONDARY));
                            ui.label(RichText::new("Actual Time (s)").strong().color(Theme::TEXT_SECONDARY));
                            ui.label(RichText::new("Fitted Time (s)").strong().color(Theme::TEXT_SECONDARY));
                            ui.label(RichText::new("Delta Error (s)").strong().color(Theme::TEXT_SECONDARY));
                            ui.label(RichText::new("Rel. Error (%)").strong().color(Theme::TEXT_SECONDARY));
                            ui.end_row();

                            for v in &best.verifications {
                                ui.label(RichText::new(format!("{:.3}", v.current)).color(Theme::TEXT_WHITE));
                                ui.label(RichText::new(format!("{:.2}x", v.current / self.pickup_current)).color(Theme::TEXT_MUTED));
                                ui.label(RichText::new(format!("{:.4}", v.actual_time)).color(Theme::TEXT_WHITE));
                                ui.label(RichText::new(format!("{:.4}", v.estimated_time)).color(Theme::ACCENT_CYAN));

                                let err_color = if v.abs_error < 0.05 {
                                    Theme::ACCENT_EMERALD
                                } else if v.abs_error < 0.5 {
                                    Theme::ACCENT_AMBER
                                } else {
                                    Theme::ACCENT_ROSE
                                };
                                ui.label(RichText::new(format!("{:.4}", v.abs_error)).color(err_color));
                                ui.label(RichText::new(format!("{:.2}%", v.rel_error_percent)).color(err_color));
                                ui.end_row();
                            }
                        });
                });
            }
        });
    }

    fn render_rankings_tab(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.label(RichText::new("All Candidate Curve Rankings").strong().size(16.0).color(Theme::TEXT_WHITE));
            ui.label(RichText::new("Comparison of all IEC 60255 and IEEE C37.112 curve families fitted to your test points, ranked by lowest Root Mean Square Error (RMSE).").size(12.0).color(Theme::TEXT_SECONDARY));
            ui.add_space(10.0);

            if let Some(report) = &self.last_report {
                Theme::card_frame().show(ui, |ui| {
                    egui::Grid::new("rankings_grid")
                        .striped(true)
                        .min_col_width(85.0)
                        .spacing([14.0, 8.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("Rank").strong().color(Theme::TEXT_SECONDARY));
                            ui.label(RichText::new("Standard").strong().color(Theme::TEXT_SECONDARY));
                            ui.label(RichText::new("Curve Name").strong().color(Theme::TEXT_SECONDARY));
                            ui.label(RichText::new("Optimal Dial").strong().color(Theme::TEXT_SECONDARY));
                            ui.label(RichText::new("Fit Quality").strong().color(Theme::TEXT_SECONDARY));
                            ui.label(RichText::new("RMSE (s)").strong().color(Theme::TEXT_SECONDARY));
                            ui.label(RichText::new("MSE").strong().color(Theme::TEXT_SECONDARY));
                            ui.label(RichText::new("Max Error %").strong().color(Theme::TEXT_SECONDARY));
                            ui.end_row();

                            for (idx, r) in report.all_ranked.iter().enumerate() {
                                let is_best = idx == 0;
                                let rank_text = if is_best { "#1 (Optimal)".to_string() } else { format!("#{}", idx + 1) };
                                let rank_color = if is_best { Theme::ACCENT_EMERALD } else { Theme::TEXT_MUTED };

                                ui.label(RichText::new(rank_text).strong().color(rank_color));
                                ui.label(RichText::new(r.curve.standard.as_str()).color(Theme::TEXT_MUTED));
                                ui.label(RichText::new(&r.curve.name).strong().color(if is_best { Theme::TEXT_WHITE } else { Theme::TEXT_SECONDARY }));
                                ui.label(RichText::new(format!("{} = {:.4}", r.curve.standard.dial_short_name(), r.dial_setting)).color(Theme::TEXT_WHITE));

                                let score_color = if r.fit_quality_score > 95.0 {
                                    Theme::ACCENT_EMERALD
                                } else if r.fit_quality_score > 75.0 {
                                    Theme::ACCENT_AMBER
                                } else {
                                    Theme::ACCENT_ROSE
                                };
                                ui.label(RichText::new(format!("{:.1}%", r.fit_quality_score)).strong().color(score_color));
                                ui.label(RichText::new(format!("{:.6}", r.rmse)).color(Theme::TEXT_WHITE));
                                ui.label(RichText::new(format!("{:.6}", r.mse)).color(Theme::TEXT_MUTED));
                                ui.label(RichText::new(format!("{:.2}%", r.max_rel_error_percent)).color(Theme::TEXT_WHITE));
                                ui.end_row();
                            }
                        });
                });
            } else {
                ui.label(RichText::new("No estimation results available. Check test points in sidebar.").color(Theme::TEXT_MUTED));
            }
        });
    }

    fn render_simulator_tab(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.label(RichText::new("Forward Trip Time Calculator & Simulator").strong().size(16.0).color(Theme::TEXT_WHITE));
            ui.label(RichText::new("Evaluate operating trip times for arbitrary fault currents and interactively adjust Time Dial settings.").size(12.0).color(Theme::TEXT_SECONDARY));
            ui.add_space(12.0);

            let all_curves = CurveDefinition::all();

            Theme::card_frame().show(ui, |ui| {
                // Curve Family Selector (Dark styled dropdown)
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Select Curve Family:").strong().color(Theme::TEXT_SECONDARY));
                    let current_curve_name = all_curves.get(self.sim_selected_curve_idx).map(|c| c.name.as_str()).unwrap_or("Select");

                    ui.scope(|ui| {
                        ui.visuals_mut().widgets.inactive.bg_fill = Theme::BG_INPUT;
                        ui.visuals_mut().widgets.inactive.weak_bg_fill = Theme::BG_INPUT;
                        ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Theme::BORDER_INPUT);
                        ui.visuals_mut().widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Theme::TEXT_WHITE);
                        ui.visuals_mut().widgets.hovered.bg_fill = Theme::BG_CARD_ALT;
                        ui.visuals_mut().widgets.active.bg_fill = Theme::BG_INPUT;

                        egui::ComboBox::from_id_salt("sim_curve_select")
                            .selected_text(RichText::new(current_curve_name).color(Theme::TEXT_WHITE).size(12.0))
                            .show_ui(ui, |ui| {
                                for (i, c) in all_curves.iter().enumerate() {
                                    if ui.selectable_label(self.sim_selected_curve_idx == i, RichText::new(format!("{} ({})", c.name, c.standard.as_str())).color(Theme::TEXT_WHITE)).clicked() {
                                        self.sim_selected_curve_idx = i;
                                    }
                                }
                            });
                    });
                });

                ui.add_space(8.0);

                // Dial setting slider (Dark styled slider and value box)
                let selected_curve = &all_curves[self.sim_selected_curve_idx.min(all_curves.len() - 1)];
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{}:", selected_curve.standard.dial_name())).strong().color(Theme::TEXT_SECONDARY));
                    ui.scope(|ui| {
                        ui.visuals_mut().widgets.inactive.bg_fill = Theme::BG_INPUT;
                        ui.visuals_mut().widgets.inactive.weak_bg_fill = Theme::BG_INPUT;
                        ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Theme::BORDER_INPUT);
                        ui.visuals_mut().widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Theme::TEXT_WHITE);
                        ui.visuals_mut().widgets.hovered.bg_fill = Theme::BG_CARD_ALT;
                        ui.visuals_mut().widgets.active.bg_fill = Theme::ACCENT_PRIMARY;
                        ui.visuals_mut().extreme_bg_color = Theme::BG_INPUT;
                        ui.visuals_mut().slider_trailing_fill = true;

                        ui.add(egui::Slider::new(&mut self.sim_dial_override, 0.01..=5.0).step_by(0.01).logarithmic(false));
                    });
                });

                ui.add_space(8.0);

                // Current input
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Fault Current (A):").strong().color(Theme::TEXT_SECONDARY));
                    let resp_sim = dark_text_edit(ui, &mut self.sim_current_str, 100.0);

                    if resp_sim.changed() {
                        if let Ok(c) = self.sim_current_str.trim().parse::<f64>() {
                            self.sim_current = c;
                        }
                    }

                    if self.pickup_current > 0.0 {
                        ui.label(RichText::new(format!("({:.2}x Pickup)", self.sim_current / self.pickup_current)).color(Theme::TEXT_MUTED));
                    }
                });

                ui.add_space(14.0);

                // Output Result Card (Clean horizontal layout)
                if let Some(trip_time) = selected_curve.calculate_operating_time(self.sim_current, self.pickup_current, self.sim_dial_override) {
                    Theme::card_frame_subtle().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Calculated Operating Time:").size(13.0).color(Theme::TEXT_SECONDARY));
                            ui.label(RichText::new(format!("{:.4} s", trip_time)).strong().size(18.0).color(Theme::ACCENT_CYAN));
                            ui.label(RichText::new(format!("({:.1} ms)", trip_time * 1000.0)).size(13.0).color(Theme::TEXT_WHITE));
                        });
                    });
                } else {
                    Theme::card_frame_subtle().show(ui, |ui| {
                        ui.label(RichText::new("Fault current must exceed Pickup Current (Is) to calculate trip time.").color(Theme::ACCENT_ROSE));
                    });
                }
            });

            ui.add_space(14.0);

            // Multi-point lookup table
            ui.label(RichText::new("Calculated Trip Times at Common Multiples of Pickup:").strong().size(13.0).color(Theme::TEXT_WHITE));
            Theme::card_frame().show(ui, |ui| {
                let selected_curve = &all_curves[self.sim_selected_curve_idx.min(all_curves.len() - 1)];
                egui::Grid::new("sim_multiples_grid")
                    .striped(true)
                    .min_col_width(110.0)
                    .spacing([20.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Multiple (I/Is)").strong().color(Theme::TEXT_SECONDARY));
                        ui.label(RichText::new("Current (A)").strong().color(Theme::TEXT_SECONDARY));
                        ui.label(RichText::new("Operating Time (s)").strong().color(Theme::TEXT_SECONDARY));
                        ui.label(RichText::new("Operating Time (ms)").strong().color(Theme::TEXT_SECONDARY));
                        ui.end_row();

                        for mult in [1.5, 2.0, 3.0, 4.0, 5.0, 8.0, 10.0, 15.0, 20.0] {
                            let curr = self.pickup_current * mult;
                            let time = selected_curve.calculate_operating_time(curr, self.pickup_current, self.sim_dial_override);

                            ui.label(RichText::new(format!("{mult:.1}x")).color(Theme::TEXT_MUTED));
                            ui.label(RichText::new(format!("{curr:.2} A")).color(Theme::TEXT_WHITE));
                            if let Some(t) = time {
                                ui.label(RichText::new(format!("{:.4} s", t)).color(Theme::ACCENT_CYAN));
                                ui.label(RichText::new(format!("{:.1} ms", t * 1000.0)).color(Theme::TEXT_WHITE));
                            } else {
                                ui.label("-");
                                ui.label("-");
                            }
                            ui.end_row();
                        }
                    });
            });
        });
    }

    fn render_reference_tab(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.label(RichText::new("Protection Relay Curve Formulas & Standards").strong().size(16.0).color(Theme::TEXT_WHITE));
            ui.label(RichText::new("Mathematical definitions and standard characteristics according to IEC 60255 and IEEE C37.112.").size(12.0).color(Theme::TEXT_SECONDARY));
            ui.add_space(10.0);

            // 1. IEC Standards Section
            Theme::card_frame().show(ui, |ui| {
                ui.label(RichText::new("1. IEC 60255 Standard Characteristic Formula").strong().size(14.0).color(Theme::ACCENT_CYAN));
                ui.add_space(6.0);

                // Math Equation Box
                render_iec_math_card(ui);

                ui.add_space(10.0);

                egui::Grid::new("iec_ref_grid")
                    .striped(true)
                    .min_col_width(120.0)
                    .spacing([16.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Curve Name").strong().color(Theme::TEXT_SECONDARY));
                        ui.label(RichText::new("Constant (k)").strong().color(Theme::TEXT_SECONDARY));
                        ui.label(RichText::new("Exponent (α)").strong().color(Theme::TEXT_SECONDARY));
                        ui.label(RichText::new("Primary Application").strong().color(Theme::TEXT_SECONDARY));
                        ui.end_row();

                        ui.label(RichText::new("Standard Inverse (SI)").color(Theme::TEXT_WHITE));
                        ui.label("0.14");
                        ui.label("0.02");
                        ui.label(RichText::new("General distribution feeders").color(Theme::TEXT_MUTED));
                        ui.end_row();

                        ui.label(RichText::new("Very Inverse (VI)").color(Theme::TEXT_WHITE));
                        ui.label("13.50");
                        ui.label("1.00");
                        ui.label(RichText::new("Feeders with substantial fault current drop").color(Theme::TEXT_MUTED));
                        ui.end_row();

                        ui.label(RichText::new("Extremely Inverse (EI)").color(Theme::TEXT_WHITE));
                        ui.label("80.00");
                        ui.label("2.00");
                        ui.label(RichText::new("Transformer inrush, fuse coordination").color(Theme::TEXT_MUTED));
                        ui.end_row();

                        ui.label(RichText::new("Long Time Inverse (LTI)").color(Theme::TEXT_WHITE));
                        ui.label("120.00");
                        ui.label("1.00");
                        ui.label(RichText::new("Motor overload, backup thermal protection").color(Theme::TEXT_MUTED));
                        ui.end_row();
                    });
            });

            ui.add_space(12.0);

            // 2. IEEE Standards Section
            Theme::card_frame().show(ui, |ui| {
                ui.label(RichText::new("2. IEEE C37.112 Standard Characteristic Formula").strong().size(14.0).color(Theme::ACCENT_CYAN));
                ui.add_space(6.0);

                // Math Equation Box
                render_ieee_math_card(ui);

                ui.add_space(10.0);

                egui::Grid::new("ieee_ref_grid")
                    .striped(true)
                    .min_col_width(120.0)
                    .spacing([16.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Curve Name").strong().color(Theme::TEXT_SECONDARY));
                        ui.label(RichText::new("A").strong().color(Theme::TEXT_SECONDARY));
                        ui.label(RichText::new("B").strong().color(Theme::TEXT_SECONDARY));
                        ui.label(RichText::new("p").strong().color(Theme::TEXT_SECONDARY));
                        ui.label(RichText::new("Primary Application").strong().color(Theme::TEXT_SECONDARY));
                        ui.end_row();

                        ui.label(RichText::new("Moderately Inverse (MI)").color(Theme::TEXT_WHITE));
                        ui.label("0.0515");
                        ui.label("0.1140");
                        ui.label("0.02");
                        ui.label(RichText::new("General distribution coordination").color(Theme::TEXT_MUTED));
                        ui.end_row();

                        ui.label(RichText::new("Very Inverse (VI)").color(Theme::TEXT_WHITE));
                        ui.label("19.610");
                        ui.label("0.4910");
                        ui.label("2.00");
                        ui.label(RichText::new("Steep inverse overcurrent clearing").color(Theme::TEXT_MUTED));
                        ui.end_row();

                        ui.label(RichText::new("Extremely Inverse (EI)").color(Theme::TEXT_WHITE));
                        ui.label("28.200");
                        ui.label("0.1217");
                        ui.label("2.00");
                        ui.label(RichText::new("High fault level instantaneous backup").color(Theme::TEXT_MUTED));
                        ui.end_row();

                        ui.label(RichText::new("Short-Time Inverse (SI)").color(Theme::TEXT_WHITE));
                        ui.label("0.16758");
                        ui.label("0.11858");
                        ui.label("0.02");
                        ui.label(RichText::new("Selective high-speed trip").color(Theme::TEXT_MUTED));
                        ui.end_row();

                        ui.label(RichText::new("Long-Time Inverse (LI)").color(Theme::TEXT_WHITE));
                        ui.label("0.00262");
                        ui.label("0.00262");
                        ui.label("0.02");
                        ui.label(RichText::new("Equipment thermal protection").color(Theme::TEXT_MUTED));
                        ui.end_row();

                        ui.label(RichText::new("Ultra Inverse (UI)").color(Theme::TEXT_WHITE));
                        ui.label("8.9341");
                        ui.label("0.17966");
                        ui.label("2.00");
                        ui.label(RichText::new("Ultra-fast high magnitude trip").color(Theme::TEXT_MUTED));
                        ui.end_row();
                    });
            });
        });
    }
}
