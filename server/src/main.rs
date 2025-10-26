use eframe::egui;
use des::cipher::{BlockEncrypt, KeyInit};
use des::Des;
use hex::{encode, decode};
use chrono::{NaiveDateTime, Local};
use std::fs;
use std::path::Path;
use std::time::Instant;
use encoding_rs::WINDOWS_1251;

const DATABASE_FILE: &str = "database.txt";

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([650.0, 750.0])
            .with_resizable(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "Сервер аутентификации",
        options,
        Box::new(|_cc| Box::<ServerApp>::default()),
    )
}

#[derive(Clone)]
struct User {
    name: String,
    login: String,
    bs2: String,
    date: String,
    time: String,
}

struct ServerApp {
    database_text: String,
    login: String,
    password: String,
    time_window: String,
    result_message: String,
    result_type: ResultType,
    response_time: String,
    users: Vec<User>,
    error_message: String,
    success_message: String,
}

#[derive(PartialEq)]
enum ResultType {
    None,
    Success,
    Error,
}

impl Default for ServerApp {
    fn default() -> Self {
        let mut app = Self {
            database_text: String::new(),
            login: String::new(),
            password: String::new(),
            time_window: "20".to_string(),
            result_message: "Ожидание аутентификации...".to_string(),
            result_type: ResultType::None,
            response_time: "-".to_string(),
            users: Vec::new(),
            error_message: String::new(),
            success_message: String::new(),
        };
        
        app.load_database();
        app
    }
}

impl ServerApp {
    fn load_database(&mut self) {
        if let Ok(content) = read_file_windows1251(DATABASE_FILE) {
            self.database_text = content;
            self.parse_database();
            self.success_message = format!("База загружена: {} пользователей", self.users.len());
        }
    }
    
    fn save_database(&mut self) {
        match write_file_windows1251(DATABASE_FILE, &self.database_text) {
            Ok(_) => {
                self.parse_database();
                self.success_message = format!("База сохранена: {} пользователей", self.users.len());
                self.error_message.clear();
            }
            Err(e) => {
                self.error_message = format!("Ошибка сохранения: {}", e);
                self.success_message.clear();
            }
        }
    }
    
    fn clear_database(&mut self) {
        self.database_text.clear();
        self.users.clear();
        let _ = fs::remove_file(DATABASE_FILE);
        self.success_message = "База данных очищена".to_string();
        self.error_message.clear();
    }
    
    fn parse_database(&mut self) {
        self.users.clear();
        
        for line in self.database_text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                self.users.push(User {
                    name: parts[0].to_string(),
                    login: parts[1].to_string(),
                    bs2: parts[2].to_uppercase(),
                    date: parts[3].to_string(),
                    time: parts[4].to_string(),
                });
            }
        }
    }
    
    fn authenticate(&mut self) {
        let start_time = Instant::now();
        self.error_message.clear();
        self.success_message.clear();
        
        let login = self.login.trim().to_string();  // .to_string() клонирует
        let password = self.password.trim().to_uppercase();
        
        if login.is_empty() || password.is_empty() {
            self.show_result("Заполните логин и пароль!", ResultType::Error, start_time);
            return;
        }
        
        if self.users.is_empty() {
            self.parse_database();
        }
        
        if self.users.is_empty() {
            self.show_result("База данных пуста!", ResultType::Error, start_time);
            return;
        }
        
        let user = match self.users.iter().find(|u| u.login == *login) {
            Some(u) => u,
            None => {
                self.show_result("❌ Доступ запрещен: пользователь не найден", ResultType::Error, start_time);
                return;
            }
        };
        
        let time_window: i64 = match self.time_window.parse() {
            Ok(v) => v,
            Err(_) => {
                self.show_result("Неверное временное окно!", ResultType::Error, start_time);
                return;
            }
        };
        
        let datetime_str = format!("{} {}", user.date, user.time);
        let start_datetime = match parse_datetime(&datetime_str) {
            Ok(dt) => dt,
            Err(e) => {
                self.show_result(&format!("Ошибка формата даты в БД: {}", e), ResultType::Error, start_time);
                return;
            }
        };
        
        let current_time = Local::now().naive_local();
        let mut authenticated = false;
        
        for offset in -time_window..=time_window {
            let test_time = current_time + chrono::Duration::seconds(offset);
            let time_diff = (test_time.and_utc().timestamp() - start_datetime.and_utc().timestamp()).max(0) as u64;
            let time_block = format!("{:016X}", time_diff);
            
            if let Ok(expected_password) = des_encrypt(&time_block, &user.bs2) {
                if expected_password == password {
                    authenticated = true;
                    break;
                }
            }
        }
        
        if authenticated {
            self.show_result("✅ Доступ разрешен", ResultType::Success, start_time);
        } else {
            self.show_result("❌ Доступ запрещен: неверный пароль", ResultType::Error, start_time);
        }
    }
    
    fn show_result(&mut self, message: &str, result_type: ResultType, start_time: Instant) {
        self.result_message = message.to_string();
        self.result_type = result_type;
        self.response_time = format!("{} мс", start_time.elapsed().as_millis());
    }
}

