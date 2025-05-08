use std::net::{ TcpStream, SocketAddr};
use std::time::Duration;
use std::io::{Read, Write};
use common::{FirstServerRequest, FirstServerResponse, MemoryUnit, SecondServerRequest, SecondServerResponse, ServerNum};
use eframe::egui::{self};
use serde_json::to_vec;

// Параметры приложения (размер окна, название и прочее)
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 320.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Client",
        options,
        Box::new(|_| {
            Ok(Box::<MyApp>::default())
        }),
    )
}

// Все состояния приложения
struct MyApp {
    server_1: Option<TcpStream>,
    server_2: Option<TcpStream>,
    last_error: Option<String>,
    selected_unit: MemoryUnit,
    server_1_last_response: Option<String>,
    server_2_last_response: Option<String>
}

// Состояние приложения при запуске
impl Default for MyApp {
    fn default() -> Self {
        Self {
            server_1: None,
            server_2: None,
            last_error: None,
            selected_unit: MemoryUnit::Bytes,
            server_1_last_response: None,
            server_2_last_response: None,
        }
    }
}

impl MyApp {
    // Метод для подключения к серверам
    fn connect(&mut self, server_number: ServerNum) {
        match server_number {
            ServerNum::Server1 => {
                match TcpStream::connect_timeout(&SocketAddr::from(([192, 168, 1, 148], 8080)), Duration::from_millis(200)) {
                    Ok(stream) => {
                        self.server_1 = Some(stream);
                        self.last_error = None;
                    },
                    Err(e) => {
                        self.last_error = Some(format!("Error during server 1 connection: {}", e));
                    }
                }
            },
            ServerNum::Server2 => {
                match TcpStream::connect_timeout(&SocketAddr::from(([192, 168, 1, 148], 8081)), Duration::from_millis(200)) {
                    Ok(stream) => {
                        self.server_2 = Some(stream);
                        self.last_error = None;
                    },
                    Err(e) => {
                        self.last_error = Some(format!("Error during server 2 connection: {}", e));
                    }
                }
            },
        }
    }

    // Метод для отключения от серверов
    fn disconnect(&mut self, server_number: ServerNum) {
        match server_number {
            ServerNum::Server1 => {
                self.server_1 = None;
                self.last_error = None;
                self.server_1_last_response = None;
            }
            ServerNum::Server2 => {
                self.server_2 = None;
                self.last_error = None;
                self.server_2_last_response = None;
            }
        }
    }

    // Метод для отправки запроса на первый сервер
    fn first_server_request(&mut self) -> Result<String, String> {
        if let Some(stream) = &mut self.server_1 {
            let unit = self.selected_unit;
            let request = FirstServerRequest { unit };
            let request_json = to_vec(&request).map_err(|e| e.to_string())?;

            stream.write_all(&request_json).map_err(|e| e.to_string())?;

            let mut buffer = [0; 1024];
            let size = stream.read(&mut buffer).map_err(|e| e.to_string())?;

            let response: FirstServerResponse = serde_json::from_slice(&buffer[..size])
                                                            .map_err(|e| e.to_string())?;

            Ok(format!(
                "Host: {}\nUser: {}\nFree memory: {:.2} {}\nTime: {}",
                response.hostname,
                response.username,
                response.free_memory,
                response.unit,
                response.timestamp,
            ))
        } else {
            Err("Not connected to the first server".to_string())
        }
    }

    // Метод для отправки запроса на второй сервер
    fn second_server_request(&mut self) -> Result<String, String> {
        if let Some(stream) = &mut self.server_2 {
            let request = SecondServerRequest { request: Some(()) };
            let request_json = to_vec(&request).map_err(|e| e.to_string())?;

            stream.write_all(&request_json).map_err(|e| e.to_string())?;

            let mut buffer = [0; 2048];
            let size = stream.read(&mut buffer).map_err(|e| e.to_string())?;

            let response: SecondServerResponse = serde_json::from_slice(&buffer[..size])
                                                            .map_err(|e| e.to_string())?;

            Ok(format!(
                "Server process priority: {}\nThread ids: {:?}\nTime: {}",
                response.priority,
                response.thread_ids,
                response.timestamp,
            ))
        } else {
            Err("Not connected to the second server".to_string())
        }
    }
}

// Основа приложения
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // UI для работы с первым сервером
            if self.server_1.is_none() {
                if ui.button("Connect to the first server").clicked() {
                    self.connect(ServerNum::Server1);
                }
            } else {
                ui.label(format!("Connected to the first server"));
                ui.horizontal(|ui| {
                    ui.label("Memory unit: ");
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", self.selected_unit))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_unit, MemoryUnit::Bytes, "Bytes");
                            ui.selectable_value(&mut self.selected_unit, MemoryUnit::MegaBytes, "MB");
                            ui.selectable_value(&mut self.selected_unit, MemoryUnit::GigaBytes, "GB");
                        }
                    );
                    if ui.button("Send").clicked() {
                        match self.first_server_request() {
                            Ok(response) => {
                                self.server_1_last_response = Some(response);
                                self.last_error = None;
                            },
                            Err(e) => {
                                self.last_error = Some(format!("Error during Server 1 request: {}", e));
                            }
                        }
                    }
                });
                if ui.button("Disconnect from the first server").clicked() {
                    self.disconnect(ServerNum::Server1);
                }
                if self.server_1_last_response.is_some() {
                    let mut text = self.server_1_last_response.as_ref().unwrap().to_owned();
                    ui.text_edit_multiline(&mut text);
                }
            }

            ui.add_space(15.);

            // UI для работы со вторым сервером
            if self.server_2.is_none() {
                if ui.button("Connect to the second server").clicked() {
                    self.connect(ServerNum::Server2);
                }
            } else {
                ui.label(format!("Connected to the second server"));
                if ui.button("Send").clicked() {
                    match self.second_server_request() {
                        Ok(response) => {
                            self.server_2_last_response = Some(response);
                            self.last_error = None;
                        },
                        Err(e) => {
                            self.last_error = Some(format!("Error during Server 2 request: {}", e));
                        }
                    }
                }
                if ui.button("Disconnect from the second server").clicked() {
                    self.disconnect(ServerNum::Server2);
                }
                if self.server_2_last_response.is_some() {
                    let mut text = self.server_2_last_response.as_ref().unwrap().to_owned();
                    ui.text_edit_multiline(&mut text);
                }
            }
            
            ui.add_space(15.);
            
            // Поле для вывода последней ошибки
            if self.last_error.is_some() {
                ui.label(egui::RichText::new(format!("{}", self.last_error.as_ref().unwrap()))
                                                    .color(egui::Color32::from_rgb(255, 0, 0)));
            }
        });
    }
}