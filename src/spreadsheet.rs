use crate::estimator::TestPoint;
use crate::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, Button, Color32, CornerRadius, FontId, Key, Rect, RichText, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCellPos {
    pub row: usize,
    pub col: usize, // 0 = Current (A), 1 = Time (s)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridRange {
    pub min_row: usize,
    pub max_row: usize,
    pub min_col: usize,
    pub max_col: usize,
}

impl GridRange {
    pub fn from_cells(a: GridCellPos, b: GridCellPos) -> Self {
        Self {
            min_row: a.row.min(b.row),
            max_row: a.row.max(b.row),
            min_col: a.col.min(b.col),
            max_col: a.col.max(b.col),
        }
    }

    pub fn contains(&self, pos: GridCellPos) -> bool {
        pos.row >= self.min_row
            && pos.row <= self.max_row
            && pos.col >= self.min_col
            && pos.col <= self.max_col
    }

    pub fn is_single_cell(&self) -> bool {
        self.min_row == self.max_row && self.min_col == self.max_col
    }
}

/// Helper to parse cleaned numeric values from pasted strings (handles units, commas, ms)
pub fn parse_cleaned_numeric(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let mut is_ms = false;
    let mut cleaned = String::new();

    for c in s.chars() {
        if c.is_ascii_digit() || c == '.' || c == '-' || c == '+' {
            cleaned.push(c);
        } else if c == ',' {
            cleaned.push('.');
        } else if c == 'm' || c == 'M' {
            is_ms = true;
        }
    }

    let mut val = cleaned.parse::<f64>().ok()?;
    if is_ms {
        val /= 1000.0;
    }
    Some(val)
}

#[derive(Debug, Clone)]
pub struct SpreadsheetGrid {
    pub active_cell: Option<GridCellPos>,
    pub anchor_cell: Option<GridCellPos>,
    pub selection_range: Option<GridRange>,
    pub is_editing: bool,
    pub is_dragging: bool,
    pub just_started_editing: bool,
}

impl Default for SpreadsheetGrid {
    fn default() -> Self {
        Self {
            active_cell: Some(GridCellPos { row: 0, col: 0 }),
            anchor_cell: Some(GridCellPos { row: 0, col: 0 }),
            selection_range: Some(GridRange {
                min_row: 0,
                max_row: 0,
                min_col: 0,
                max_col: 0,
            }),
            is_editing: false,
            is_dragging: false,
            just_started_editing: false,
        }
    }
}

impl SpreadsheetGrid {
    pub fn select_cell(&mut self, row: usize, col: usize) {
        let pos = GridCellPos { row, col: col.min(1) };
        self.active_cell = Some(pos);
        self.anchor_cell = Some(pos);
        self.selection_range = Some(GridRange::from_cells(pos, pos));
        self.is_editing = false;
        self.just_started_editing = false;
    }

    pub fn select_range(&mut self, a: GridCellPos, b: GridCellPos) {
        self.active_cell = Some(b);
        self.anchor_cell = Some(a);
        self.selection_range = Some(GridRange::from_cells(a, b));
        self.is_editing = false;
        self.just_started_editing = false;
    }

    pub fn select_all(&mut self, num_rows: usize) {
        if num_rows == 0 {
            return;
        }
        let a = GridCellPos { row: 0, col: 0 };
        let b = GridCellPos { row: num_rows - 1, col: 1 };
        self.select_range(a, b);
    }

    /// Copies selected range to system clipboard in standard TSV format
    pub fn copy_selection(&self, ctx: &egui::Context, points: &[TestPoint]) -> Option<String> {
        let range = self.selection_range.or_else(|| {
            self.active_cell.map(|c| GridRange::from_cells(c, c))
        })?;

        let mut lines = Vec::new();
        for r in range.min_row..=range.max_row.min(points.len().saturating_sub(1)) {
            if let Some(pt) = points.get(r) {
                let mut cols = Vec::new();
                for c in range.min_col..=range.max_col.min(1) {
                    if c == 0 {
                        cols.push(pt.current_str.clone());
                    } else {
                        cols.push(pt.time_str.clone());
                    }
                }
                lines.push(cols.join("\t"));
            }
        }

        if lines.is_empty() {
            return None;
        }

        let tsv = lines.join("\r\n");
        ctx.copy_text(tsv.clone());

        if let Ok(mut clip) = arboard::Clipboard::new() {
            let _ = clip.set_text(tsv.clone());
        }

        Some(tsv)
    }