impl eframe::App for ServerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            
            ui.vertical_centered(|ui| {
                ui.heading("🔒 Сервер аутентификации");
            });
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            
            // Инфо-блок
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(231, 243, 255))
                .inner_margin(10.0)
                .rounding(5.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("📋 Формат базы данных:").color(egui::Color32::from_rgb(8, 66, 152)));
                    ui.label(egui::RichText::new("Фамилия_И.О. Логин БазовыйСекрет2 ДД.ММ.ГГГГ ЧЧ:ММ:СС").color(egui::Color32::from_rgb(8, 66, 152)));
                    ui.label(egui::RichText::new("Пример: Іваненко_І.І. Johnny AAAAE2D76510BF24 06.05.2007 21:24:30").italics().color(egui::Color32::from_rgb(8, 66, 152)));
                });
            
            ui.add_space(10.0);
            
            // База данных
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(248, 249, 250))
                .inner_margin(10.0)
                .rounding(5.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("📊 База данных пользователей").strong());
                    ui.add_space(5.0);
                    
                    ui.add(
                        egui::TextEdit::multiline(&mut self.database_text)
                            .desired_rows(8)
                            .font(egui::TextStyle::Monospace)
                            .hint_text("Лапин_Е.В. Johnny AE23e2d76510bf24 06.05.2007 21:24:30\nМатюшенко_Н.В. mtkolya ED7240deba345612 14.12.1985 18:00:00")
                    );
                    
                    ui.add_space(5.0);
                    ui.label(format!("Загружено пользователей: {}", self.users.len()));
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("💾 Сохранить").clicked() {
                            self.save_database();
                        }
                        if ui.button("📂 Загрузить").clicked() {
                            self.load_database();
                        }
                        if ui.button("🗑️ Очистить").clicked() {
                            self.clear_database();
                        }
                    });
                });
            
            ui.add_space(15.0);
            
            // Аутентификация
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(102, 126, 234))
                .inner_margin(15.0)
                .rounding(5.0)
                .show(ui, |ui| {
                    ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
                    
                    ui.label(egui::RichText::new("🔐 Аутентификация").strong().size(16.0));
                    ui.add_space(10.0);
                    
                    ui.label("Введите логин:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.login)
                            .hint_text("Johnny")
                            .font(egui::TextStyle::Monospace)
                    );
                    
                    ui.add_space(5.0);
                    
                    ui.label("Введите пароль:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.password)
                            .hint_text("0200000061290047")
                            .font(egui::TextStyle::Monospace)
                    );
                    
                    ui.add_space(10.0);
                    
                    if ui.add_sized([ui.available_width(), 40.0], 
                        egui::Button::new(egui::RichText::new("🚀 Получить доступ").strong())
                    ).clicked() {
                        self.authenticate();
                    }
                });
            
            ui.add_space(15.0);
            
            // Сообщения
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
            
            // Результат
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(248, 249, 250))
                .inner_margin(15.0)
                .rounding(5.0)
                .show(ui, |ui| {
                    let color = match self.result_type {
                        ResultType::Success => egui::Color32::from_rgb(212, 237, 218),
                        ResultType::Error => egui::Color32::from_rgb(248, 215, 218),
                        ResultType::None => egui::Color32::from_rgb(233, 236, 239),
                    };
                    
                    let text_color = match self.result_type {
                        ResultType::Success => egui::Color32::from_rgb(21, 87, 36),
                        ResultType::Error => egui::Color32::from_rgb(114, 28, 36),
                        ResultType::None => egui::Color32::from_rgb(108, 117, 125),
                    };
                    
                    egui::Frame::none()
                        .fill(color)
                        .inner_margin(15.0)
                        .rounding(5.0)
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new(&self.result_message)
                                    .color(text_color)
                                    .strong()
                                    .size(16.0));
                            });
                        });
                    
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new("Время ответа").color(egui::Color32::from_gray(100)));
                            ui.label(egui::RichText::new(&self.response_time).strong().size(18.0));
                        });
                    });
                });
            
            ui.add_space(15.0);
            
            // Настройки
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(248, 249, 250))
                .inner_margin(10.0)
                .rounding(5.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("⚙️ Временное окно (секунды):");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.time_window)
                                .desired_width(80.0)
                        );
                    });
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
