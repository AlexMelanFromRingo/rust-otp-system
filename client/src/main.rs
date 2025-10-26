use eframe::egui;
use des::cipher::{BlockEncrypt, KeyInit};
use des::Des;
use hex::{encode, decode};
use chrono::{NaiveDateTime, Local};
use std::fs;
use std::path::Path;
use encoding_rs::WINDOWS_1251;

const BS1_FILE: &str = "bs1.txt";
const BS3_FILE: &str = "bs3.txt";

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([550.0, 700.0])
            .with_resizable(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "Генератор одноразовых паролей",
        options,
        Box::new(|_cc| Box::<ClientApp>::default()),
    )
}

struct ClientApp {
    pin: String,
    bs1: String,
    bs3: String,
    password: String,
    bs2: String,
    time_block: String,
    error_message: String,
    success_message: String,
}

impl Default for ClientApp {
    fn default() -> Self {
        let mut app = Self {
            pin: String::new(),
            bs1: String::new(),
            bs3: String::new(),
            password: String::new(),
            bs2: String::new(),
            time_block: String::new(),
            error_message: String::new(),
            success_message: String::new(),
        };
        
        app.load_config();
        app
    }
}

impl ClientApp {
    fn load_config(&mut self) {
        if let Ok(content) = read_file_windows1251(BS1_FILE) {
            self.bs1 = content.trim().to_uppercase();
        }
        
        if let Ok(content) = read_file_windows1251(BS3_FILE) {
            self.bs3 = content.trim().to_string();
        }
        
        if !self.bs1.is_empty() || !self.bs3.is_empty() {
            self.success_message = "Конфигурация загружена".to_string();
        }
    }
    
    fn save_config(&mut self) {
        let mut errors = Vec::new();
        
        if let Err(e) = write_file_windows1251(BS1_FILE, &self.bs1) {
            errors.push(format!("BS1: {}", e));
        }
        
        if let Err(e) = write_file_windows1251(BS3_FILE, &self.bs3) {
            errors.push(format!("BS3: {}", e));
        }
        
        if errors.is_empty() {
            self.success_message = "Конфигурация сохранена!".to_string();
            self.error_message.clear();
        } else {
            self.error_message = format!("Ошибки: {}", errors.join(", "));
            self.success_message.clear();
        }
    }
    
    fn clear_config(&mut self) {
        self.bs1.clear();
        self.bs3.clear();
        self.pin.clear();
        self.password.clear();
        self.bs2.clear();
        self.time_block.clear();
        
        let _ = fs::remove_file(BS1_FILE);
        let _ = fs::remove_file(BS3_FILE);
        
        self.success_message = "Конфигурация очищена".to_string();
        self.error_message.clear();
    }
    
    fn generate_password(&mut self) {
        self.error_message.clear();
        self.success_message.clear();
        
        let pin = self.pin.trim().to_uppercase();
        if pin.len() != 4 || !pin.chars().all(|c| c.is_ascii_hexdigit()) {
            self.error_message = "PIN: 4 HEX символа (0-9, A-F)!".to_string();
            return;
        }
        
        let bs1 = self.bs1.trim().to_uppercase();
        if bs1.len() != 12 || !bs1.chars().all(|c| c.is_ascii_hexdigit()) {
            self.error_message = "BS1: 12 HEX символов!".to_string();
            return;
        }
        
        self.bs2 = format!("{}{}", pin, bs1);
        
        let start_time = match parse_datetime(self.bs3.trim()) {
            Ok(dt) => dt,
            Err(e) => {
                self.error_message = format!("Дата: {}. Формат: ДД.ММ.ГГГГ ЧЧ:ММ:СС", e);
                return;
            }
        };
        
        let current_time = Local::now().naive_local();
        let time_diff = (current_time.and_utc().timestamp() - start_time.and_utc().timestamp()).max(0) as u64;
        
        self.time_block = format!("{:016X}", time_diff);
        
        match des_encrypt(&self.time_block, &self.bs2) {
            Ok(encrypted) => {
                self.password = encrypted;
                self.success_message = "Пароль сгенерирован!".to_string();
            }
            Err(e) => {
                self.error_message = format!("Шифрование: {}", e);
            }
        }
    }
}