    /// Pastes tabular data from clipboard into the spreadsheet starting at the active cell
    pub fn paste_clipboard(&mut self, ctx: &egui::Context, points: &mut Vec<TestPoint>) -> Option<usize> {
        let clipboard_text = ctx.input(|i| {
            i.events.iter().find_map(|e| {
                if let egui::Event::Paste(text) = e {
                    Some(text.clone())
                } else {
                    None
                }
            })
        });

        let text_to_parse = clipboard_text.or_else(|| {
            let mut clip = arboard::Clipboard::new().ok()?;
            clip.get_text().ok()
        })?;

        let start_pos = self.active_cell.unwrap_or(GridCellPos { row: 0, col: 0 });
        let mut parsed_matrix: Vec<Vec<f64>> = Vec::new();

        for line in text_to_parse.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = if line.contains('\t') {
                line.split('\t').filter(|s| !s.trim().is_empty()).collect()
            } else if line.contains(',') {
                line.split(',').filter(|s| !s.trim().is_empty()).collect()
            } else if line.contains(';') {
                line.split(';').filter(|s| !s.trim().is_empty()).collect()
            } else {
                line.split_whitespace().collect()
            };

            let mut row_vals = Vec::new();
            for p in parts {
                if let Some(val) = parse_cleaned_numeric(p) {
                    row_vals.push(val);
                }
            }

            if !row_vals.is_empty() {
                parsed_matrix.push(row_vals);
            }
        }

        if parsed_matrix.is_empty() {
            return None;
        }

        let pasted_rows = parsed_matrix.len();
        let max_target_row = start_pos.row + pasted_rows;

        // Auto-expand points if pasted data exceeds current length
        while points.len() < max_target_row {
            let next_curr = 2.0 * (points.len() as f64 + 1.0);
            points.push(TestPoint::with_label(next_curr, 1.0, format!("Point {}", points.len() + 1)));
        }

        let mut max_col_pasted = start_pos.col;

        for (r_offset, row_data) in parsed_matrix.iter().enumerate() {
            let target_r = start_pos.row + r_offset;
            if let Some(pt) = points.get_mut(target_r) {
                for (c_offset, &val) in row_data.iter().enumerate() {
                    let target_c = start_pos.col + c_offset;
                    if target_c == 0 {
                        pt.current = val;
                        pt.current_str = format!("{:.3}", val);
                        max_col_pasted = max_col_pasted.max(0);
                    } else if target_c == 1 {
                        pt.time = val;
                        pt.time_str = format!("{:.4}", val);
                        max_col_pasted = max_col_pasted.max(1);
                    }
                }
            }
        }

        // Set selection range to cover pasted rectangle
        let end_row = (start_pos.row + pasted_rows - 1).min(points.len().saturating_sub(1));
        self.select_range(
            start_pos,
            GridCellPos {
                row: end_row,
                col: max_col_pasted.min(1),
            },
        );

