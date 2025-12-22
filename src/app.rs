use eframe::egui;
use obfstr::obfstr;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Receiver, Sender};
use crate::config::{AppConfig, ConfigManager};
use crate::printing;
use crate::capture;
use crate::ocr::OcrEngine;

use crate::updater;

pub enum AppMessage {
    Log(String),
    TextDetected(String),
    Status(bool),
}

pub struct PvzApp {
    config_manager: ConfigManager,
    config: AppConfig,
    
    // UI State
    detected_text: String,
    logs: String, 
    is_running: bool,
    
    // Threads & Communication
    stop_signal: Arc<Mutex<bool>>, 
    rx: Receiver<AppMessage>,
    tx: Sender<AppMessage>,
    
    // Printer State
    printers: Vec<String>,
    
    // Overlay State
    is_selecting_region: bool,
    selection_bg: Option<egui::TextureHandle>,
    selection_start: Option<egui::Pos2>,
    selection_curr: Option<egui::Pos2>,
}

impl PvzApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Apply modern visuals
        let mut socials = egui::Visuals::dark();
        socials.window_rounding = 12.0.into();
        socials.widgets.active.rounding = 8.0.into();
        socials.widgets.hovered.rounding = 8.0.into();
        socials.widgets.inactive.rounding = 8.0.into();
        _cc.egui_ctx.set_visuals(socials);

        let cm = ConfigManager::new();
        let config = cm.load();
        let (tx, rx) = channel();
        let tx_clone = tx.clone();
        
        // Spawn update check
        thread::spawn(move || {
            // Use obfuscated string directly in the call
            if let Ok(info) = updater::check_for_updates(obfstr!("https://pvz.wax1nay.ru/version.json")) {
                if info.current_version != info.latest_version {
                    if let Some(url) = info.download_url {
                        tx_clone.send(AppMessage::Log(format!("🚀 Найдено обязательное обновление: {} -> {}. Установка...", info.current_version, info.latest_version))).ok();
                        match updater::download_and_replace(&url) {
                            Ok(_) => {
                                thread::sleep(Duration::from_secs(1));
                                updater::restart_app();
                            }
                            Err(e) => {
                                tx_clone.send(AppMessage::Log(format!("❌ Ошибка автообновления: {}", e))).ok();
                            }
                        }
                    }
                }
            }
        });

        let printers = printing::get_printers();
        
        Self {
            config_manager: cm,
            config,
            detected_text: "---".to_string(),
            logs: String::new(),
            is_running: false,
            stop_signal: Arc::new(Mutex::new(false)),
            rx,
            tx,
            printers,
            is_selecting_region: false,
            selection_bg: None,
            selection_start: None,
            selection_curr: None,
        }
    }
    
    fn log(&mut self, msg: &str) {
        let time = chrono::Local::now().format("%H:%M:%S");
        self.logs.push_str(&format!("[{}] {}\n", time, msg));
    }

    fn toggle_monitoring(&mut self, ctx: &egui::Context) {
        if self.is_running {
            self.is_running = false;
            if let Ok(mut stop) = self.stop_signal.lock() {
                *stop = true;
            }
            self.log("⏹ Мониторинг остановлен");
        } else {
            if self.config.region.is_none() {
                self.log("⚠ Ошибка: Сначала выберите область!");
                return;
            }
            
            self.is_running = true;
            let stop_signal = self.stop_signal.clone();
            {
                let mut stop = stop_signal.lock().unwrap();
                *stop = false;
            }
            
            let config = self.config.clone();
            let tx = self.tx.clone();
            let ctx_clone = ctx.clone();

            self.log("▶ Мониторинг запущен...");

            thread::spawn(move || {
                let ocr = OcrEngine::new();
                let mut last_processed_text = String::new();
                let mut candidate_text = String::new();
                let mut candidate_count = 0;
                
                loop {
                    if let Ok(stop) = stop_signal.lock() {
                        if *stop { break; }
                    }
                    
                    if let Some(r) = config.region {
                        if let Some(img) = capture::capture_region(r[0], r[1], r[2], r[3]) {
                            let text = ocr.process_image(img, config.debug_mode);
                            
                            if !text.is_empty() {
                                // Extract only digits
                                let digits: String = text.chars().filter(|c| c.is_digit(10)).collect();
                                
                                if !digits.is_empty() {
                                    // Log for visibility
                                    if digits == text {
                                        tx.send(AppMessage::Log(format!("👁 Вижу: {}", text))).ok();
                                    } else {
                                        tx.send(AppMessage::Log(format!("👁 Вижу: {} -> (цифры: {})", text, digits))).ok();
                                    }

                                    if digits == candidate_text {
                                        candidate_count += 1;
                                    } else {
                                        candidate_text = digits.clone();
                                        candidate_count = 1;
                                    }

                                    // If text is stable for 2 consecutive frames
                                    if candidate_count >= 2 && digits != last_processed_text {
                                        last_processed_text = digits.clone();
                                        tx.send(AppMessage::TextDetected(digits.clone())).ok();
                                        tx.send(AppMessage::Log(format!("✅ Стабильно: {}", digits))).ok();
                                        
                                        if config.print_enabled {
                                            if let Err(e) = printing::print_label(&config.printer_name, &digits, config.label_width_mm, config.label_height_mm, config.print_orientation) {
                                                tx.send(AppMessage::Log(format!("❌ Ошибка печати: {}", e))).ok();
                                            } else {
                                                tx.send(AppMessage::Log(format!("🖨 Напечатано: {}", digits))).ok();
                                            }
                                        } else {
                                            tx.send(AppMessage::Log(format!("ℹ Пропуск печати (отключено): {}", digits))).ok();
                                        }
                                    }
                                } else {
                                    // Log noise if it doesn't contain digits
                                    tx.send(AppMessage::Log(format!("☁ Пропуск (нет цифр): {}", text))).ok();
                                    candidate_count = 0;
                                    candidate_text.clear();
                                }
                            }
                        }
                    }
                    
                    ctx_clone.request_repaint();
                    thread::sleep(Duration::from_millis(200));
                }
                tx.send(AppMessage::Status(false)).ok();
                ctx_clone.request_repaint();
            });
        }
    }
}

