#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use egui::{FontData, FontDefinitions, FontFamily};
use simple_logger::SimpleLogger;

mod model;
mod view;

fn configure_fonts(ctx: &egui::Context) {
    // 从文件系统加载字体数据
    let font_data = include_bytes!("../fonts/SourceHanSansSC-Normal.otf").to_vec();

    let mut fonts = FontDefinitions::default();

    // 注册字体数据
    fonts.font_data.insert(
        "my_chinese_font".to_owned(),
        std::sync::Arc::new(FontData::from_owned(font_data)),
    );

    // 将中文字体设为默认字体的首选
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "my_chinese_font".to_owned());

    // 同时添加到等宽字体家族（可选）
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "my_chinese_font".to_owned());

    ctx.set_fonts(fonts);
}

fn main() -> eframe::Result {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    eframe::run_native(
        "IAP Tool v0.0.1",
        options,
        Box::new(|ctx| {
            configure_fonts(&ctx.egui_ctx);
            Ok(Box::new(view::app_view::AppView::default()))
        }),
    )
}