        Some(pasted_rows)
    }

    /// Clears cell contents inside the current selection range
    pub fn clear_selection(&mut self, points: &mut [TestPoint]) {
        let range = match self.selection_range.or_else(|| self.active_cell.map(|c| GridRange::from_cells(c, c))) {
            Some(r) => r,
            None => return,
        };

        for r in range.min_row..=range.max_row.min(points.len().saturating_sub(1)) {
            if let Some(pt) = points.get_mut(r) {
                for c in range.min_col..=range.max_col.min(1) {
                    if c == 0 {
                        pt.current = 0.0;
                        pt.current_str.clear();
                    } else if c == 1 {
                        pt.time = 0.0;
                        pt.time_str.clear();
                    }
                }
            }
        }
    }

    /// Main interactive spreadsheet grid renderer
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        points: &mut Vec<TestPoint>,
        pickup_current: f64,
        needs_recalc: &mut bool,
        toast: &mut Option<(String, std::time::Instant)>,
    ) {
        let num_rows = points.len();
        if num_rows == 0 {
            points.push(TestPoint::with_label(pickup_current * 2.0, 10.0, "Point 1"));
        }

        let mut to_add_row: Option<GridCellPos> = None;

        // 1. Process Global Spreadsheet Keyboard Events (when in cell navigation mode)
        if !self.is_editing {
            let shift = ui.input(|i| i.modifiers.shift);
            let ctrl = ui.input(|i| i.modifiers.command);
            let active = self.active_cell.unwrap_or(GridCellPos { row: 0, col: 0 });

            // Ctrl+A -> Select All
            if ctrl && ui.input(|i| i.key_pressed(Key::A)) {
                self.select_all(num_rows);
            }

            // Ctrl+C -> Copy Selection
            if ctrl && ui.input(|i| i.key_pressed(Key::C)) {
                if let Some(_) = self.copy_selection(ui.ctx(), points) {
                    *toast = Some(("Copied selection to clipboard".to_string(), std::time::Instant::now()));
                }
            }

            // Ctrl+X -> Cut Selection
            if ctrl && ui.input(|i| i.key_pressed(Key::X)) {
                if let Some(_) = self.copy_selection(ui.ctx(), points) {
                    self.clear_selection(points);
                    *needs_recalc = true;
                    *toast = Some(("Cut selection to clipboard".to_string(), std::time::Instant::now()));
                }
            }

            // Ctrl+V -> Paste
            if ctrl && ui.input(|i| i.key_pressed(Key::V)) {
                if let Some(pasted_count) = self.paste_clipboard(ui.ctx(), points) {
                    *needs_recalc = true;
                    *toast = Some((format!("Pasted {pasted_count} rows from clipboard"), std::time::Instant::now()));
                }
            }

            // Delete or Backspace
            if ui.input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace)) {
                self.clear_selection(points);
                *needs_recalc = true;
            }

            // F2 -> Enter Edit Mode
            if ui.input(|i| i.key_pressed(Key::F2)) {
                self.is_editing = true;
                self.just_started_editing = true;
            }

            // Arrow Keys Navigation
            if ui.input(|i| i.key_pressed(Key::ArrowUp)) && active.row > 0 {
                let next = GridCellPos { row: active.row - 1, col: active.col };
                if shift {
                    let anchor = self.anchor_cell.unwrap_or(active);
                    self.select_range(anchor, next);
                } else {
                    self.select_cell(next.row, next.col);
                }
            } else if ui.input(|i| i.key_pressed(Key::ArrowDown)) && active.row + 1 < num_rows {
                let next = GridCellPos { row: active.row + 1, col: active.col };
                if shift {
                    let anchor = self.anchor_cell.unwrap_or(active);
                    self.select_range(anchor, next);
                } else {
                    self.select_cell(next.row, next.col);
                }
            } else if ui.input(|i| i.key_pressed(Key::ArrowLeft)) && active.col > 0 {
                let next = GridCellPos { row: active.row, col: 0 };
                if shift {
                    let anchor = self.anchor_cell.unwrap_or(active);
                    self.select_range(anchor, next);
                } else {
                    self.select_cell(next.row, next.col);
                }
            } else if ui.input(|i| i.key_pressed(Key::ArrowRight)) && active.col < 1 {
                let next = GridCellPos { row: active.row, col: 1 };
                if shift {
                    let anchor = self.anchor_cell.unwrap_or(active);
                    self.select_range(anchor, next);
                } else {
                    self.select_cell(next.row, next.col);
                }
            } else if ui.input(|i| i.key_pressed(Key::Tab)) {
                if shift {
                    if active.col > 0 {
                        self.select_cell(active.row, 0);
                    } else if active.row > 0 {
                        self.select_cell(active.row - 1, 1);
                    }
                } else {
                    if active.col == 0 {
                        self.select_cell(active.row, 1);
                    } else if active.row + 1 < num_rows {
                        self.select_cell(active.row + 1, 0);
                    } else {
                        // Add new row on Tab at table end
                        to_add_row = Some(GridCellPos { row: num_rows, col: 0 });
                    }
                }
            } else if ui.input(|i| i.key_pressed(Key::Enter)) {
                if shift {
                    if active.row > 0 {
                        self.select_cell(active.row - 1, active.col);
                    }
                } else {
                    if active.row + 1 < num_rows {
                        self.select_cell(active.row + 1, active.col);
                    } else {
                        // Add new row on Enter at bottom
                        to_add_row = Some(GridCellPos { row: num_rows, col: active.col });
                    }
                }
            }

            // Direct digit typing starts in-cell edit mode (just like Excel!)
            if !ctrl {
                let typed_char = ui.input(|i| {
                    i.events.iter().find_map(|e| {
                        if let egui::Event::Text(t) = e {
                            if t.len() == 1 {
                                let c = t.chars().next().unwrap();
                                if c.is_ascii_digit() || c == '.' || c == '-' || c == '+' {
                                    return Some(c);
                                }
                            }
                        }
                        None
                    })
                });

                if let Some(c) = typed_char {
                    if let Some(act) = self.active_cell {
                        if let Some(pt) = points.get_mut(act.row) {
                            if act.col == 0 {
                                pt.current_str = c.to_string();
                            } else {
                                pt.time_str = c.to_string();
                            }
                            self.is_editing = true;
                            self.just_started_editing = true;
                        }
                    }
                }
            }
        }

        // 2. Render Spreadsheet Container Geometry
        let avail_w = ui.available_width();
        let row_header_w = 32.0;
        let delete_col_w = 26.0;
        let col_w = ((avail_w - row_header_w - delete_col_w) / 2.0).max(105.0);
        let row_h = 25.0;

        let mut to_remove_row = None;

        // Render Table Header
        let (header_rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, 26.0), egui::Sense::hover());
        if ui.is_rect_visible(header_rect) {
            let p = ui.painter();
            p.rect_filled(header_rect, CornerRadius::ZERO, Color32::from_rgb(26, 34, 48));

            let x0 = header_rect.left();
            let x1 = x0 + row_header_w;
            let x2 = x1 + col_w;
            let x3 = x2 + col_w;

            let border_stroke = Stroke::new(1.0_f32, Color32::from_rgb(45, 58, 82));
            p.line_segment([header_rect.left_bottom(), header_rect.right_bottom()], Stroke::new(1.5_f32, Theme::ACCENT_CYAN));
            p.line_segment([pos2(x1, header_rect.top()), pos2(x1, header_rect.bottom())], border_stroke);
            p.line_segment([pos2(x2, header_rect.top()), pos2(x2, header_rect.bottom())], border_stroke);
            p.line_segment([pos2(x3, header_rect.top()), pos2(x3, header_rect.bottom())], border_stroke);

            p.text(pos2((x0 + x1) * 0.5, header_rect.center().y), Align2::CENTER_CENTER, "#", FontId::proportional(11.0), Color32::from_rgb(148, 163, 184));
            p.text(pos2(x1 + 8.0, header_rect.center().y), Align2::LEFT_CENTER, "CURRENT (A)", FontId::proportional(11.0), Color32::from_rgb(226, 232, 240));
            p.text(pos2(x2 + 8.0, header_rect.center().y), Align2::LEFT_CENTER, "TIME (s)", FontId::proportional(11.0), Color32::from_rgb(226, 232, 240));
        }

        // Pointer state for click-and-drag range selection
        let pointer_pos = ui.input(|i| i.pointer.interact_pos());
        let pointer_down = ui.input(|i| i.pointer.primary_down());

        if !pointer_down {
            self.is_dragging = false;
        }

        let current_range = self.selection_range.unwrap_or(GridRange { min_row: 0, max_row: 0, min_col: 0, max_col: 0 });
        let mut cell_rects: Vec<((usize, usize), Rect)> = Vec::new();

        let total_pts = points.len();

        for (row_idx, point) in points.iter_mut().enumerate() {
            let (row_rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, row_h), egui::Sense::hover());
            if !ui.is_rect_visible(row_rect) {
                continue;
            }

            let bg = if row_idx % 2 == 0 {
                Color32::from_rgb(13, 17, 24)
            } else {
                Color32::from_rgb(17, 22, 32)
            };

            ui.painter().rect_filled(row_rect, CornerRadius::ZERO, bg);

            let x0 = row_rect.left();
            let x1 = x0 + row_header_w;
            let x2 = x1 + col_w;
            let x3 = x2 + col_w;
            let x4 = row_rect.right();

            let border_stroke = Stroke::new(1.0_f32, Color32::from_rgb(38, 48, 68));

            // Grid lines
            ui.painter().line_segment([row_rect.left_bottom(), row_rect.right_bottom()], border_stroke);
            ui.painter().line_segment([pos2(x1, row_rect.top()), pos2(x1, row_rect.bottom())], border_stroke);
            ui.painter().line_segment([pos2(x2, row_rect.top()), pos2(x2, row_rect.bottom())], border_stroke);
            ui.painter().line_segment([pos2(x3, row_rect.top()), pos2(x3, row_rect.bottom())], border_stroke);

            // Row Header (#)
            let header_cell_rect = Rect::from_min_max(pos2(x0, row_rect.top()), pos2(x1, row_rect.bottom()));
            let is_row_selected = current_range.min_row <= row_idx && row_idx <= current_range.max_row;

            let row_header_bg = if is_row_selected {
                Color32::from_rgb(30, 41, 59)
            } else {
                Color32::from_rgb(22, 28, 40)
            };
            ui.painter().rect_filled(header_cell_rect, CornerRadius::ZERO, row_header_bg);
            ui.painter().text(
                header_cell_rect.center(),
                Align2::CENTER_CENTER,
                format!("{}", row_idx + 1),
                FontId::monospace(11.0),
                if is_row_selected { Theme::ACCENT_CYAN } else { Color32::from_rgb(148, 163, 184) },
            );

            // Row header click -> select row
            let resp_row_hdr = ui.interact(header_cell_rect, ui.id().with(("row_hdr", row_idx)), egui::Sense::click());
            if resp_row_hdr.clicked() {
                self.select_range(GridCellPos { row: row_idx, col: 0 }, GridCellPos { row: row_idx, col: 1 });
            }

            // Cell (row_idx, 0) - Current (A)
            let cell_0_rect = Rect::from_min_max(pos2(x1 + 1.0, row_rect.top() + 1.0), pos2(x2 - 1.0, row_rect.bottom() - 1.0));
            cell_rects.push(((row_idx, 0), cell_0_rect));
            let cell_0_pos = GridCellPos { row: row_idx, col: 0 };

            // Cell (row_idx, 1) - Time (s)
            let cell_1_rect = Rect::from_min_max(pos2(x2 + 1.0, row_rect.top() + 1.0), pos2(x3 - 1.0, row_rect.bottom() - 1.0));
            cell_rects.push(((row_idx, 1), cell_1_rect));
            let cell_1_pos = GridCellPos { row: row_idx, col: 1 };

            // -------------------------------------------------------------
            // Cell 0 Interaction & Render (Current)
            // -------------------------------------------------------------
            let is_cell_0_active = self.active_cell == Some(cell_0_pos);
            let is_cell_0_editing = self.is_editing && is_cell_0_active;

            if current_range.contains(cell_0_pos) {
                ui.painter().rect_filled(cell_0_rect, CornerRadius::ZERO, Color32::from_rgba_unmultiplied(56, 189, 248, 35));
            }

            if is_cell_0_editing {
                let edit_resp = ui.put(
                    cell_0_rect,
                    egui::TextEdit::singleline(&mut point.current_str)
                        .frame(false)
                        .font(FontId::monospace(13.0))
                        .text_color(Theme::TEXT_WHITE)
                        .margin(egui::Margin::symmetric(6, 2)),
                );

                if self.just_started_editing {
                    edit_resp.request_focus();
                    self.just_started_editing = false;
                }

                if edit_resp.changed() {
                    if let Ok(v) = point.current_str.trim().parse::<f64>() {
                        point.current = v;
                        *needs_recalc = true;
                    }
                }

                // Handle Exit from In-Cell Edit Mode
                if ui.input(|i| i.key_pressed(Key::Enter)) {
                    self.is_editing = false;
                    if row_idx + 1 < num_rows {
                        self.select_cell(row_idx + 1, 0);
                    } else {
                        to_add_row = Some(GridCellPos { row: num_rows, col: 0 });
                    }
                } else if ui.input(|i| i.key_pressed(Key::Tab)) {
                    self.is_editing = false;
                    self.select_cell(row_idx, 1);
                } else if ui.input(|i| i.key_pressed(Key::Escape)) {
                    self.is_editing = false;
                }
            } else {
                // Interactive Cell View Mode
                let resp_0 = ui.interact(cell_0_rect, ui.id().with(("cell_view", row_idx, 0)), egui::Sense::click_and_drag());

                if resp_0.double_clicked() {
                    self.select_cell(row_idx, 0);
                    self.is_editing = true;
                    self.just_started_editing = true;
                } else if resp_0.clicked() {
                    if ui.input(|i| i.modifiers.shift) {
                        let anchor = self.anchor_cell.unwrap_or(cell_0_pos);
                        self.select_range(anchor, cell_0_pos);
                    } else {
                        self.select_cell(row_idx, 0);
                    }
                } else if resp_0.drag_started() {
                    self.anchor_cell = Some(cell_0_pos);
                    self.select_cell(row_idx, 0);
                    self.is_dragging = true;
                }

                // Hover tracking during drag selection
                if self.is_dragging {
                    if let Some(pos) = pointer_pos {
                        if cell_0_rect.contains(pos) {
                            if let Some(anchor) = self.anchor_cell {
                                self.select_range(anchor, cell_0_pos);
                            }
                        }
                    }
                }

                // Paint cell text
                ui.painter().text(
                    pos2(cell_0_rect.left() + 6.0, cell_0_rect.center().y),
                    Align2::LEFT_CENTER,
                    &point.current_str,
                    FontId::monospace(13.0),
                    Theme::TEXT_WHITE,
                );
            }

            // -------------------------------------------------------------
            // Cell 1 Interaction & Render (Time)
            // -------------------------------------------------------------
            let is_cell_1_active = self.active_cell == Some(cell_1_pos);
            let is_cell_1_editing = self.is_editing && is_cell_1_active;

            if current_range.contains(cell_1_pos) {
                ui.painter().rect_filled(cell_1_rect, CornerRadius::ZERO, Color32::from_rgba_unmultiplied(56, 189, 248, 35));
            }

            if is_cell_1_editing {
                let edit_resp = ui.put(
                    cell_1_rect,
                    egui::TextEdit::singleline(&mut point.time_str)
                        .frame(false)
                        .font(FontId::monospace(13.0))
                        .text_color(Theme::TEXT_WHITE)
                        .margin(egui::Margin::symmetric(6, 2)),
                );

                if self.just_started_editing {
                    edit_resp.request_focus();
                    self.just_started_editing = false;
                }

                if edit_resp.changed() {
                    if let Ok(v) = point.time_str.trim().parse::<f64>() {
                        point.time = v;
                        *needs_recalc = true;
                    }
                }

                if ui.input(|i| i.key_pressed(Key::Enter)) {
                    self.is_editing = false;
                    if row_idx + 1 < num_rows {
                        self.select_cell(row_idx + 1, 1);
                    } else {
                        to_add_row = Some(GridCellPos { row: num_rows, col: 1 });
                    }
                } else if ui.input(|i| i.key_pressed(Key::Tab)) {
                    self.is_editing = false;
                    if row_idx + 1 < num_rows {
                        self.select_cell(row_idx + 1, 0);
                    } else {
                        to_add_row = Some(GridCellPos { row: num_rows, col: 0 });
                    }
                } else if ui.input(|i| i.key_pressed(Key::Escape)) {
                    self.is_editing = false;
                }
            } else {
                let resp_1 = ui.interact(cell_1_rect, ui.id().with(("cell_view", row_idx, 1)), egui::Sense::click_and_drag());

                if resp_1.double_clicked() {
                    self.select_cell(row_idx, 1);
                    self.is_editing = true;
                    self.just_started_editing = true;
                } else if resp_1.clicked() {
                    if ui.input(|i| i.modifiers.shift) {
                        let anchor = self.anchor_cell.unwrap_or(cell_1_pos);
                        self.select_range(anchor, cell_1_pos);
                    } else {
                        self.select_cell(row_idx, 1);
                    }
                } else if resp_1.drag_started() {
                    self.anchor_cell = Some(cell_1_pos);
                    self.select_cell(row_idx, 1);
                    self.is_dragging = true;
                }

                if self.is_dragging {
                    if let Some(pos) = pointer_pos {
                        if cell_1_rect.contains(pos) {
                            if let Some(anchor) = self.anchor_cell {
                                self.select_range(anchor, cell_1_pos);
                            }
                        }
                    }
                }

                ui.painter().text(
                    pos2(cell_1_rect.left() + 6.0, cell_1_rect.center().y),
                    Align2::LEFT_CENTER,
                    &point.time_str,
                    FontId::monospace(13.0),
                    Theme::TEXT_WHITE,
                );
            }

            // Delete cell (X)
            if total_pts > 1 {
                let del_rect = Rect::from_min_max(pos2(x3, row_rect.top()), pos2(x4, row_rect.bottom()));
                let btn = Button::new(RichText::new("X").size(11.0).color(Color32::from_rgb(148, 163, 184)))
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::NONE);
                if ui.put(del_rect, btn).on_hover_text("Delete row").clicked() {
                    to_remove_row = Some(row_idx);
                }
            }
        }

        // 3. Draw Outer Excel Selection Perimeter Marquee
        if let Some(range) = self.selection_range {
            let top_left_rect = cell_rects.iter().find(|(pos, _)| pos.0 == range.min_row && pos.1 == range.min_col).map(|(_, r)| *r);
            let btm_right_rect = cell_rects.iter().find(|(pos, _)| pos.0 == range.max_row && pos.1 == range.max_col).map(|(_, r)| *r);

            if let (Some(tl), Some(br)) = (top_left_rect, btm_right_rect) {
                let selection_rect = Rect::from_min_max(tl.min, br.max);
                ui.painter().rect_stroke(
                    selection_rect,
                    CornerRadius::ZERO,
                    Stroke::new(1.5_f32, Theme::ACCENT_CYAN),
                    egui::StrokeKind::Inside,
                );

                // Draw tiny active bottom-right handle square (signature Excel fill handle)
                let handle_rect = Rect::from_center_size(selection_rect.right_bottom(), vec2(5.0, 5.0));
                ui.painter().rect_filled(handle_rect, CornerRadius::ZERO, Theme::ACCENT_CYAN);
            }
        }

        // Handle deferred row addition
        if let Some(target) = to_add_row {
            let next_curr = pickup_current * (points.len() as f64 + 2.0);
            points.push(TestPoint::with_label(next_curr, 1.0, format!("Point {}", points.len() + 1)));
            self.select_cell(target.row, target.col);
            *needs_recalc = true;
        }

        if let Some(del_idx) = to_remove_row {
            points.remove(del_idx);
            *needs_recalc = true;
            if let Some(act) = self.active_cell {
                if act.row >= points.len() {
                    self.select_cell(points.len().saturating_sub(1), act.col);
                }
            }
        }

        ui.add_space(8.0);

        // 4. Spreadsheet Bottom Bar
        ui.horizontal(|ui| {
            if ui.add(
                Button::new(RichText::new("+ Add Row").color(Theme::ACCENT_CYAN).size(11.0))
                    .fill(Theme::BG_CARD_ALT)
                    .stroke(Stroke::new(1.0_f32, Theme::BORDER_CARD))
                    .corner_radius(CornerRadius::same(4)),
            ).clicked() {
                let next_curr = pickup_current * (points.len() as f64 + 2.0);
                points.push(TestPoint::with_label(next_curr, 1.0, format!("Point {}", points.len() + 1)));
                self.select_cell(points.len() - 1, 0);
                *needs_recalc = true;
            }

            if ui.add(
                Button::new(RichText::new("Clear Table").color(Theme::TEXT_MUTED).size(11.0))
                    .fill(Theme::BG_CARD_ALT)
                    .stroke(Stroke::new(1.0_f32, Theme::BORDER_CARD))
                    .corner_radius(CornerRadius::same(4)),
            ).clicked() {
                points.clear();
                points.push(TestPoint::with_label(pickup_current * 2.0, 10.0, "Point 1"));
                self.select_cell(0, 0);
                *needs_recalc = true;
            }

            ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                ui.label(RichText::new("Drag / Arrows / Enter / Ctrl+C/V").size(10.0).color(Theme::TEXT_MUTED));
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cleaned_numeric_formats() {
        assert_eq!(parse_cleaned_numeric("2.5"), Some(2.5));
        assert_eq!(parse_cleaned_numeric("2,5"), Some(2.5));
        assert_eq!(parse_cleaned_numeric("10.0 A"), Some(10.0));
        assert_eq!(parse_cleaned_numeric("26.6667 s"), Some(26.6667));
        assert_eq!(parse_cleaned_numeric("500 ms"), Some(0.5));
        assert_eq!(parse_cleaned_numeric("Current (A)"), None);
    }

    #[test]
    fn test_grid_range_from_cells() {
        let a = GridCellPos { row: 3, col: 1 };
        let b = GridCellPos { row: 1, col: 0 };
        let r = GridRange::from_cells(a, b);
        assert_eq!(r.min_row, 1);
        assert_eq!(r.max_row, 3);
        assert_eq!(r.min_col, 0);
        assert_eq!(r.max_col, 1);
        assert!(r.contains(GridCellPos { row: 2, col: 0 }));
        assert!(!r.contains(GridCellPos { row: 4, col: 0 }));
    }
}