impl eframe::App for PvzApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppMessage::Log(s) => self.log(&s),
                AppMessage::TextDetected(s) => self.detected_text = s,
                AppMessage::Status(running) => self.is_running = running,
            }
        }

        // --- OVERLAY LAYER ---
        if self.is_selecting_region {
             if self.selection_bg.is_none() {
                 if let Some(img) = capture::capture_fullscreen() {
                     let size = [img.width() as usize, img.height() as usize];
                     let pixels = img.as_flat_samples();
                     let color_img = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                     self.selection_bg = Some(ctx.load_texture("selection_bg", color_img, Default::default()));
                 }
                 ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                 ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
             }

             egui::Area::new(egui::Id::new("overlay"))
                .fixed_pos(egui::pos2(0.0, 0.0))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let screen_rect = ui.input(|i| i.screen_rect());
                    
                    // Draw background screenshot
                    if let Some(bg) = &self.selection_bg {
                        ui.painter().image(bg.id(), screen_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                    }
                    
                    // Dimming
                    ui.painter().rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(100));
                    
                    if ui.input(|i| i.pointer.primary_pressed()) {
                        self.selection_start = ui.input(|i| i.pointer.interact_pos());
                    }
                    if ui.input(|i| i.pointer.primary_down()) {
                         self.selection_curr = ui.input(|i| i.pointer.interact_pos());
                    }
                    if ui.input(|i| i.pointer.primary_released()) {
                        if let (Some(start), Some(curr)) = (self.selection_start, self.selection_curr) {
                            let rect = egui::Rect::from_two_pos(start, curr);
                            let ppp = ctx.pixels_per_point();
                            self.config.region = Some([
                                (rect.min.x * ppp) as i32, (rect.min.y * ppp) as i32,
                                (rect.width() * ppp) as i32, (rect.height() * ppp) as i32,
                            ]);
                            self.config_manager.save(&self.config).ok();
                            self.log("✅ Область сохранена");
                        }
                        self.is_selecting_region = false;
                        self.selection_bg = None;
                        self.selection_start = None;
                        self.selection_curr = None;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
                    }
                    
                    if let (Some(start), Some(curr)) = (self.selection_start, self.selection_curr) {
                         ui.painter().rect_stroke(egui::Rect::from_two_pos(start, curr), 0.0, egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE));
                         ui.painter().rect_filled(egui::Rect::from_two_pos(start, curr), 0.0, egui::Color32::from_rgba_unmultiplied(0, 0, 255, 30));
                    }

                    ui.centered_and_justified(|ui| {
                        if self.selection_start.is_none() {
                            ui.label(egui::RichText::new("ВЫДЕЛИТЕ ОБЛАСТЬ").size(16.0).strong().color(egui::Color32::WHITE).raised());
                        }
                    });
                });
             return;
        }

        // --- MAIN UI ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading(egui::RichText::new("📦 ПВЗ Помощник").size(28.0).strong());
                ui.add_space(10.0);
            });

            ui.separator();

            ui.columns(2, |cols| {
                // LEFT COLUMN: SETTINGS
                cols[0].vertical(|ui| {
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.label(egui::RichText::new("⚙ Настройки").strong());
                        ui.add_space(5.0);

                        ui.label("Принтер:");
                        egui::ComboBox::from_id_salt("printer_cb")
                            .width(ui.available_width() - 10.0)
                            .selected_text(&self.config.printer_name)
                            .show_ui(ui, |ui| {
                                for printer in &self.printers {
                                    ui.selectable_value(&mut self.config.printer_name, printer.clone(), printer);
                                }
                            });
                        
                        if ui.button("🔄 Обновить список").clicked() {
                            self.printers = printing::get_printers();
                        }

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label("Этикетка (мм):");
                                ui.horizontal(|ui| {
                                    ui.add(egui::DragValue::new(&mut self.config.label_width_mm).speed(0.1).suffix(" w"));
                                    ui.add(egui::DragValue::new(&mut self.config.label_height_mm).speed(0.1).suffix(" h"));
                                });
                            });
                            
                            ui.add_space(10.0);
                            
                            ui.vertical(|ui| {
                                ui.label("Ориентация:");
                                let orientation_text = match self.config.print_orientation {
                                    0 => "Книжная",
                                    1 => "Альбомная",
                                    2 => "Книжная 180°",
                                    3 => "Альбомная 180°",
                                    _ => "Книжная",
                                };
                                egui::ComboBox::from_id_salt("orientation_cb")
                                    .selected_text(orientation_text)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.config.print_orientation, 0, "Книжная");
                                        ui.selectable_value(&mut self.config.print_orientation, 1, "Альбомная");
                                        ui.selectable_value(&mut self.config.print_orientation, 2, "Книжная 180°");
                                        ui.selectable_value(&mut self.config.print_orientation, 3, "Альбомная 180°");
                                    });
                            });
                        });

                        ui.add_space(10.0);
                        if ui.add_sized([ui.available_width(), 30.0], egui::Button::new("💾 Сохранить")).clicked() {
                            self.config_manager.save(&self.config).ok();
                            self.log("💾 Настройки сохранены");
                        }
                    });

                    ui.add_space(10.0);
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.label(egui::RichText::new("📷 Область экрана").strong());
                        ui.label(format!("Координаты: {:?}", self.config.region.unwrap_or([0,0,0,0])));
                        if ui.add_sized([ui.available_width(), 30.0], egui::Button::new("🎯 Выбрать область")).clicked() {
                            self.is_selecting_region = true;
                            self.selection_bg = None;
                        }
                    });

                    ui.add_space(10.0);
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.add_space(5.0);
                        
                        let (p_text, p_color) = if self.config.print_enabled {
                            ("ПЕЧАТЬ: ВКЛ", egui::Color32::from_rgb(50, 150, 50))
                        } else {
                            ("ПЕЧАТЬ: ВЫКЛ", egui::Color32::from_rgb(200, 100, 50))
                        };
                        
                        if ui.add_sized(
                            [ui.available_width(), 40.0], 
                            egui::Button::new(egui::RichText::new(p_text).strong().color(egui::Color32::WHITE)).fill(p_color)
                        ).clicked() {
                            self.config.print_enabled = !self.config.print_enabled;
                            self.config_manager.save(&self.config).ok();
                        }
                        
                        ui.add_space(8.0);
                        
                        let (d_text, d_color) = if self.config.debug_mode {
                            ("🐞 ОТЛАДКА: ВКЛ", egui::Color32::from_rgb(180, 150, 50))
                        } else {
                            ("🐞 ОТЛАДКА: ВЫКЛ", egui::Color32::from_rgb(100, 100, 100))
                        };
                        
                        if ui.add_sized(
                            [ui.available_width(), 30.0],
                            egui::Button::new(egui::RichText::new(d_text).strong().color(egui::Color32::WHITE)).fill(d_color)
                        ).clicked() {
                            self.config.debug_mode = !self.config.debug_mode;
                            self.config_manager.save(&self.config).ok();
                        }
                    });
                });

                // RIGHT COLUMN: STATUS & LOGS
                cols[1].vertical(|ui| {
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.vertical_centered(|ui| {
                            ui.label("Ячейка:");
                            ui.heading(egui::RichText::new(&self.detected_text).size(40.0).color(egui::Color32::LIGHT_BLUE).strong());
                        });
                    });

                    ui.add_space(10.0);
                    
                    let (btn_text, btn_color) = if self.is_running {
                        ("⏹ ОСТАНОВИТЬ", egui::Color32::from_rgb(200, 50, 50))
                    } else {
                        ("▶ ЗАПУСТИТЬ", egui::Color32::from_rgb(50, 150, 50))
                    };

                    if ui.add_sized(
                        [ui.available_width(), 50.0], 
                        egui::Button::new(egui::RichText::new(btn_text).size(20.0).strong().color(egui::Color32::WHITE))
                            .fill(btn_color)
                    ).clicked() {
                        self.toggle_monitoring(ctx);
                    }

                    ui.add_space(10.0);
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.set_height(ui.available_height() - 10.0);
                        ui.label(egui::RichText::new("📋 Журнал событий").strong());
                        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                            ui.add_sized([ui.available_width(), 200.0], egui::Label::new(&self.logs).wrap());
                        });
                    });
                });
            });

            // Version Label Bottom-Right
            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.add_space(5.0);
                ui.label(egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).color(egui::Color32::GRAY).small());
            });
        });
    }
}
