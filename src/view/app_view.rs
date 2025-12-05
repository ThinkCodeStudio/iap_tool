use log::error;
use probe_rs::{Permissions, config::Registry, flashing::{FormatKind, download_file}, probe::{DebugProbeInfo, list::{self, Lister}}};

use crate::model::{AppDataStruct, Firmware};

pub struct IapParameter {
    pub series_name: String,
    pub product_name: String,
    pub firmware: Firmware,
}

impl Default for IapParameter {
    fn default() -> Self {
        Self {
            series_name: Default::default(),
            product_name: Default::default(),
            firmware: Default::default(),
        }
    }
}

pub struct AppView {
    app_data: AppDataStruct,
    iap_parameter: IapParameter,
    registry: Registry,
    admin_mode: bool,
    msg: String,
    probes: Vec<DebugProbeInfo>,
    probe_index: usize,
}

impl Default for AppView {
    fn default() -> Self {
        let app_data = match AppDataStruct::load_from_file("app_data.json") {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to load app data: {}", e);
                AppDataStruct::new()
            }
        };
        let lister = Lister::new();

        Self {
            app_data,
            iap_parameter: IapParameter::default(),
            registry: Registry::from_builtin_families(),
            msg: String::new(),
            admin_mode: false,
            probes: lister.list_all(),
            probe_index: 0,
        }
    }
}

impl AppView {}

