use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, Stroke, Visuals};

pub struct Theme;

impl Theme {
    // Primary Accents
    pub const ACCENT_PRIMARY: Color32 = Color32::from_rgb(2, 132, 199);  // Sky 600 / Sapphire
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(56, 189, 248);    // Sky 400 (Fitted curve)
    pub const ACCENT_EMERALD: Color32 = Color32::from_rgb(16, 185, 129); // Emerald 500 (Test points & Optimal badge)
    pub const ACCENT_AMBER: Color32 = Color32::from_rgb(245, 158, 11);   // Amber 500 (Runner up)
    pub const ACCENT_ROSE: Color32 = Color32::from_rgb(244, 63, 94);     // Rose 500 (Residual errors)

    // Neutral Dark Engineering Palette
    pub const BG_APP: Color32 = Color32::from_rgb(15, 18, 25);          // Deep canvas
    pub const BG_PANEL: Color32 = Color32::from_rgb(20, 24, 34);        // Sidebar container
    pub const BG_CARD: Color32 = Color32::from_rgb(26, 32, 45);         // Card container
    pub const BG_CARD_ALT: Color32 = Color32::from_rgb(34, 42, 59);     // Hovered card
    pub const BG_INPUT: Color32 = Color32::from_rgb(13, 16, 23);        // Dark input fields
    pub const BG_PLOT: Color32 = Color32::from_rgb(12, 15, 22);         // Scope background

    // High-contrast Borders
    pub const BORDER_CARD: Color32 = Color32::from_rgb(45, 55, 78);      // Distinct border
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(35, 43, 62);    // Subtle divider
    pub const BORDER_INPUT: Color32 = Color32::from_rgb(55, 68, 96);     // Input border
    pub const BORDER_ACTIVE: Color32 = Color32::from_rgb(56, 189, 248);   // Cyan highlight

    // High-Contrast Typography
    pub const TEXT_WHITE: Color32 = Color32::from_rgb(255, 255, 255);    // 100% white
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(241, 245, 249);  // Slate 100
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(218, 228, 242);// Crisp high-contrast silver
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(148, 163, 184);    // Slate 400
    pub const TEXT_DIM: Color32 = Color32::from_rgb(100, 116, 139);      // Slate 500

    pub fn apply_custom_dark_visuals(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();

        visuals.dark_mode = true;
        visuals.panel_fill = Self::BG_PANEL;
        visuals.window_fill = Self::BG_APP;
        visuals.extreme_bg_color = Self::BG_INPUT;
        visuals.faint_bg_color = Self::BG_INPUT;
        visuals.code_bg_color = Self::BG_INPUT;
        visuals.override_text_color = Some(Self::TEXT_PRIMARY);

        // Non-interactive widgets
        visuals.widgets.noninteractive.bg_fill = Self::BG_CARD;
        visuals.widgets.noninteractive.weak_bg_fill = Self::BG_CARD;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Self::BORDER_CARD);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_PRIMARY);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);

        // Inactive widgets (buttons, dropdowns, slider tracks, text edits)
        visuals.widgets.inactive.bg_fill = Self::BG_INPUT;
        visuals.widgets.inactive.weak_bg_fill = Self::BG_INPUT;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Self::BORDER_INPUT);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_PRIMARY);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(6);

        // Hovered widgets
        visuals.widgets.hovered.bg_fill = Self::BG_CARD_ALT;
        visuals.widgets.hovered.weak_bg_fill = Self::BG_CARD_ALT;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Self::ACCENT_CYAN);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_WHITE);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(6);

        // Active widgets
        visuals.widgets.active.bg_fill = Self::ACCENT_PRIMARY;
        visuals.widgets.active.weak_bg_fill = Self::ACCENT_PRIMARY;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, Self::ACCENT_CYAN);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_WHITE);
        visuals.widgets.active.corner_radius = CornerRadius::same(6);

        // Open popups / menus
        visuals.widgets.open.bg_fill = Self::BG_CARD;
        visuals.widgets.open.weak_bg_fill = Self::BG_CARD;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, Self::ACCENT_CYAN);
        visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_WHITE);
        visuals.widgets.open.corner_radius = CornerRadius::same(6);

        // Slider styling
        visuals.slider_trailing_fill = true;

        // Window & menu styling
        visuals.window_corner_radius = CornerRadius::same(8);
        visuals.menu_corner_radius = CornerRadius::same(6);
        visuals.window_stroke = Stroke::new(1.0_f32, Self::BORDER_CARD);

        // Selection
        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(2, 132, 199, 120);
        visuals.selection.stroke = Stroke::new(1.0_f32, Self::ACCENT_CYAN);

        ctx.set_visuals(visuals);
    }

    pub fn card_frame() -> Frame {
        Frame::NONE
            .fill(Self::BG_CARD)
            .stroke(Stroke::new(1.0_f32, Self::BORDER_CARD))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(12))
    }

    pub fn card_frame_highlight() -> Frame {
        Frame::NONE
            .fill(Color32::from_rgb(18, 30, 48))
            .stroke(Stroke::new(1.0_f32, Self::BORDER_CARD))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(14))
    }

    pub fn card_frame_subtle() -> Frame {
        Frame::NONE
            .fill(Self::BG_PANEL)
            .stroke(Stroke::new(1.0_f32, Self::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::same(10))
    }

    pub fn plot_frame() -> Frame {
        Frame::NONE
            .fill(Self::BG_PLOT)
            .stroke(Stroke::new(1.0_f32, Self::BORDER_CARD))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::same(4))
    }
}