impl eframe::App for ClientApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            
            ui.vertical_centered(|ui| {
                ui.heading("🔐 Генератор одноразовых паролей");
            });
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(248, 249, 250))
                .inner_margin(10.0)
                .rounding(5.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("⚙️ Конфигурация").strong());
                    ui.add_space(5.0);
                    
                    ui.label("Базовый секрет 1 (48-bit HEX):");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.bs1)
                            .hint_text("e2d76510bf24")
                            .font(egui::TextStyle::Monospace)
                    );
                    
                    ui.add_space(5.0);
                    
                    ui.label("Начальная настройка часов (ДД.ММ.ГГГГ ЧЧ:ММ:СС):");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.bs3)
                            .hint_text("06.05.2007 21:24:30")
                    );
                    
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("💾 Сохранить").clicked() {
                            self.save_config();
                        }
                        if ui.button("📂 Загрузить").clicked() {
                            self.load_config();
                        }
                        if ui.button("🗑️ Очистить").clicked() {
                            self.clear_config();
                        }
                    });
                });
            
            ui.add_space(15.0);
            
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(40, 167, 69))
                .inner_margin(15.0)
                .rounding(5.0)
                .show(ui, |ui| {
                    ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
                    
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("Введите PIN-код (4 HEX):").strong());
                        ui.add_space(5.0);
                        
                        ui.add(
                            egui::TextEdit::singleline(&mut self.pin)
                                .hint_text("AAAA")
                                .font(egui::TextStyle::Heading)
                                .char_limit(4)
                        );
                    });
                    
                    ui.add_space(10.0);
                    
                    if ui.add_sized([ui.available_width(), 40.0], 
                        egui::Button::new(egui::RichText::new("🔑 Получить пароль").strong())
                    ).clicked() {
                        self.generate_password();
                    }
                    
                    ui.add_space(10.0);
                    
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgba_premultiplied(255, 255, 255, 50))
                        .inner_margin(10.0)
                        .rounding(5.0)
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                if self.password.is_empty() {
                                    ui.label(egui::RichText::new("Пароль появится здесь").color(egui::Color32::from_gray(200)));
                                } else {
                                    ui.label(egui::RichText::new(&self.password)
                                        .font(egui::FontId::monospace(20.0))
                                        .strong()
                                    );
                                }
                            });
                        });
                });
            
            ui.add_space(15.0);
            
            if !self.error_message.is_empty() {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(248, 215, 218))
                    .inner_margin(10.0)
                    .rounding(5.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(&self.error_message).color(egui::Color32::from_rgb(114, 28, 36)));
                    });
                ui.add_space(10.0);
            }
            
            if !self.success_message.is_empty() {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(212, 237, 218))
                    .inner_margin(10.0)
                    .rounding(5.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(&self.success_message).color(egui::Color32::from_rgb(21, 87, 36)));
                    });
                ui.add_space(10.0);
            }
            
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(248, 249, 250))
                .inner_margin(10.0)
                .rounding(5.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Служебная информация:").strong().color(egui::Color32::from_gray(100)));
                    ui.add_space(5.0);
                    
                    ui.label(format!("Базовый секрет 1: {}", if self.bs1.is_empty() { "-" } else { &self.bs1 }));
                    ui.label(format!("Базовый секрет 2 (ключ): {}", if self.bs2.is_empty() { "-" } else { &self.bs2 }));
                    ui.label(format!("Начальная настройка: {}", if self.bs3.is_empty() { "-" } else { &self.bs3 }));
                    ui.label(format!("Показание часов (блок): {}", if self.time_block.is_empty() { "-" } else { &self.time_block }));
                });
        });
    }
}

fn parse_datetime(s: &str) -> Result<NaiveDateTime, String> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return Err("Неверный формат".to_string());
    }
    
    let date_parts: Vec<&str> = parts[0].split('.').collect();
    let time_parts: Vec<&str> = parts[1].split(':').collect();
    
    if date_parts.len() != 3 || time_parts.len() != 3 {
        return Err("Неверный формат".to_string());
    }
    
    let day: u32 = date_parts[0].parse().map_err(|_| "Неверный день")?;
    let month: u32 = date_parts[1].parse().map_err(|_| "Неверный месяц")?;
    let year: i32 = date_parts[2].parse().map_err(|_| "Неверный год")?;
    
    let hour: u32 = time_parts[0].parse().map_err(|_| "Неверный час")?;
    let minute: u32 = time_parts[1].parse().map_err(|_| "Неверная минута")?;
    let second: u32 = time_parts[2].parse().map_err(|_| "Неверная секунда")?;
    
    NaiveDateTime::parse_from_str(
        &format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", year, month, day, hour, minute, second),
        "%Y-%m-%d %H:%M:%S"
    ).map_err(|e| format!("Ошибка: {}", e))
}

fn des_encrypt(data_hex: &str, key_hex: &str) -> Result<String, String> {
    let data_bytes = decode(data_hex).map_err(|e| format!("Данные: {}", e))?;
    let key_bytes = decode(key_hex).map_err(|e| format!("Ключ: {}", e))?;
    
    if data_bytes.len() != 8 {
        return Err(format!("Данные: 8 байт, получено {}", data_bytes.len()));
    }
    
    if key_bytes.len() != 8 {
        return Err(format!("Ключ: 8 байт, получено {}", key_bytes.len()));
    }
    
    let cipher = Des::new_from_slice(&key_bytes)
        .map_err(|e| format!("Cipher: {}", e))?;
    
    // ✅ Правильное преобразование
    let mut block = [0u8; 8];
    block.copy_from_slice(&data_bytes);
    
    let mut block_array = block.into();
    cipher.encrypt_block(&mut block_array);
    
    Ok(encode(block_array).to_uppercase())
}

fn read_file_windows1251<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
    let bytes = fs::read(path)?;
    let (decoded, _, _) = WINDOWS_1251.decode(&bytes);
    Ok(decoded.into_owned())
}

fn write_file_windows1251<P: AsRef<Path>>(path: P, content: &str) -> Result<(), std::io::Error> {
    let (encoded, _, _) = WINDOWS_1251.encode(content);
    fs::write(path, encoded.as_ref())
}
