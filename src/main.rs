#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console window on Windows in release

use eframe::egui::{self, IconData, Vec2};
use relay_curve_estimator::RelayCurveApp;
use std::sync::Arc;

fn load_app_icon() -> Option<IconData> {
    let icon_bytes = include_bytes!("../assets/icon.png");
    if let Ok(img) = image::load_from_memory(icon_bytes) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Some(IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

fn main() -> eframe::Result<()> {
    let icon_data = load_app_icon();

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Relay Curve Estimator - Protection TCC Analysis")
        .with_inner_size(Vec2::new(1220.0, 840.0))
        .with_min_inner_size(Vec2::new(900.0, 620.0))
        .with_active(true)
        .with_drag_and_drop(true);

    if let Some(icon) = icon_data {
        viewport = viewport.with_icon(Arc::new(icon));
    }

    let native_options = eframe::NativeOptions {
        viewport,
        vsync: true,
        multisampling: 4,
        depth_buffer: 0,
        stencil_buffer: 0,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "Relay Curve Estimator",
        native_options,
        Box::new(|cc| Ok(Box::new(RelayCurveApp::new(cc)))),
    )
}