impl eframe::App for AppView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);
        egui::CentralPanel::default().show(ctx, |_| {
            egui::TopBottomPanel::top("top").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("管理模式:");
                    if ui
                        .button(if self.admin_mode { "关闭" } else { "开启" })
                        .clicked()
                    {
                        self.admin_mode = !self.admin_mode;
                    }
                });
            });

            egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
                ui.label(&self.msg);
            });

            let windew_width = ctx.available_rect().width();
            egui::SidePanel::left("left")
                .min_width(windew_width / 3.0)
                .show(ctx, |ui| {
                    for series in &self.app_data.series {
                        egui::CollapsingHeader::new(series.name.clone()).show(ui, |ui| {
                            for product in &series.products {
                                egui::CollapsingHeader::new(product.name.clone()).show(ui, |ui| {
                                    for firmware in &product.firmware {
                                        if ui
                                            .label(format!(
                                                "{} - {} ({})",
                                                firmware.name, firmware.version, firmware.chip_type
                                            ))
                                            .clicked()
                                        {
                                            self.iap_parameter.series_name = series.name.clone();
                                            self.iap_parameter.product_name = product.name.clone();
                                            self.iap_parameter.firmware = firmware.clone();
                                            self.msg = format!(
                                                "Selected: {} / {} / {}-{}",
                                                series.name,
                                                product.name,
                                                firmware.name,
                                                firmware.version
                                            );
                                        }
                                    }
                                });
                            }
                        });
                    }
                });

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("欢迎使用 IAP Tool");
                ui.horizontal(|ui| {
                    ui.label("系列: ");
                    ui.add_enabled_ui(self.admin_mode, |ui| {
                        egui::TextEdit::singleline(&mut self.iap_parameter.series_name)
                            .hint_text("请输入系列名称")
                            .show(ui);
                    });
                });
                ui.horizontal(|ui| {
                    ui.label("产品: ");
                    ui.add_enabled_ui(self.admin_mode, |ui| {
                        egui::TextEdit::singleline(&mut self.iap_parameter.product_name)
                            .hint_text("请输入产品名称")
                            .show(ui);
                    });
                });
                ui.horizontal(|ui| {
                    ui.label("固件: ");
                    ui.add_enabled_ui(self.admin_mode, |ui| {
                        egui::TextEdit::singleline(&mut self.iap_parameter.firmware.name)
                            .hint_text("请输入固件名称")
                            .show(ui);
                    });
                });
                ui.horizontal(|ui| {
                    ui.label("版本: ");
                    ui.add_enabled_ui(self.admin_mode, |ui| {
                        egui::TextEdit::singleline(&mut self.iap_parameter.firmware.version)
                            .hint_text("请输入固件版本")
                            .show(ui);
                    });
                });
                ui.horizontal(|ui| {
                    ui.label("路径: ");
                    ui.add_enabled_ui(self.admin_mode, |ui| {
                        egui::TextEdit::singleline(&mut self.iap_parameter.firmware.fw_path)
                            .hint_text("请输入固件路径")
                            .show(ui);
                        if ui.button("浏览...").on_hover_text("选择固件文件").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("ELF", &["elf"])
                                .add_filter("HEX", &["hex"])
                                .add_filter("BIN", &["bin"])
                                .pick_file()
                            {
                                self.iap_parameter.firmware.fw_path =
                                    path.to_string_lossy().to_string();
                            }
                        }
                    });
                });
                ui.horizontal(|ui| {
                    ui.label("芯片类型: ");
                    ui.add_enabled_ui(self.admin_mode, |ui| {
                        egui::ComboBox::from_label("Series")
                            .selected_text(&self.iap_parameter.firmware.chip_series)
                            .show_ui(ui, |ui| {
                                for family in self.registry.families() {
                                    ui.selectable_value(
                                        &mut self.iap_parameter.firmware.chip_series,
                                        family.name.clone(),
                                        family.name.clone(),
                                    );
                                }
                            });
                        egui::ComboBox::from_label("Chip")
                            .selected_text(&self.iap_parameter.firmware.chip_type)
                            .show_ui(ui, |ui| {
                                for family in &self
                                    .registry
                                    .get_targets_by_family_name(
                                        &self.iap_parameter.firmware.chip_series,
                                    )
                                    .unwrap()
                                {
                                    ui.selectable_value(
                                        &mut self.iap_parameter.firmware.chip_type,
                                        family.clone(),
                                        family,
                                    );
                                }
                            });
                    });
                });

                ui.horizontal(|ui|{
                    if ui.button("🔄").clicked() {
                        let lister = Lister::new();
                        self.probes = lister.list_all();
                    }
                    ui.label("调试探针: ");
                    egui::ComboBox::from_label("Probe")
                        .selected_text(
                            if self.probes.is_empty() {
                                "".to_string()
                            }
                            else{
                                format!("[{}]{}", self.probe_index, &self.probes[self.probe_index].identifier)
                            }
                        )
                        .show_ui(ui, |ui| {
                            for (i, probe) in self.probes.iter().enumerate() {
                                ui.selectable_value(
                                    &mut self.probe_index,
                                    i,
                                    format!("[{}]{}-({:?})",i , probe.identifier, probe.serial_number.clone().unwrap_or("None".to_string())),
                                );
                            }
                        });
                });

                ui.horizontal(|ui| {
                    if self.admin_mode {
                        if ui.button("保存").clicked() {
                            // 保存逻辑
                            let series = self
                                .app_data
                                .series
                                .iter_mut()
                                .find(|s| s.name == self.iap_parameter.series_name);
                            let series = match series {
                                Some(s) => s,
                                None => {
                                    self.app_data.series.push(crate::model::Series {
                                        name: self.iap_parameter.series_name.clone(),
                                        products: Vec::new(),
                                    });
                                    self.app_data.series.last_mut().unwrap()
                                }
                            };

                            let product = series
                                .products
                                .iter_mut()
                                .find(|p| p.name == self.iap_parameter.product_name);
                            let product = match product {
                                Some(p) => p,
                                None => {
                                    series.products.push(crate::model::Product {
                                        name: self.iap_parameter.product_name.clone(),
                                        firmware: Vec::new(),
                                    });
                                    series.products.last_mut().unwrap()
                                }
                            };

                            let firmware = product.firmware.iter_mut().find(|f| {
                                f.name == self.iap_parameter.firmware.name
                                    && f.version == self.iap_parameter.firmware.version
                                    && f.chip_series == self.iap_parameter.firmware.chip_series
                                    && f.chip_type == self.iap_parameter.firmware.chip_type
                            });
                            match firmware {
                                Some(f) => {
                                    *f = self.iap_parameter.firmware.clone();
                                }
                                None => {
                                    product.firmware.push(self.iap_parameter.firmware.clone());
                                }
                            };

                            if let Err(e) = self.app_data.save_to_file("app_data.json") {
                                self.msg = format!("保存失败: {}", e);
                            } else {
                                self.msg = "保存成功".to_string();
                            }
                        }

                        if ui.button("删除").clicked() {
                            // 删除逻辑
                            if let Some(series) = self
                                .app_data
                                .series
                                .iter_mut()
                                .find(|s| s.name == self.iap_parameter.series_name)
                            {
                                if let Some(product) = series
                                    .products
                                    .iter_mut()
                                    .find(|p| p.name == self.iap_parameter.product_name)
                                {
                                    product
                                        .firmware
                                        .retain(|f| f.name != self.iap_parameter.firmware.name);
                                }
                                series.products.retain(|p| p.firmware.len() > 0);
                            }

                            self.app_data.series.retain(|s| s.products.len() > 0);

                            self.iap_parameter = IapParameter::default();

                            if let Err(e) = self.app_data.save_to_file("app_data.json") {
                                self.msg = format!("删除失败: {}", e);
                            } else {
                                self.msg = "删除成功".to_string();
                            }
                        }
                    }

                    if ui.button("下载").clicked() {
                        if let Ok(probe) = self.probes[0].open(){
                            if let Ok(session) = &mut probe.attach(self.iap_parameter.firmware.chip_type.as_str(), Permissions::default()) {
                                match download_file(session, self.iap_parameter.firmware.fw_path.clone(), FormatKind::Elf) {
                                    Ok(_) => {
                                        self.msg = "下载成功".to_string()
                                    },
                                    Err(_) => {
                                        self.msg = "下载失败".to_string()
                                    },
                                }
                            }
                            else{
                                self.msg = "无法附加到目标芯片".to_string();
                            }
                        }
                        else{
                            self.msg = "无法打开调试探针".to_string();
                        }
                    }
                });
            });
        });
    }
}
